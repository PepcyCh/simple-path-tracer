use crate::core::{color::Color, light::Light, sampler::Sampler};

pub struct EnvLight {
    texture: Vec<Vec<Color>>,
    scale: Color,
    height: usize,
    width: usize,
    atlas_table: AtlasTable,
}

struct AtlasTable {
    props: Vec<f32>,
    u: Vec<f32>,
    k: Vec<usize>,
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
        for prop in &mut props {
            *prop /= sum;
        }

        let atlas_table = AtlasTable::new(props);
        Self {
            texture,
            scale,
            height,
            width,
            atlas_table,
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

        let p00 = self.atlas_table.props[y0 * self.width + x0];
        let p01 = self.atlas_table.props[y1 * self.width + x0];
        let p10 = self.atlas_table.props[y0 * self.width + x1];
        let p11 = self.atlas_table.props[y1 * self.width + x1];
        let p0 = p00 * (1.0 - yt) + p01 * yt;
        let p1 = p10 * (1.0 - yt) + p11 * yt;
        let p = p0 * (1.0 - xt) * p1 * xt;

        (c * self.scale, f32::INFINITY, p)
    }
}

impl Light for EnvLight {
    fn sample(
        &self,
        _position: glam::Vec3A,
        sampler: &mut dyn Sampler,
    ) -> (glam::Vec3A, f32, Color, f32) {
        let rand = sampler.uniform_1d();
        let (ind, _) = self.atlas_table.sample(rand);
        let x = ind % self.width;
        let y = ind / self.width;

        let (rand_x, rand_y) = sampler.uniform_2d();
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
}

impl AtlasTable {
    fn new(props: Vec<f32>) -> Self {
        let n = props.len();
        let mut u: Vec<f32> = props.iter().map(|prop| *prop * n as f32).collect();
        let mut k: Vec<usize> = (0..n).collect();

        let mut poor = u
            .iter()
            .enumerate()
            .find(|(_, val)| **val < 1.0)
            .map(|(ind, _)| ind);
        let mut poor_max = poor;
        let mut rich = u
            .iter()
            .enumerate()
            .find(|(_, val)| **val > 1.0)
            .map(|(ind, _)| ind);

        while poor.is_some() && rich.is_some() {
            let poor_ind = poor.unwrap();
            let rich_ind = rich.unwrap();

            let diff = 1.0 - u[poor_ind];
            u[rich_ind] -= diff;
            k[poor_ind] = rich_ind;

            if u[rich_ind] < 1.0 && rich_ind < poor_max.unwrap() {
                poor = Some(rich_ind);
            } else {
                poor = None;
                for i in poor_max.unwrap() + 1..u.len() {
                    if u[i] < 1.0 {
                        poor = Some(i);
                        poor_max = Some(i);
                        break;
                    }
                }
            }

            rich = None;
            for i in rich_ind..u.len() {
                if u[i] > 1.0 {
                    rich = Some(i);
                    break;
                }
            }
        }

        Self { props, u, k }
    }

    fn sample(&self, rand: f32) -> (usize, f32) {
        let temp = rand * self.props.len() as f32;
        let x = temp as usize;
        let y = temp - x as f32;
        if y < self.u[x] {
            (x, self.props[x])
        } else {
            (self.k[x], self.props[self.k[x]])
        }
    }
}
