use crate::{FileData, INCLUSIONS, TextureData, generators::InternalGenerator};
use image::{Rgba, RgbaImage};
use rayon::prelude::*;

const SCALE_FACTOR: usize = 4;

pub struct Xbrz;

impl InternalGenerator for Xbrz {
    fn generator_name(&self) -> &'static str {
        "xBRZ"
    }

    fn modify_textures(&self, textures: Box<[TextureData]>) -> Box<[TextureData]> {
        textures
            .into_par_iter()
            .map(|texture_data| (texture_data.file, texture_data.path))
            .map(|(file, path)| match file {
                FileData::Texture(texture) => {
                    let old_width = texture.width() as usize;
                    let old_height = texture.height() as usize;
                    let new_width = old_width * SCALE_FACTOR;
                    let new_height = old_height * SCALE_FACTOR;

                    let buf =
                        xbrz::scale_rgba(texture.as_raw(), old_width, old_height, SCALE_FACTOR);
                    let mut new_image =
                        RgbaImage::from_raw(new_width as u32, new_height as u32, buf)
                            .expect("buffer should be correctly sized");

                    // When the texture is an item or effect, remove all pixels
                    // which aren't fully opaque.
                    // The xBrz upscaling algorithm occasionally leaves
                    // pixels semi-transparent when upscaling textures whose pixel-space
                    // isn't fully occupied. This looks weird on items and effects.
                    let fix_fucked_pixels = path.components().any(|component| {
                        let s = component.as_str();
                        s.eq(INCLUSIONS[2]) || s.eq(INCLUSIONS[3])
                    });
                    if fix_fucked_pixels {
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

                _ => unreachable!("no textures should have been encoded"),
            })
            .collect()
    }
}
