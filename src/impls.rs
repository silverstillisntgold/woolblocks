use crate::generator::TextureGenerator;
use crate::{KdMap, PNG_EXT, SIZE, Texture, rgb_iter};
use image::*;

/*pub struct AllTextures {
    resolution: usize,
}

impl AllTextures {
    pub fn new(resolution: usize) -> Self {
        Self { resolution }
    }
}

impl TextureGenerator for AllTextures {
    fn compute_texture_avg_map(&self, textures: &[Texture]) -> KdMap {
        todo!()
    }
}
*/

pub struct SingleTexture<'a> {
    resolution: usize,
    texture_name: &'a str,
}

impl<'a> SingleTexture<'a> {
    pub fn new(texture_name: &'a str, resolution: usize) -> Self {
        let resolution = resolution.clamp(1, 8);
        Self {
            resolution,
            texture_name,
        }
    }
}

impl<'a> TextureGenerator for SingleTexture<'a> {
    fn compute_texture_avg_map(&self, textures: &[Texture]) -> KdMap {
        let lookup = {
            let tmp = self.texture_name.to_string();
            if !tmp.ends_with(PNG_EXT) {
                tmp + PNG_EXT
            } else {
                tmp
            }
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
                for (x, y, old_pixel) in old_block.enumerate_pixels() {
                    let luminance = calculate_luminance(old_pixel);
                    // The alpha channel is always maxed out because opacity should be deteremined
                    // exclusively by the block whose texture is being replaced.
                    let new_pixel = recolor_pixel(luminance, r, g, b, u8::MAX);
                    new_block.put_pixel(x, y, new_pixel);
                }
                ([r as f64, g as f64, b as f64], new_block)
            })
            .into()
    }
}

const U8_MAX_F64: f64 = u8::MAX as f64;
const U8_MIN_F64: f64 = u8::MIN as f64;

fn calculate_luminance(pixel: &Rgba<u8>) -> f64 {
    let r = pixel[0] as f64 / U8_MAX_F64;
    let g = pixel[1] as f64 / U8_MAX_F64;
    let b = pixel[2] as f64 / U8_MAX_F64;
    // Rec. 709
    (0.2126 * r) + (0.7152 * g) + (0.0722 * b)
}

fn recolor_pixel(luminance: f64, r: u8, g: u8, b: u8, a: u8) -> Rgba<u8> {
    let r = (r as f64 * luminance).round().clamp(U8_MIN_F64, U8_MAX_F64) as u8;
    let g = (g as f64 * luminance).round().clamp(U8_MIN_F64, U8_MAX_F64) as u8;
    let b = (b as f64 * luminance).round().clamp(U8_MIN_F64, U8_MAX_F64) as u8;
    Rgba::from([r, g, b, a])
}
