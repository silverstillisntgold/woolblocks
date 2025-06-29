mod all_textures;
mod single_texture;
mod xbrz;

use crate::{ClientJar, KdMap, SIZE, Texture};
use image::{Pixel, Rgba, RgbaImage};
use rayon::prelude::*;

pub use all_textures::AllTextures;
pub use single_texture::SingleTexture;

/// The massa trait for generating textures, which will be exposed
/// to the end-user through a CLI interface in main.
pub trait TextureGenerator {
    fn generate(self, dst_zip_name: &str, client_jar: ClientJar, write_dir: bool);
}

impl<T: InternalGenerator> TextureGenerator for T {
    fn generate(self, dst_zip_name: &str, client_jar: ClientJar, write_dir: bool) {
        let (old_textures, version) = client_jar.parse();
        let new_textures = self.modify_textures(old_textures);
        if write_dir {
            self.write(&new_textures);
        }
        self.zip(new_textures);
    }
}

pub trait InternalGenerator {
    fn modify_textures(&self, textures: Vec<Texture>) -> Vec<Texture>;

    fn zip(&self, textures: Vec<Texture>) {
        todo!()
    }

    fn write(&self, textures: &[Texture]) {
        todo!()
    }
}

pub trait UpscalingGenerator {
    fn upscale(&self, textures: Vec<Texture>) -> Vec<Texture>;
}

pub trait MappingGenerator {
    fn create_rgb_map(&self, textures: &[Texture]) -> KdMap;

    fn map(&self, textures: Vec<Texture>, map: KdMap) -> Vec<Texture> {
        textures
            .into_par_iter()
            .map(|texture| {
                let old_width = texture.img.width();
                let old_height = texture.img.height();
                let new_width = old_width * SIZE;
                let new_height = old_height * SIZE;
                let mut new_image = RgbaImage::new(new_width, new_height);
                for (x, y, old_pixel) in texture.img.enumerate_pixels() {
                    let query = old_pixel.to_rgb().0.map(f64::from);
                    // Find the whole texture whose approximate average color
                    // is closest to the current pixel.
                    let closest_block = map.nearest(&query);
                    let offset_x = x * SIZE;
                    let offset_y = y * SIZE;
                    for (d_x, d_y, closest_pixel) in closest_block.enumerate_pixels() {
                        match old_pixel[3] != 0 {
                            true => {
                                let mut pixel = closest_pixel.clone();
                                pixel[3] = old_pixel[3];
                                new_image.put_pixel(offset_x + d_x, offset_y + d_y, pixel);
                            }
                            false => {
                                new_image.put_pixel(
                                    offset_x + d_x,
                                    offset_y + d_y,
                                    Rgba::from([0, 0, 0, 0]),
                                );
                            }
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
