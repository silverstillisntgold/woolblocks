use crate::{FileData, INCLUSIONS, SIZE, TextureData, generators::InternalGenerator};
use image::RgbaImage;
use rayon::prelude::*;

const CUTOFF: u8 = u8::MAX / 2;
const SCALE_FACTOR: usize = 4;
const WRAP_BORDER: u32 = 4;

pub struct Xbrz;

impl InternalGenerator for Xbrz {
    fn generator_name(&self) -> &'static str {
        "xBRZ"
    }

    fn modify_textures(&self, textures: Box<[TextureData]>) -> Box<[TextureData]> {
        textures
            .into_par_iter()
            .map(|texture_data| {
                let TextureData { file, path } = texture_data;

                match file {
                    FileData::Texture(texture) => {
                        let is_block = path
                            .components()
                            .any(|component| component.as_str().eq(INCLUSIONS[0]));
                        let mut new_image =
                            if is_block && texture.width() == SIZE && texture.height() == SIZE {
                                scale_wrapped(&texture)
                            } else {
                                scale(&texture)
                            };

                        // The xBRZ upscaling algorithm occasionally leaves pixels semi-transparent
                        // when upscaling textures whose pixel-space isn't fully occupied, which looks
                        // particularly weird on items and effects.
                        // A convenient solution that looks pretty good is to map pixels which are mostly
                        // visible to being fully opaque and pixels which are mostly invisible to being
                        // fully transparent.
                        let fix_fucked_pixels = path.components().any(|component| {
                            let tmp = component.as_str();
                            tmp.eq(INCLUSIONS[2]) || tmp.eq(INCLUSIONS[3])
                        });
                        if fix_fucked_pixels {
                            for pixel in new_image.pixels_mut() {
                                if pixel[3] > CUTOFF {
                                    pixel[3] = u8::MAX;
                                } else {
                                    pixel[3] = u8::MIN
                                }
                            }
                        }

                        let file = FileData::Texture(new_image);

                        TextureData { file, path }
                    }

                    FileData::McMeta(_) => TextureData { file, path },

                    _ => unreachable!("no textures should have been encoded"),
                }
            })
            .collect()
    }
}

fn scale(texture: &RgbaImage) -> RgbaImage {
    let width = texture.width() as usize;
    let height = texture.height() as usize;

    let buf = xbrz::scale_rgba(texture.as_raw(), width, height, SCALE_FACTOR);

    RgbaImage::from_raw(
        (width * SCALE_FACTOR) as u32,
        (height * SCALE_FACTOR) as u32,
        buf,
    )
    .expect("xBRZ output buffer should be correctly sized")
}

fn scale_wrapped(texture: &RgbaImage) -> RgbaImage {
    let padded = wrap_pad(texture);
    let scaled = scale(&padded);

    let scale = SCALE_FACTOR as u32;
    let crop_offset = WRAP_BORDER * scale;

    // Only retain the region corresponding to the original image.
    image::imageops::crop_imm(
        &scaled,
        crop_offset,
        crop_offset,
        texture.width() * scale,
        texture.height() * scale,
    )
    .to_image()
}

fn wrap_pad(texture: &RgbaImage) -> RgbaImage {
    let width = texture.width();
    let height = texture.height();

    // Pretty sure `rem_euclid` is the correct option here.
    // TODO: Make this less messy.
    RgbaImage::from_fn(width + WRAP_BORDER * 2, height + WRAP_BORDER * 2, |x, y| {
        let source_x = (i64::from(x) - i64::from(WRAP_BORDER)).rem_euclid(i64::from(width)) as u32;
        let source_y = (i64::from(y) - i64::from(WRAP_BORDER)).rem_euclid(i64::from(height)) as u32;

        *texture.get_pixel(source_x, source_y)
    })
}
