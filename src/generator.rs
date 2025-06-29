use crate::{ClientJar, KdMap, SIZE, Texture};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{Pixel, Rgba, RgbaImage};
use rayon::prelude::*;
use std::fs;

#[allow(unused)]
pub trait TextureGeneratorV2 {
    fn generate(&self, dst_zip_name: &str, textures: Vec<Texture>, version: u64);
}

pub trait TextureGenerator {
    fn compute_texture_avg_map(&self, textures: &[Texture]) -> KdMap;

    fn run(&self, zip_name: &str, client_jar: ClientJar) {
        let (textures, version) = client_jar.parse();
        let map = self.compute_texture_avg_map(&textures);
        let textures = self.into_writable(textures, map);
        self.write(zip_name, textures, version);
    }

    fn into_writable(&self, textures: Vec<Texture>, map: KdMap) -> Vec<Texture> {
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

    fn write(&self, zip_name: &str, textures: Vec<Texture>, version: u64) {
        let _ = version;
        textures.into_par_iter().for_each(|texture| {
            let path = zip_name.to_string() + texture.path.as_str();
            let tmp = std::path::Path::new(path.as_str()).parent().unwrap();
            fs::create_dir_all(tmp).unwrap();
            let f = fs::File::create(path).unwrap();
            let enc = PngEncoder::new_with_quality(f, CompressionType::Best, FilterType::Adaptive);
            texture.img.write_with_encoder(enc).unwrap();
        });
    }
}
