use image::{Rgb, RgbImage};

use crate::{
    core::color::Color,
    filter::{Filter, FilterT},
};

#[derive(Copy, Clone)]
struct SampleData {
    offset: (f32, f32),
    color: Color,
}

pub struct Film {
    width: u32,
    height: u32,
    data: Vec<Vec<SampleData>>,
}

impl Film {
    pub fn new(width: u32, height: u32) -> Self {
        let data = vec![vec![]; (width * height) as usize];
        Self {
            width,
            height,
            data,
        }
    }

    #[allow(dead_code)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[allow(dead_code)]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn add_sample(&mut self, x: u32, y: u32, offset: (f32, f32), color: Color) {
        let data = SampleData { offset, color };
        let index = self.index_of(x, y);
        self.data[index].push(data);
    }

    pub fn filter_to_image(&self, filter: &Filter) -> RgbImage {
        let mut image: RgbImage = RgbImage::new(self.width, self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                let filtered = self.filter_pixel(x, y, filter);
                image.put_pixel(x, y, color_to_rgb(filtered));
            }
        }
        image
    }

    fn index_of(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    fn filter_pixel(&self, x: u32, y: u32, filter: &Filter) -> Color {
        let radius = filter.radius();

        let mut color = Color::BLACK;
        let mut weight_sum = 0.0;
        for j in -radius..=radius {
            if !(0..self.height as i32).contains(&(y as i32 + j)) {
                continue;
            }
            for i in -radius..=radius {
                if !(0..self.width as i32).contains(&(x as i32 + i)) {
                    continue;
                }
                let index = self.index_of((x as i32 + i) as u32, (y as i32 + j) as u32);
                for sample in &self.data[index] {
                    let weight =
                        filter.weight(i as f32 + sample.offset.0, j as f32 + sample.offset.1);
                    color += sample.color;
                    weight_sum += weight;
                }
            }
        }
        color / weight_sum
    }
}

fn color_to_rgb(color: Color) -> Rgb<u8> {
    let r = (color.r * 255.0).clamp(0.0, 255.0) as u8;
    let g = (color.g * 255.0).clamp(0.0, 255.0) as u8;
    let b = (color.b * 255.0).clamp(0.0, 255.0) as u8;
    Rgb([r, g, b])
}
