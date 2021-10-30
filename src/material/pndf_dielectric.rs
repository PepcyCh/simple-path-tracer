use std::sync::Arc;

use image::GenericImageView;

use crate::{
    core::{
        color::Color, intersection::Intersection, material::Material, scatter::Scatter,
        scene::Scene, texture::Texture,
    },
    loader::{self, JsonObject, Loadable},
    scatter::{
        FresnelDielectricRR, LambertReflect, MicrofacetReflect, PndfAccel, PndfGaussTerm,
        PndfReflect, SpecularReflect,
    },
};

pub struct PndfDielectric {
    ior: f32,
    albedo: Arc<dyn Texture<Color>>,
    sigma_r: f32,
    sigma_hx: f32,
    sigma_hy: f32,
    base_normal_tiling: glam::Vec2,
    base_normal_offset: glam::Vec2,
    fallback_roughness: Arc<dyn Texture<f32>>,
    /// used to avoid drop of terms
    _terms: Vec<PndfGaussTerm>,
    bvh: PndfAccel,
}

impl PndfDielectric {
    pub fn new(
        ior: f32,
        albedo: Arc<dyn Texture<Color>>,
        sigma_r: f32,
        base_normal: image::DynamicImage,
        base_normal_tiling: glam::Vec2,
        base_normal_offset: glam::Vec2,
        fallback_roughness: Arc<dyn Texture<f32>>,
        h: f32,
    ) -> Self {
        let h_inv = 1.0 / h;
        let (normal_width, normal_height) = base_normal.dimensions();
        let terms_count_y = (normal_height as f32 * h_inv) as usize;
        let terms_count_x = (normal_width as f32 * h_inv) as usize;
        let terms_count = terms_count_x * terms_count_y;
        let mut terms = Vec::with_capacity(terms_count);

        let hx = 1.0 / terms_count_x as f32;
        let sigma_hx = hx / (8.0 * 2.0_f32.ln()).sqrt();
        let hx_inv = 1.0 / hx;
        let hy = 1.0 / terms_count_y as f32;
        let sigma_hy = hy / (8.0 * 2.0_f32.ln()).sqrt();
        let hy_inv = 1.0 / hy;

        for i in 0..terms_count_y {
            for j in 0..terms_count_x {
                let u = (j as f32 + 0.5) * hx;
                let v = (i as f32 + 0.5) * hy;
                let s = get_normal_bilinear(&base_normal, u, v);

                let s_up = get_normal_bilinear(&base_normal, u + 0.5 * hx, v);
                let s_un = get_normal_bilinear(&base_normal, u - 0.5 * hx, v);
                let dsdu = (s_up - s_un) * hx_inv;
                let s_vp = get_normal_bilinear(&base_normal, u, v + 0.5 * hy);
                let s_vn = get_normal_bilinear(&base_normal, u, v - 0.5 * hy);
                let dsdv = (s_vp - s_vn) * hy_inv;
                let jacobian = glam::Mat2::from_cols(dsdu, dsdv);

                let term = PndfGaussTerm::new(
                    glam::Vec2::new(u, v),
                    s,
                    jacobian,
                    sigma_hx,
                    sigma_hy,
                    sigma_r,
                );
                terms.push(term);
            }
        }

        let terms_ref: Vec<_> = terms.iter_mut().collect();
        let s_block_count = ((2.0 / (sigma_r * 16.0)) as usize).clamp(1, 20);
        let bvh = PndfAccel::new(terms_ref, 5, s_block_count);

        Self {
            ior,
            albedo,
            sigma_r,
            sigma_hx,
            sigma_hy,
            base_normal_tiling,
            base_normal_offset,
            fallback_roughness,
            _terms: terms,
            bvh,
        }
    }
}

impl Material for PndfDielectric {
    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let albedo = self.albedo.value_at(inter);
        let u = glam::Vec2::new(inter.texcoords.x, inter.texcoords.y) * self.base_normal_tiling
            + self.base_normal_offset;
        let (u_new, v_new) = crate::texture::util::wrap_uv(u.x, u.y);
        let duvdx = inter.duvdx * self.base_normal_tiling;
        let duvdy = inter.duvdy * self.base_normal_tiling;
        let sigma_p = duvdx.length().max(duvdy.length()) / 3.0;
        let bvh: *const PndfAccel = &self.bvh;

        if sigma_p > 0.0 {
            Box::new(FresnelDielectricRR::new(
                self.ior,
                PndfReflect::new(
                    Color::WHITE,
                    glam::Vec2::new(u_new, v_new),
                    sigma_p,
                    self.sigma_hx,
                    self.sigma_hy,
                    self.sigma_r,
                    bvh,
                ),
                LambertReflect::new(albedo),
            )) as Box<dyn Scatter>
        } else {
            let roughness = self.fallback_roughness.value_at(inter);
            if roughness < 0.001 {
                Box::new(FresnelDielectricRR::new(
                    self.ior,
                    SpecularReflect::new(Color::WHITE),
                    LambertReflect::new(albedo),
                )) as Box<dyn Scatter>
            } else {
                Box::new(FresnelDielectricRR::new(
                    self.ior,
                    MicrofacetReflect::new(Color::WHITE, roughness),
                    LambertReflect::new(albedo),
                )) as Box<dyn Scatter>
            }
        }
    }
}

fn get_normal_bilinear(image: &image::DynamicImage, u: f32, v: f32) -> glam::Vec2 {
    let normal_color = crate::texture::util::sample_blinear(image, u, v);
    let normal_color = normal_color * 2.0 - Color::WHITE;
    let normal = glam::Vec3A::new(normal_color.r, normal_color.g, normal_color.b).normalize();
    glam::Vec2::new(normal.x, normal.y)
}

impl Loadable for PndfDielectric {
    fn load(
        scene: &mut Scene,
        path: &std::path::PathBuf,
        json_value: &JsonObject,
    ) -> anyhow::Result<()> {
        let name = loader::get_str_field(json_value, "material-pndf_dielectric", "name")?;
        let env = format!("material-pndf_dielectric({})", name);
        if scene.materials.contains_key(name) {
            anyhow::bail!(format!("{}: name is duplicated", env));
        }

        let ior = loader::get_float_field(json_value, &env, "ior")?;

        let albedo = loader::get_str_field(json_value, &env, "albedo")?;
        let albedo = if let Some(tex) = scene.textures_color.get(albedo) {
            tex.clone()
        } else {
            anyhow::bail!(format!("{}: albedo '{}' not found", env, albedo))
        };

        let sigma_r = loader::get_float_field(json_value, &env, "sigma_r")?;

        let base_normal = loader::get_image_field(json_value, &env, "base_normal", path)?;
        let base_normal_tiling =
            loader::get_float_array2_field_or(json_value, &env, "base_normal_tiling", [1.0, 1.0])?;
        let base_normal_offset =
            loader::get_float_array2_field_or(json_value, &env, "base_normal_offset", [0.0, 0.0])?;

        let fallback_roughness = loader::get_str_field(json_value, &env, "fallback_roughness")?;
        let fallback_roughness = if let Some(tex) = scene.textures_f32.get(fallback_roughness) {
            tex.clone()
        } else {
            anyhow::bail!(format!(
                "{}: fallback_roughness '{}' not found",
                env, fallback_roughness
            ))
        };

        let h = loader::get_float_field(json_value, &env, "h")?;

        let mat = PndfDielectric::new(
            ior,
            albedo,
            sigma_r,
            base_normal,
            base_normal_tiling.into(),
            base_normal_offset.into(),
            fallback_roughness,
            h,
        );
        scene.materials.insert(name.to_owned(), Arc::new(mat));

        Ok(())
    }
}
