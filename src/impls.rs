use crate::generator::TextureGenerator;
use crate::{KdMap, PNG_EXT, SIZE, Texture, rgb_iter};
use image::*;

pub struct SingleTexture<'a> {
    resolution: usize,
    texture_name: &'a str,
}

impl<'a> SingleTexture<'a> {
    pub fn new(texture_name: &'a str, resolution: usize) -> Self {
        let resolution = resolution.clamp(2, 16);
        Self {
            resolution,
            texture_name,
        }
    }
}

impl<'a> TextureGenerator for SingleTexture<'a> {
    fn compute_texture_avg_map(&self, textures: &[Texture]) -> KdMap {
        let lookup = {
            let file_name = self
                .texture_name
                .split_once('.')
                .map(|(prefix, _)| prefix)
                .unwrap_or(self.texture_name);
            let mut tmp = String::with_capacity(file_name.len() + PNG_EXT.len());
            tmp.push_str(file_name);
            tmp.push_str(PNG_EXT);
            tmp
        };
        let old_block = &textures
            .into_iter()
            .find(|t| t.path.as_str().ends_with(&lookup))
            .unwrap()
            .img;
        let width = old_block.width();
        let height = old_block.height();
        assert!(
            width == SIZE && height == SIZE,
            "the source texture must be {}x{}, '{}' is {}x{}",
            SIZE,
            SIZE,
            lookup,
            width,
            height
        );
        rgb_iter(self.resolution)
            .map(|(r, g, b)| {
                let mut new_block = RgbaImage::new(SIZE, SIZE);
                for (x, y, pixel) in old_block.enumerate_pixels() {
                    // The alpha channel is always maxed out because opacity should be deteremined
                    // exclusively by the block whose texture is being replaced.
                    let new_pixel = recolor_pixel(pixel, r, g, b, u8::MAX);
                    new_block.put_pixel(x, y, new_pixel);
                }
                ([r as f64, g as f64, b as f64], new_block)
            })
            .into()
    }
}

const U8_MAX_F64: f64 = u8::MAX as f64;

fn calculate_luminance(pixel: &Rgba<u8>) -> f64 {
    let r = pixel[0] as f64 / U8_MAX_F64;
    let g = pixel[1] as f64 / U8_MAX_F64;
    let b = pixel[2] as f64 / U8_MAX_F64;
    // Rec. 709
    (0.2126 * r) + (0.7152 * g) + (0.0722 * b)
}

fn recolor_pixel(src_pixel: &Rgba<u8>, r: u8, g: u8, b: u8, a: u8) -> Rgba<u8> {
    let luminance = calculate_luminance(src_pixel);
    let r = (r as f64 * luminance).round().clamp(0.0, U8_MAX_F64) as u8;
    let g = (g as f64 * luminance).round().clamp(0.0, U8_MAX_F64) as u8;
    let b = (b as f64 * luminance).round().clamp(0.0, U8_MAX_F64) as u8;
    Rgba::from([r, g, b, a])
}
