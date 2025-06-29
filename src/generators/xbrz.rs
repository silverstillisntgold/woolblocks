use super::{InternalGenerator, UpscalingGenerator};
use crate::Texture;
use image::RgbaImage;
use rayon::prelude::*;
use xbrz::scale_rgba;

const FACTOR: usize = 4;

pub struct Xbrz;

impl InternalGenerator for Xbrz {
    fn modify_textures(&self, textures: Vec<Texture>) -> Vec<Texture> {
        let tmp = self.upscale(textures);
        self.upscale(tmp)
        //self.upscale(textures)
    }
}

impl UpscalingGenerator for Xbrz {
    fn upscale(&self, textures: Vec<Texture>) -> Vec<Texture> {
        textures
            .into_par_iter()
            .map(|texture| {
                let old_width = texture.img.width() as usize;
                let old_height = texture.img.height() as usize;
                let new_width = old_width * FACTOR;
                let new_height = old_height * FACTOR;
                let buf = scale_rgba(texture.img.as_raw(), old_width, old_height, FACTOR);
                let new_image =
                    RgbaImage::from_raw(new_width as u32, new_height as u32, buf).unwrap();
                Texture {
                    img: new_image,
                    path: texture.path,
                }
            })
            .collect()
    }
}
