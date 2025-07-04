use super::InternalGenerator;
use crate::INCLUSIONS;
use crate::types::{FileData, TextureData};
use image::{Rgba, RgbaImage};
use rayon::prelude::*;

const SCALE_FACTOR: usize = 4;

pub struct Xbrz;

impl InternalGenerator for Xbrz {
    const GENERATOR_NAME: &str = "xBRZ";

    fn modify_textures(&self, textures: Vec<TextureData>) -> Vec<TextureData> {
        textures
            .into_par_iter()
            .map(TextureData::extract)
            .map(|(file, path)| match file {
                FileData::Texture(texture) => {
                    let old_width = texture.width() as usize;
                    let old_height = texture.height() as usize;
                    let new_width = old_width * SCALE_FACTOR;
                    let new_height = old_height * SCALE_FACTOR;
                    let buf =
                        xbrz::scale_rgba(texture.as_raw(), old_width, old_height, SCALE_FACTOR);
                    let mut new_image =
                        RgbaImage::from_raw(new_width as u32, new_height as u32, buf).unwrap();
                    // When the texture is an item or effect, remove all pixels
                    // which aren't fully opaque.
                    // The xBrz upscaling algorithm occasionally leaves
                    // pixels semi-transparent when upscaling textures whose pixel-space
                    // isn't fully occupied. This looks weird on items and effects.
                    if path.contains(INCLUSIONS[2]) || path.contains(INCLUSIONS[3]) {
                        for pixel in new_image.pixels_mut() {
                            if pixel[3] != u8::MAX {
                                *pixel = Rgba::from([0, 0, 0, 0]);
                            }
                        }
                    }
                    let file = FileData::Texture(new_image);
                    TextureData { file, path }
                }
                FileData::McMeta(_) => TextureData { file, path },
            })
            .collect()
    }
}
