use crate::{KdMap, PNG_EXT, SIZE, Texture, VERSION_JSON};
use camino::Utf8PathBuf;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ImageFormat, Pixel, Rgba, RgbaImage, load_from_memory_with_format};
use rayon::prelude::*;
use std::fs;
use vfs::{VfsFileType, VfsPath};

pub trait TextureGenerator {
    fn compute_texture_avg_map(&self, textures: &[Texture]) -> KdMap;

    /// Convert the raw data of all images within `virt_root` into textures,
    /// and extract the resource pack version from its `version.json`.
    fn extract_data(&self, virt_root: VfsPath) -> (Vec<Texture>, u64) {
        let textures = virt_root
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .filter_map(|path| {
                let md = path.metadata().unwrap();
                let is_file = md.file_type == VfsFileType::File;
                let is_png = path.as_str().ends_with(PNG_EXT);
                (is_file && is_png).then(|| {
                    let capacity = md.len as usize;
                    // It's very important that the length of `buf` starts at 0, since
                    // `read_to_end` appends data instead of overwriting it.
                    let mut buf = Vec::with_capacity(capacity);
                    let file_size = path.open_file().unwrap().read_to_end(&mut buf).unwrap();
                    // This being true guarantees no reallocations are made.
                    assert_eq!(capacity, file_size);
                    let img = load_from_memory_with_format(&buf, ImageFormat::Png)
                        .unwrap()
                        .to_rgba8();
                    let path = Utf8PathBuf::from(path.as_str());
                    Texture { img, path }
                })
            })
            .collect();

        // It's fucking beautiful.
        let json_string = virt_root
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .find(|path| path.as_str().ends_with(VERSION_JSON))
            .unwrap()
            .read_to_string()
            .unwrap();
        let resource_pack_version = serde_json::from_str::<serde_json::Value>(&json_string)
            .unwrap()
            .as_object()
            .unwrap()
            .get("pack_version")
            .unwrap()
            .as_object()
            .unwrap()
            .get("resource")
            .unwrap()
            .as_number()
            .unwrap()
            .as_u64()
            .unwrap();

        (textures, resource_pack_version)
    }

    fn write(&self, target_dir: &str, textures: Vec<Texture>, map: KdMap) {
        //let tmp =
        textures.into_par_iter().for_each(|texture| {
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
                    match *old_pixel.0.last().unwrap() != 0 {
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
            let path = target_dir.to_string() + texture.path.as_str();
            let tmp = std::path::Path::new(path.as_str()).parent().unwrap();
            fs::create_dir_all(tmp).unwrap();
            let f = fs::File::create(path).unwrap();
            let enc = PngEncoder::new_with_quality(f, CompressionType::Best, FilterType::Adaptive);
            new_image.write_with_encoder(enc).unwrap();
            /*Texture {
                img: new_image,
                path: texture.path,
            }*/
        });
        //.collect::<Vec<_>>();
        // _ = tmp;
    }
}
