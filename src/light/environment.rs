use crate::core::{
    alias_table::AliasTable, color::Color, loader::InputParams, rng::Rng,
    scene_resources::SceneResources,
};

use super::LightT;

pub struct EnvLight {
    texture: Vec<Vec<Color>>,
    scale: Color,
    height: usize,
    width: usize,
    alias_table: AliasTable,
    avg_power: f32,
}

impl EnvLight {
    pub fn new(texture: Vec<Vec<Color>>, scale: Color) -> Self {
        let height = texture.len();
        let width = texture[0].len();
        let size = width * height;

        let mut props = Vec::with_capacity(size);
        let mut sum = 0.0;
        let height_inv = 1.0 / height as f32;
        for (row_ind, row) in texture.iter().enumerate() {
            for pixel in row {
                let theta = (row_ind as f32 + 0.5) * height_inv;
                let prop = pixel.luminance() * theta.sin();
                sum += prop;
                props.push(prop);
            }
        }

        let sum_inv = 1.0 / sum;
        for prop in &mut props {
            *prop *= sum_inv;
        }
        let avg_power = sum / props.len() as f32;

        let alias_table = AliasTable::new(props);
        Self {
            texture,
            scale,
            height,
            width,
            alias_table,
            avg_power,
        }
    }

    fn strength_dist_pdf(&self, theta: f32, phi: f32) -> (Color, f32, f32) {
        let x = phi * 0.5 * std::f32::consts::FRAC_1_PI * self.width as f32;
        let x1 = x.round() as i32;
        let x0 = x1 - 1;
        let xt = x - x0 as f32 - 0.5;
        let x0 = x0.clamp(0, self.width as i32 - 1) as usize;
        let x1 = x1.clamp(0, self.width as i32 - 1) as usize;

        let y = theta * std::f32::consts::FRAC_1_PI * self.height as f32;
        let y1 = y.round() as i32;
        let y0 = y1 - 1;
        let yt = y - y0 as f32 - 0.5;
        let y0 = y0.clamp(0, self.height as i32 - 1) as usize;
        let y1 = y1.clamp(0, self.height as i32 - 1) as usize;

        let c00 = self.texture[y0][x0];
        let c01 = self.texture[y1][x0];
        let c10 = self.texture[y0][x1];
        let c11 = self.texture[y1][x1];
        let c0 = c00 * (1.0 - yt) + c01 * yt;
        let c1 = c10 * (1.0 - yt) + c11 * yt;
        let c = c0 * (1.0 - xt) + c1 * xt;

        let p00 = self.alias_table.probability(y0 * self.width + x0);
        let p01 = self.alias_table.probability(y1 * self.width + x0);
        let p10 = self.alias_table.probability(y0 * self.width + x1);
        let p11 = self.alias_table.probability(y1 * self.width + x1);
        let p0 = p00 * (1.0 - yt) + p01 * yt;
        let p1 = p10 * (1.0 - yt) + p11 * yt;
        let p = p0 * (1.0 - xt) * p1 * xt;

        (c * self.scale, f32::INFINITY, p)
    }

    pub fn load(rsc: &mut SceneResources, params: &mut InputParams) -> anyhow::Result<()> {
        let ty = params.get_str("type")?;
        let scale: Color = params.get_float3_or("scale", [1.0, 1.0, 1.0]).into();

        let res = match ty.as_str() {
            "color" => {
                let color = params.get_float3("color")?.into();
                Self::new(vec![vec![color]], scale)
            }
            "exr" => {
                let image = params.get_exr_image("exr_file")?;
                Self::new(image, scale)
            }
            _ => anyhow::bail!(format!("{} - unknown type", params.name())),
        };

        rsc.add_environment(res)?;

        params.check_unused_keys();

        Ok(())
    }
}

impl LightT for EnvLight {
    fn sample(&self, _position: glam::Vec3A, rng: &mut Rng) -> (glam::Vec3A, f32, Color, f32) {
        let rand = rng.uniform_1d();
        let (ind, _) = self.alias_table.sample(rand);
        let x = ind % self.width;
        let y = ind / self.width;

        let (rand_x, rand_y) = rng.uniform_2d();
        let theta = (y as f32 + rand_y) / self.height as f32 * std::f32::consts::PI;
        let phi = (x as f32 + rand_x) / self.width as f32 * 2.0 * std::f32::consts::PI;
        let sin_theta = theta.sin();
        let wi = glam::Vec3A::new(sin_theta * phi.sin(), theta.cos(), sin_theta * phi.cos());

        let (strength, dist, pdf) = self.strength_dist_pdf(theta, phi);

        (wi, pdf, strength, dist)
    }

    fn strength_dist_pdf(&self, _position: glam::Vec3A, wi: glam::Vec3A) -> (Color, f32, f32) {
        let theta = wi.y.acos();
        let phi = wi.x.atan2(wi.z) + std::f32::consts::PI;

        self.strength_dist_pdf(theta, phi)
    }

    fn is_delta(&self) -> bool {
        false
    }

    fn power(&self) -> f32 {
        self.avg_power * 4.0 * std::f32::consts::PI
    }
}
