pub use all_textures::AllTextures;
pub use xbrz::Xbrz;

use crate::{
    FileData, OUTPUT_DIR, PACK_MCMETA, SIZE, TextureData, WoolError, ZIP_EXT,
    client::ClientFetcher, kdmap::KdMap,
};
use camino::{Utf8Path, Utf8PathBuf};
use image::{Pixel, Rgba, RgbaImage};
use rayon::prelude::*;
use std::{fs, io::Write};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

mod all_textures;
mod xbrz;

/// Massa trait for generating textures.
pub trait TextureGenerator {
    fn generate(self, client_fetcher: ClientFetcher, write_dir: bool) -> Result<(), WoolError>;
}

impl<T> TextureGenerator for T
where
    T: InternalGenerator,
{
    fn generate(self, client_fetcher: ClientFetcher, write_dir: bool) -> Result<(), WoolError> {
        let (old_textures, pack_version) = client_fetcher.fetch()?;
        let mut new_textures = self.modify_textures(old_textures);
        // Encoding always comes **after** modification.
        new_textures
            .par_iter_mut()
            .try_for_each(|texture| texture.file.encode_textures())?;

        // This is kinda aids.
        let pack_mcmeta = format!(
            "\
{{
  \"pack\": {{
    \"description\": \"{} is goated\",
    \"min_format\": {},
    \"max_format\": {}
  }}
}}\n",
            self.generator_name(),
            pack_version,
            pack_version
        )
        .into_bytes();

        let mut path = Utf8PathBuf::from(OUTPUT_DIR);
        path.push(self.generator_name());

        if fs::exists(OUTPUT_DIR)? {
            fs::remove_dir_all(OUTPUT_DIR)?;
        }
        fs::create_dir(OUTPUT_DIR)?;

        if write_dir {
            fs::create_dir(&path)?;
            self.write(&path, &new_textures, &pack_mcmeta)?;
        }

        path.add_extension(ZIP_EXT);
        self.zip(&path, &new_textures, &pack_mcmeta)?;

        Ok(())
    }
}

trait InternalGenerator {
    fn generator_name(&self) -> &'static str;

    fn modify_textures(&self, textures: Box<[TextureData]>) -> Box<[TextureData]>;

    fn write(
        &self,
        path: &Utf8Path,
        textures: &[TextureData],
        pack_mcmeta: &[u8],
    ) -> Result<(), WoolError> {
        let mut pack_mcmeta_path = path.to_path_buf();
        pack_mcmeta_path.push(PACK_MCMETA);
        fs::write(pack_mcmeta_path, pack_mcmeta)?;

        for texture_data in textures {
            let mut file_path = path.to_path_buf();
            file_path.push(&texture_data.path);
            if let Some(parent_path) = file_path.parent() {
                fs::create_dir_all(parent_path)?;
            }

            let buf = texture_data.file.data();

            fs::write(file_path, buf)?;
        }

        Ok(())
    }

    fn zip(
        &self,
        path: &Utf8Path,
        textures: &[TextureData],
        pack_mcmeta: &[u8],
    ) -> Result<(), WoolError> {
        let inner = fs::File::create(path)?;
        let mut zip = ZipWriter::new(inner);
        // Not bothering with compression because all PNGs are already encoded using the
        // highest level. Compressing the final zip only gives back a few MB.
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        zip.start_file(PACK_MCMETA, options)?;
        zip.write_all(pack_mcmeta)?;

        for texture_data in textures {
            zip.start_file_from_path(&texture_data.path, options)?;

            let buf = texture_data.file.data();

            zip.write_all(buf)?;
        }

        // Explicitly finish the zip to avoid silent errors when dropping.
        zip.finish()?;

        Ok(())
    }
}

trait MappingGenerator {
    fn create_rgb_map(&self, textures: &[TextureData]) -> KdMap;

    fn map(&self, map: KdMap, textures: Box<[TextureData]>) -> Box<[TextureData]> {
        textures
            .into_par_iter()
            .map(|texture_data| {
                let TextureData { file, path } = texture_data;

                match file {
                    FileData::Texture(texture) => {
                        let old_width = texture.width();
                        let old_height = texture.height();
                        let new_width = old_width * SIZE;
                        let new_height = old_height * SIZE;
                        let mut new_texture = RgbaImage::new(new_width, new_height);

                        for (x, y, old_pixel) in texture.enumerate_pixels() {
                            // Find the whole texture whose approximate average color is closest to the current pixel.
                            let query = old_pixel.to_rgb().0.map(f64::from);
                            let closest_block = map.find_most_similar(&query);

                            let offset_x = x * SIZE;
                            let offset_y = y * SIZE;
                            for (d_x, d_y, closest_pixel) in closest_block.enumerate_pixels() {
                                let x = offset_x + d_x;
                                let y = offset_y + d_y;

                                let pixel = if old_pixel[3] != 0 {
                                    let mut pixel = *closest_pixel;
                                    pixel[3] = old_pixel[3];
                                    pixel
                                } else {
                                    Rgba::from([0, 0, 0, 0])
                                };

                                new_texture.put_pixel(x, y, pixel);
                            }
                        }

                        let file = FileData::Texture(new_texture);

                        TextureData { file, path }
                    }

                    FileData::McMeta(_) => TextureData { file, path },

                    _ => unreachable!("no textures should have been encoded"),
                }
            })
            .collect()
    }
}
