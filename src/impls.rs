use crate::generator::TextureGenerator;
use crate::{HashMap, PNG_EXT, SIZE, Texture, rgb_iter};
use image::*;

pub struct SingleTexture<'a> {
    resolution: usize,
    texture_name: &'a str,
}

impl<'a> SingleTexture<'a> {
    pub fn new(texture_name: &'a str, resolution: usize) -> Self {
        let resolution = resolution.next_power_of_two().clamp(2, 16);
        Self {
            resolution,
            texture_name,
        }
    }
}

impl<'a> TextureGenerator for SingleTexture<'a> {
    fn compute_texture_avg_map(&self, textures: &[Texture]) -> HashMap<Rgba<u8>, RgbaImage> {
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
            .find(|t| t.path.ends_with(&lookup))
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
                let rgba_src = Rgba::from([r, g, b, u8::MAX]);
                let mut new_block = RgbaImage::new(width, height);
                for (x, y, pixel) in old_block.enumerate_pixels() {
                    let luminance = pixel.to_luma_alpha()[0] as f64 / (u8::MAX as f64);
                    let mut new_pixel = rgba_src.clone();
                    for i in 0..(new_pixel.0.len() - 1) {
                        let new_val = new_pixel[i] as f64 * luminance;
                        new_pixel[i] = new_val.round() as u8;
                    }
                    new_block.put_pixel(x, y, new_pixel);
                }
                (rgba_src, new_block)
            })
            .collect()
    }
}
