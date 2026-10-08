pub use pixelblocks::PixelBlocks;
pub use xbrz::Xbrz;

use crate::{
    FileData, OUTPUT_DIR, PACK_MCMETA, SIZE, TextureData, WoolError, ZIP_EXT,
    client::ClientFetcher, kdmap::KdMap,
};
use camino::Utf8PathBuf;
use image::{Pixel, Rgba, RgbaImage};
use rayon::prelude::*;
use std::{fs, io::Write};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

mod pixelblocks;
mod xbrz;

/// Massa trait for generating textures.
pub trait TextureGenerator {
    /// She fetch my client until I generate.
    fn generate(self, client_fetcher: ClientFetcher, write_dir: bool) -> Result<(), WoolError>;
}

impl<T> TextureGenerator for T
where
    T: InternalGenerator,
{
    fn generate(self, client_fetcher: ClientFetcher, write_dir: bool) -> Result<(), WoolError> {
        let (old_textures, pack_version) = client_fetcher.fetch()?;

        let new_textures = self.modify_textures(old_textures);

        // This is kinda aids, but it works :).
        let pack_mcmeta = format!(
            "\
{{
  \"pack\": {{
    \"description\": \"\\u00A7k{}.zip\",
    \"min_format\": {},
    \"max_format\": {}
  }}
}}\n",
            self.generator_name(),
            pack_version,
            pack_version
        )
        .into_bytes();

        if fs::exists(OUTPUT_DIR)? {
            fs::remove_dir_all(OUTPUT_DIR)?;
        }
        fs::create_dir(OUTPUT_DIR)?;

        let mut path = Utf8PathBuf::from(OUTPUT_DIR);
        path.push(self.generator_name());

        self.zip(path, new_textures, &pack_mcmeta, write_dir)?;

        Ok(())
    }
}

trait InternalGenerator {
    fn generator_name(&self) -> &'static str;

    fn modify_textures(&self, textures: Box<[TextureData]>) -> Box<[TextureData]>;

    fn zip(
        &self,
        path: Utf8PathBuf,
        textures: Box<[TextureData]>,
        pack_mcmeta: &[u8],
        write_dir: bool,
    ) -> Result<(), WoolError> {
        if write_dir {
            fs::create_dir(&path)?;
            let mut pack_mcmeta_path = path.clone();
            pack_mcmeta_path.push(PACK_MCMETA);
            fs::write(pack_mcmeta_path, pack_mcmeta)?;
        }

        let mut path_zip = path.clone();
        path_zip.add_extension(ZIP_EXT);

        let inner = fs::File::create(path_zip)?;
        let mut zip = ZipWriter::new(inner);
        // Not bothering with compression because all PNGs are already encoded using the
        // highest level. Compressing the final zip only gives back a few MB.
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        zip.start_file(PACK_MCMETA, options)?;
        zip.write_all(pack_mcmeta)?;

        for texture_data in textures {
            let buf = texture_data.file.data()?;

            if write_dir {
                let mut file_path = path.clone();
                file_path.push(&texture_data.path);
                if let Some(parent_path) = file_path.parent() {
                    fs::create_dir_all(parent_path)?;
                }

                fs::write(file_path, &buf)?;
            }

            zip.start_file_from_path(&texture_data.path, options)?;

            zip.write_all(&buf)?;
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
                }
            })
            .collect()
    }
}
