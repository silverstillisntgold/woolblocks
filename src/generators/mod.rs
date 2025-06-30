mod all_textures;
mod single_texture;
mod xbrz;

use crate::client::ClientJar;
use crate::types::{KdMap, Texture, Version};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{Pixel, Rgba, RgbaImage};
use rayon::prelude::*;
use std::fs;
use std::io::Write;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

pub use all_textures::AllTextures;
pub use single_texture::SingleTexture;
pub use xbrz::Xbrz;

const PACK_MCMETA: &str = "pack.mcmeta";
const SIZE: u32 = 16;

/// The massa trait for generating textures, which will be exposed
/// to the end-user through a CLI interface in main.
pub trait TextureGenerator {
    fn generate(self, zip_name: &str, version_id: Version, write_dir: bool);
}

impl<T: InternalGenerator> TextureGenerator for T {
    fn generate(self, dst_name: &str, version_id: Version, write_dir: bool) {
        let (old_textures, version) = ClientJar::new(version_id).parse();
        let pack_mcmeta = format!(
            "\
{{
  \"pack\": {{
    \"description\": \"TRULY THE GREATEST TEXTURE PACK OF ALL TIME!!!\",
    \"pack_format\": {}
  }}
}}\n",
            version
        );
        let new_textures = self.modify_textures(old_textures);
        if write_dir {
            let dir_name = dst_name.to_string() + "/";
            self.write(&dir_name, &new_textures, &pack_mcmeta);
        }
        let zip_name = dst_name.to_string() + ".zip";
        self.zip(&zip_name, new_textures, &pack_mcmeta);
    }
}

trait InternalGenerator {
    fn modify_textures(&self, textures: Vec<Texture>) -> Vec<Texture>;

    fn zip(&self, zip_name: &str, textures: Vec<Texture>, pack_mcmeta: &str) {
        let inner = fs::File::create(zip_name).unwrap();
        let mut zip = ZipWriter::new(inner);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        zip.start_file(PACK_MCMETA, options).unwrap();
        zip.write_all(pack_mcmeta.as_bytes()).unwrap();
        textures.into_iter().for_each(|texture| {
            zip.start_file(texture.path, options).unwrap();
            let mut buf = Vec::with_capacity(texture.img.as_raw().len());
            let encoder =
                PngEncoder::new_with_quality(&mut buf, CompressionType::Best, FilterType::Adaptive);
            texture.img.write_with_encoder(encoder).unwrap();
            zip.write_all(&buf).unwrap();
        });
        zip.finish().unwrap();
    }

    fn write(&self, dir_name: &str, textures: &[Texture], pack_mcmeta: &str) {
        fs::create_dir_all(dir_name).unwrap();
        fs::write(dir_name.to_string() + PACK_MCMETA, pack_mcmeta).unwrap();
        textures.into_par_iter().for_each(|texture| {
            let path = dir_name.to_string() + texture.path.as_str();
            let tmp = std::path::Path::new(path.as_str()).parent().unwrap();
            fs::create_dir_all(tmp).unwrap();
            let f = fs::File::create(path).unwrap();
            let enc = PngEncoder::new_with_quality(f, CompressionType::Best, FilterType::Adaptive);
            texture.img.write_with_encoder(enc).unwrap();
        });
    }
}

trait MappingGenerator {
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
