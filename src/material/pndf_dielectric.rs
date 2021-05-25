use std::sync::Arc;

use cgmath::{ElementWise, InnerSpace, Matrix2, Vector2};
use image::GenericImageView;

use crate::core::color::Color;
use crate::core::intersection::Intersection;
use crate::core::material::Material;
use crate::core::scatter::Scatter;
use crate::core::texture::{self, Texture};
use crate::scatter::{FresnelDielectricRR, LambertReflect, PndfAccel, PndfGaussTerm, PndfReflect};

pub struct PndfDielectric {
    ior: f32,
    albedo: Arc<dyn Texture<Color>>,
    sigma_r: f32,
    sigma_hx: f32,
    sigma_hy: f32,
    base_normal_tiling: Vector2<f32>,
    base_normal_offset: Vector2<f32>,
    emissive: Arc<dyn Texture<Color>>,
    normal_map: Arc<dyn Texture<Color>>,
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
        base_normal_tiling: Vector2<f32>,
        base_normal_offset: Vector2<f32>,
        h: f32,
        emissive: Arc<dyn Texture<Color>>,
        normal_map: Arc<dyn Texture<Color>>,
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
                let jacobian = Matrix2::from_cols(dsdu, dsdv);

                let term = PndfGaussTerm::new(
                    Vector2::new(u, v),
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
        let bvh = PndfAccel::new(terms_ref, 5, 10);

        Self {
            ior,
            albedo,
            sigma_r,
            sigma_hx,
            sigma_hy,
            base_normal_tiling,
            base_normal_offset,
            emissive,
            normal_map,
            _terms: terms,
            bvh,
        }
    }
}

impl Material for PndfDielectric {
    fn apply_normal_map(&self, inter: &Intersection<'_>) -> cgmath::Vector3<f32> {
        texture::get_normal_at(&self.normal_map, inter)
    }

    fn scatter(&self, inter: &Intersection<'_>) -> Box<dyn Scatter> {
        let albedo = self.albedo.value_at(inter);
        let u = Vector2::new(inter.texcoords.x, inter.texcoords.y)
            .mul_element_wise(self.base_normal_tiling)
            + self.base_normal_offset;
        let (u_new, v_new) = crate::texture::util::wrap_uv(u.x, u.y);
        let sigma_p = inter.duvdx.magnitude().max(inter.duvdy.magnitude()) + 0.0001;
        let bvh: *const PndfAccel = &self.bvh;

        // Box::new(PndfReflect::new(
        //     albedo,
        //     u,
        //     sigma_p,
        //     self.sigma_hx,
        //     self.sigma_hy,
        //     self.sigma_r,
        //     bvh,
        // )) as Box<dyn Scatter>
        Box::new(FresnelDielectricRR::new(
            self.ior,
            PndfReflect::new(
                Color::WHITE,
                Vector2::new(u_new, v_new),
                sigma_p,
                self.sigma_hx,
                self.sigma_hy,
                self.sigma_r,
                bvh,
            ),
            LambertReflect::new(albedo),
        )) as Box<dyn Scatter>
    }

    fn emissive(&self, inter: &Intersection<'_>) -> Color {
        self.emissive.value_at(inter)
    }
}

fn get_normal_bilinear(image: &image::DynamicImage, u: f32, v: f32) -> Vector2<f32> {
    let normal_color = crate::texture::util::sample_blinear(image, u, v);
    let normal_color = normal_color * 2.0 - Color::WHITE;
    let normal = cgmath::Vector3::new(normal_color.r, normal_color.g, normal_color.b).normalize();
    Vector2::new(normal.x, normal.y)
}
