use image::GenericImageView;

use crate::core::{color::Color, loader::InputParams, scene_resources::SceneResources};

use super::{TextureChannel, TextureInput, TextureT};

pub struct ImageTex {
    images: Vec<image::DynamicImage>,
}

impl ImageTex {
    pub fn new(image: image::DynamicImage) -> Self {
        let images = generate_mipmap(image);
        Self { images }
    }

    pub fn load(_rsc: &SceneResources, params: &mut InputParams) -> anyhow::Result<Self> {
        let image = params.get_image("image_file")?;

        Ok(Self::new(image))
    }
}

impl TextureT for ImageTex {
    fn color_at(&self, input: TextureInput) -> Color {
        let uv = input.value_vec2_wrapped();
        let value = sample_trilinear(&self.images, uv.x, uv.y, input.duvdx, input.duvdy);
        Color::new(value.x, value.y, value.z)
    }

    fn float_at(&self, input: TextureInput, chan: TextureChannel) -> f32 {
        let uv = input.value_vec2_wrapped();
        let value = sample_trilinear(&self.images, uv.x, uv.y, input.duvdx, input.duvdy);
        match chan {
            TextureChannel::R => value.x,
            TextureChannel::G => value.y,
            TextureChannel::B => value.z,
            TextureChannel::A => value.w,
        }
    }

    fn average_color(&self) -> Color {
        let value = rgba_to_vec4(self.images.last().unwrap().get_pixel(0, 0));
        Color::new(value.x, value.y, value.z)
    }

    fn average_float(&self, chan: TextureChannel) -> f32 {
        let value = rgba_to_vec4(self.images.last().unwrap().get_pixel(0, 0));
        match chan {
            TextureChannel::R => value.x,
            TextureChannel::G => value.y,
            TextureChannel::B => value.z,
            TextureChannel::A => value.w,
        }
    }

    fn dimensions(&self) -> Option<(u32, u32, u32)> {
        let (w, h) = self.images[0].dimensions();
        Some((w, h, 1))
    }
}

fn generate_mipmap(image: image::DynamicImage) -> Vec<image::DynamicImage> {
    let mut width = image.width();
    let mut height = image.height();
    let mut images = vec![image];

    while width > 1 || height > 1 {
        width = (width + 1) >> 1;
        height = (height + 1) >> 1;
        let mut image = image::ImageBuffer::new(width, height);

        let last_image = images.last().unwrap();
        for i in 0..width {
            for j in 0..height {
                let x0 = 2 * i;
                let x1 = (2 * i + 1).min(last_image.width() - 1);
                let y0 = 2 * j;
                let y1 = (2 * j + 1).min(last_image.height() - 1);

                let p0 = last_image.get_pixel(x0, y0).0;
                let p1 = last_image.get_pixel(x0, y1).0;
                let p2 = last_image.get_pixel(x1, y0).0;
                let p3 = last_image.get_pixel(x1, y1).0;
                let p = [
                    ((p0[0] as f32 + p1[0] as f32 + p2[0] as f32 + p3[0] as f32) * 0.25) as u8,
                    ((p0[1] as f32 + p1[1] as f32 + p2[1] as f32 + p3[1] as f32) * 0.25) as u8,
                    ((p0[2] as f32 + p1[2] as f32 + p2[2] as f32 + p3[2] as f32) * 0.25) as u8,
                    ((p0[3] as f32 + p1[3] as f32 + p2[3] as f32 + p3[3] as f32) * 0.25) as u8,
                ];
                image.put_pixel(i, j, image::Rgba(p));
            }
        }
        images.push(image::DynamicImage::ImageRgba8(image));
    }

    images
}

fn sample_blinear(image: &image::DynamicImage, u: f32, v: f32) -> glam::Vec4 {
    let x = u * image.width() as f32;
    let x1 = x.round() as i32;
    let x0 = x1 - 1;
    let xt = x - x0 as f32 - 0.5;
    let x0 = x0.clamp(0, image.width() as i32 - 1) as u32;
    let x1 = x1.clamp(0, image.width() as i32 - 1) as u32;

    let y = v * image.height() as f32;
    let y1 = y.round() as i32;
    let y0 = y1 - 1;
    let yt = y - y0 as f32 - 0.5;
    let y0 = y0.clamp(0, image.height() as i32 - 1) as u32;
    let y1 = y1.clamp(0, image.height() as i32 - 1) as u32;

    let c00 = rgba_to_vec4(image.get_pixel(x0, y0));
    let c01 = rgba_to_vec4(image.get_pixel(x0, y1));
    let c10 = rgba_to_vec4(image.get_pixel(x1, y0));
    let c11 = rgba_to_vec4(image.get_pixel(x1, y1));

    let c0 = c00 * (1.0 - yt) + c01 * yt;
    let c1 = c10 * (1.0 - yt) + c11 * yt;
    c0 * (1.0 - xt) + c1 * xt
}

fn sample_trilinear(
    images: &Vec<image::DynamicImage>,
    u: f32,
    v: f32,
    duvdx: glam::Vec2,
    duvdy: glam::Vec2,
) -> glam::Vec4 {
    if images.is_empty() {
        return glam::Vec4::ZERO;
    }

    let scale = glam::Vec2::new(images[0].width() as f32, images[0].height() as f32);
    let duvdx = duvdx * scale;
    let duvdy = duvdy * scale;

    let level = (duvdx.length().max(duvdy.length()) + 0.001)
        .log2()
        .clamp(0.0, (images.len() - 1) as f32);
    let l0 = level.floor() as usize;
    if l0 + 1 == images.len() {
        sample_blinear(&images[l0], u, v)
    } else {
        let l1 = l0 + 1;
        let lt = level - l0 as f32;
        let c0 = sample_blinear(&images[l0], u, v);
        let c1 = sample_blinear(&images[l1], u, v);
        c0 * (1.0 - lt) + c1 * lt
    }
}

fn rgba_to_vec4(rgba: image::Rgba<u8>) -> glam::Vec4 {
    glam::Vec4::new(
        rgba.0[0] as f32 / 255.0,
        rgba.0[1] as f32 / 255.0,
        rgba.0[2] as f32 / 255.0,
        rgba.0[3] as f32 / 255.0,
    )
}
