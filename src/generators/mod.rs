mod all_textures;
mod single_texture;
mod xbrz;

use crate::client::ClientJar;
use crate::types::{FileData, KdMap, TextureData};
use crate::{PACK_MCMETA, SIZE, Version};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{Pixel, Rgba, RgbaImage};
use rayon::prelude::*;
use std::fs;
use std::io::Write;
use std::path;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

pub use all_textures::AllTextures;
pub use single_texture::SingleTexture;
pub use xbrz::Xbrz;

/// The massa trait for generating textures, which will be exposed
/// to the end-user through a CLI interface in main.
pub trait TextureGenerator {
    fn generate(self, path: &str, version_id: Version, write_dir: bool, write_jar: bool);
}

impl<T: InternalGenerator> TextureGenerator for T {
    fn generate(self, path: &str, version_id: Version, write_dir: bool, write_jar: bool) {
        let path = path.to_string() + Self::GENERATOR_NAME + "/";
        fs::create_dir_all(&path).unwrap();
        let client_jar = ClientJar::new(version_id);
        if write_jar {
            client_jar.write(path.clone());
        }
        let (old_textures, version) = client_jar.parse();
        // This shit is aids. FUCK.
        let pack_mcmeta = format!(
            "\
{{
  \"pack\": {{
    \"description\": \"TRULY THE GREATEST TEXTURE PACK OF ALL TIME!!!\",
    \"pack_format\": {}
  }}
}}\n",
            version
        )
        .into_bytes();
        let new_textures = self.modify_textures(old_textures);
        if write_dir {
            let path = path.clone() + "pack_output/";
            fs::create_dir_all(&path).unwrap();
            self.write(path, &new_textures, &pack_mcmeta);
        }
        let zip_name = path + Self::GENERATOR_NAME + ".zip";
        self.zip(zip_name, &new_textures, &pack_mcmeta);
    }
}

trait InternalGenerator {
    const GENERATOR_NAME: &str;

    fn modify_textures(&self, textures: Vec<TextureData>) -> Vec<TextureData>;

    fn zip(&self, zip_name: String, textures: &[TextureData], pack_mcmeta: &[u8]) {
        let inner = fs::File::create(zip_name).unwrap();
        let mut zip = ZipWriter::new(inner);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        zip.start_file(PACK_MCMETA, options).unwrap();
        zip.write_all(pack_mcmeta).unwrap();
        textures.into_iter().for_each(|texture_data| {
            zip.start_file(&texture_data.path, options).unwrap();
            let buf = match &texture_data.file {
                FileData::Texture(texture) => {
                    let mut buf = Vec::with_capacity(texture.len());
                    let enc = PngEncoder::new_with_quality(
                        &mut buf,
                        CompressionType::Best,
                        FilterType::Adaptive,
                    );
                    texture.write_with_encoder(enc).unwrap();
                    buf
                }
                FileData::McMeta(data) => data.clone().into_vec(),
            };
            zip.write_all(&buf).unwrap();
        });
        zip.finish().unwrap();
    }

    fn write(&self, output_path: String, textures: &[TextureData], pack_mcmeta: &[u8]) {
        fs::write(output_path.to_string() + PACK_MCMETA, pack_mcmeta).unwrap();
        textures.into_par_iter().for_each(|texture_data| {
            let path = output_path.clone() + texture_data.path.as_str();
            let tmp = path::Path::new(&path).parent().unwrap();
            fs::create_dir_all(tmp).unwrap();
            match &texture_data.file {
                FileData::Texture(texture) => {
                    let f = fs::File::create(path).unwrap();
                    let enc = PngEncoder::new_with_quality(
                        f,
                        CompressionType::Best,
                        FilterType::Adaptive,
                    );
                    texture.write_with_encoder(enc).unwrap();
                }
                FileData::McMeta(data) => fs::write(path, data).unwrap(),
            };
        });
    }
}

trait MappingGenerator {
    fn create_rgb_map(&self, textures: &[TextureData]) -> KdMap;

    fn map(&self, textures: Vec<TextureData>, map: KdMap) -> Vec<TextureData> {
        textures
            .into_par_iter()
            .map(TextureData::extract)
            .map(|(file, path)| {
                match file {
                    FileData::Texture(texture) => {
                        let old_width = texture.width();
                        let old_height = texture.height();
                        let new_width = old_width * SIZE;
                        let new_height = old_height * SIZE;
                        let mut new_image = RgbaImage::new(new_width, new_height);
                        for (x, y, old_pixel) in texture.enumerate_pixels() {
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
                        let file = FileData::Texture(new_image);
                        TextureData { file, path }
                    }
                    FileData::McMeta(_) => TextureData { file, path },
                }
            })
            .collect()
    }
}
