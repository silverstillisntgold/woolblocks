use super::InternalGenerator;
use crate::INCLUSIONS;
use crate::types::Texture;
use image::{Rgba, RgbaImage};
use rayon::prelude::*;

const FACTOR: usize = 4;

pub struct Xbrz;

impl InternalGenerator for Xbrz {
    fn modify_textures(&self, textures: Vec<Texture>) -> Vec<Texture> {
        textures
            .into_par_iter()
            .map(|texture| {
                let old_width = texture.img.width() as usize;
                let old_height = texture.img.height() as usize;
                let new_width = old_width * FACTOR;
                let new_height = old_height * FACTOR;
                let buf = xbrz::scale_rgba(texture.img.as_raw(), old_width, old_height, FACTOR);
                let mut new_image =
                    RgbaImage::from_raw(new_width as u32, new_height as u32, buf).unwrap();
                // When the texture is an item, remove all pixels which aren't
                // fully opaque. The xBrz upscaling algorithm occasionally leaves
                // pixels semi-transparent when upscaling textures whose pixel-space
                // isn't fully occupied. This looks weird on items.
                if texture.path.contains(INCLUSIONS[2]) {
                    for pixel in new_image.pixels_mut() {
                        if pixel[3] != u8::MAX {
                            *pixel = Rgba::from([0, 0, 0, 0]);
                        }
                    }
                }
                Texture {
                    img: new_image,
                    path: texture.path,
                }
            })
            .collect()
    }
}
