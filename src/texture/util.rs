use crate::core::color::Color;
use image::GenericImageView;

pub fn get_pixel(image: &image::DynamicImage, u: f32, v: f32) -> Color {
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
