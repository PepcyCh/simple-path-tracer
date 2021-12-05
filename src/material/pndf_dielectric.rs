use std::sync::Arc;

use crate::{
    core::{
        color::Color, intersection::Intersection, loader::InputParams,
        scene_resources::SceneResources,
    },
    material::MaterialT,
    scatter::{
        FresnelDielectricRR, LambertReflect, MicrofacetReflect, PndfAccel, PndfGaussTerm,
        PndfReflect, Scatter, SpecularReflect,
    },
    texture::{Texture, TextureChannel, TextureInput, TextureT},
};

pub struct PndfDielectric {
    ior: f32,
    albedo: Arc<Texture>,
    sigma_r: f32,
    sigma_hx: f32,
    sigma_hy: f32,
    base_normal_tiling: glam::Vec2,
    base_normal_offset: glam::Vec2,
    fallback_roughness: Arc<Texture>,
    /// used to avoid drop of terms
    _terms: Vec<PndfGaussTerm>,
    bvh: PndfAccel,
}

impl PndfDielectric {
    pub fn new(
        ior: f32,
        albedo: Arc<Texture>,
        sigma_r: f32,
        base_normal: Arc<Texture>,
        fallback_roughness: Arc<Texture>,
        h: f32,
    ) -> Self {
        let h_inv = 1.0 / h;
        let (normal_width, normal_height, _) = base_normal.dimensions().unwrap();
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
                let s = get_normal_bilinear(base_normal.as_ref(), u, v);

                let s_up = get_normal_bilinear(base_normal.as_ref(), u + 0.5 * hx, v);
                let s_un = get_normal_bilinear(base_normal.as_ref(), u - 0.5 * hx, v);
                let dsdu = (s_up - s_un) * hx_inv;
                let s_vp = get_normal_bilinear(base_normal.as_ref(), u, v + 0.5 * hy);
                let s_vn = get_normal_bilinear(base_normal.as_ref(), u, v - 0.5 * hy);
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

        let base_normal_tiling = base_normal.tiling().truncate();
        let base_normal_offset = base_normal.offset().truncate();

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

    pub fn load(rsc: &mut SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let ior = params.get_float("ior")?;

        let albedo = rsc.clone_texture(params.get_str("albedo")?)?;
        let base_normal = rsc.clone_texture(params.get_str("base_normal")?)?;
        let fallback_roughness = rsc.clone_texture(params.get_str("fallback_roughness")?)?;

        let sigma_r = params.get_float("sigma_r")?;
        let h = params.get_float("h")?;

        if base_normal.dimensions().is_none() {
            anyhow::bail!(format!(
                "{} - 'base_normal' should be a Texture with non-None dimensions",
                params.name()
            ));
        }

        Ok(Self::new(
            ior,
            albedo,
            sigma_r,
            base_normal,
            fallback_roughness,
            h,
        ))
    }
}

impl MaterialT for PndfDielectric {
    fn scatter(&self, inter: &Intersection<'_>) -> Scatter {
        let albedo = self.albedo.color_at(inter.into());
        let u = inter.texcoords * self.base_normal_tiling + self.base_normal_offset;
        let u_new = wrap_uv(u);
        let duvdx = inter.duvdx * self.base_normal_tiling;
        let duvdy = inter.duvdy * self.base_normal_tiling;
        let sigma_p = duvdx.length().max(duvdy.length()) / 3.0;
        let bvh: *const PndfAccel = &self.bvh;

        if sigma_p > 0.0 {
            FresnelDielectricRR::new(
                self.ior,
                PndfReflect::new(
                    Color::WHITE,
                    u_new,
                    sigma_p,
                    self.sigma_hx,
                    self.sigma_hy,
                    self.sigma_r,
                    bvh,
                ),
                LambertReflect::new(albedo),
            )
            .into()
        } else {
            let roughness = self
                .fallback_roughness
                .float_at(inter.into(), TextureChannel::R);
            if roughness < 0.001 {
                FresnelDielectricRR::new(
                    self.ior,
                    SpecularReflect::new(Color::WHITE),
                    LambertReflect::new(albedo),
                )
                .into()
            } else {
                FresnelDielectricRR::new(
                    self.ior,
                    MicrofacetReflect::new(Color::WHITE, roughness, roughness),
                    LambertReflect::new(albedo),
                )
                .into()
            }
        }
    }
}

fn get_normal_bilinear(tex: &Texture, u: f32, v: f32) -> glam::Vec2 {
    let normal_color = tex.color_at(TextureInput::specified(glam::Vec3A::new(u, v, 0.0)));
    let normal = glam::Vec3A::new(normal_color.r, normal_color.g, normal_color.b).normalize();
    glam::Vec2::new(normal.x, normal.y)
}

fn wrap_uv(uv: glam::Vec2) -> glam::Vec2 {
    let u_new = if uv.x >= 0.0 {
        uv.x.fract()
    } else {
        1.0 + uv.x.fract()
    };
    let v_new = if uv.y >= 0.0 {
        uv.y.fract()
    } else {
        1.0 + uv.y.fract()
    };
    glam::Vec2::new(u_new, v_new)
}
