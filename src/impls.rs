use crate::generator::TextureGenerator;
use crate::{KdMap, PNG_EXT, SIZE, Texture};
use image::{Rgba, RgbaImage};

pub struct AllTextures;

impl TextureGenerator for AllTextures {
    fn compute_texture_avg_map(&self, textures: &[Texture]) -> KdMap {
        textures
            .into_iter()
            .filter(|texture| texture.img.width() == SIZE && texture.img.height() == SIZE)
            .filter(|texture| {
                let s = texture.path.as_str();
                // Some shitty textures I don't want being used.
                !s.contains("debug") && !s.contains("jigsaw") && !s.contains("test")
            })
            .filter_map(|texture| {
                calculate_average(&texture.img).map(|avg| (avg, texture.img.clone()))
            })
            .into()
    }
}

fn calculate_average(texture: &RgbaImage) -> Option<[f64; 3]> {
    let pixel_count = (texture.width() * texture.height()) as f64;
    let mut r_sum = 0.0;
    let mut g_sum = 0.0;
    let mut b_sum = 0.0;
    for pixel in texture.pixels() {
        // Immediately terminate on transparent pixel.
        match pixel[3] != 0 {
            true => {
                r_sum += pixel[0] as f64;
                g_sum += pixel[1] as f64;
                b_sum += pixel[2] as f64;
            }
            false => return None,
        }
    }
    Some([
        r_sum / pixel_count,
        g_sum / pixel_count,
        b_sum / pixel_count,
    ])
}

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
            self.texture_name,
            width,
            height
        );
        rgb_iter(self.resolution)
            .map(|(r, g, b)| {
                let mut new_block = RgbaImage::new(SIZE, SIZE);
                for (x, y, old_pixel) in old_block.enumerate_pixels() {
                    let luminance = calculate_luminance(old_pixel);
                    let new_pixel = recolor_pixel(luminance, r, g, b);
                    new_block.put_pixel(x, y, new_pixel);
                }
                ([r as f64, g as f64, b as f64], new_block)
            })
            .into()
    }
}

#[inline]
fn rgb_iter(step: usize) -> impl Iterator<Item = (u8, u8, u8)> {
    (0..=u8::MAX)
        .rev()
        .step_by(step)
        .flat_map(move |r| (0..=u8::MAX).rev().step_by(step).map(move |g| (r, g)))
        .flat_map(move |(r, g)| (0..=u8::MAX).rev().step_by(step).map(move |b| (r, g, b)))
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

fn recolor_pixel(luminance: f64, r: u8, g: u8, b: u8) -> Rgba<u8> {
    let r = (r as f64 * luminance).round().clamp(U8_MIN_F64, U8_MAX_F64) as u8;
    let g = (g as f64 * luminance).round().clamp(U8_MIN_F64, U8_MAX_F64) as u8;
    let b = (b as f64 * luminance).round().clamp(U8_MIN_F64, U8_MAX_F64) as u8;
    // The alpha channel is always max because opacity should be deteremined
    // exclusively by the block whose texture is being replaced.
    Rgba::from([r, g, b, u8::MAX])
}
