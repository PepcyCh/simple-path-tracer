use crate::core::color::Color;
use cgmath::{ElementWise, InnerSpace};
use image::GenericImageView;

pub fn generate_mipmap(image: image::DynamicImage) -> Vec<image::DynamicImage> {
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

pub fn sample_blinear(image: &image::DynamicImage, u: f32, v: f32) -> Color {
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

    let c00: Color = image.get_pixel(x0, y0).into();
    let c01: Color = image.get_pixel(x0, y1).into();
    let c10: Color = image.get_pixel(x1, y0).into();
    let c11: Color = image.get_pixel(x1, y1).into();

    let c0 = c00 * (1.0 - yt) + c01 * yt;
    let c1 = c10 * (1.0 - yt) + c11 * yt;
    c0 * (1.0 - xt) + c1 * xt
}

pub fn sample_trilinear(
    images: &Vec<image::DynamicImage>,
    u: f32,
    v: f32,
    duvdx: cgmath::Vector2<f32>,
    duvdy: cgmath::Vector2<f32>,
) -> Color {
    if images.is_empty() {
        return Color::BLACK;
    }

    let scale = cgmath::Vector2::new(images[0].width() as f32, images[0].height() as f32);
    let duvdx = duvdx.mul_element_wise(scale);
    let duvdy = duvdy.mul_element_wise(scale);

    let level = (duvdx.magnitude().max(duvdy.magnitude()) + 0.001)
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

pub fn wrap_uv(u: f32, v: f32) -> (f32, f32) {
    let u_new = if u >= 0.0 { u.fract() } else { 1.0 + u.fract() };
    let v_new = if v >= 0.0 { v.fract() } else { 1.0 + v.fract() };
    (u_new, v_new)
}
