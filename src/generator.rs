use crate::{KdMap, PNG_EXT, SIZE, Texture, VERSION_JSON};
use camino::Utf8PathBuf;
use image::codecs::png::*;
use image::*;
use rayon::prelude::*;
use std::fs;
use vfs::{VfsFileType, VfsPath};

pub trait TextureGenerator {
    /// The returned [`KdTree`] contains points which map to indexes of the returned `Vec`.
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
        textures.into_par_iter().for_each(|texture| {
            let old_width = texture.img.width();
            let old_height = texture.img.height();
            let new_width = old_width * SIZE;
            let new_height = old_height * SIZE;
            let mut new_img = RgbaImage::new(new_width, new_height);
            for x in 0..old_width {
                for y in 0..old_height {
                    // Get current pixel from old texture.
                    let old_pixel = texture.img.get_pixel(x, y);
                    let closest_block = map.get_nearest(&[
                        old_pixel[0] as f64,
                        old_pixel[1] as f64,
                        old_pixel[2] as f64,
                    ]);
                    let x_offset = x * SIZE;
                    let y_offset = y * SIZE;
                    // Iterate over sub-pixel group.
                    for dx in 0..SIZE {
                        for dy in 0..SIZE {
                            let mut pixel = closest_block.get_pixel(dx, dy).clone();
                            if old_pixel.0[3] == 0 {
                                for val in &mut pixel.0 {
                                    *val = 0;
                                }
                            } else {
                                pixel.0[3] = old_pixel.0[3];
                            }
                            new_img.put_pixel(x_offset + dx, y_offset + dy, pixel);
                        }
                    }
                }
            }
            let path = target_dir.to_string() + texture.path.as_str();
            let tmp = std::path::Path::new(path.as_str()).parent().unwrap();
            fs::create_dir_all(tmp).unwrap();
            let f = fs::File::create(path).unwrap();
            let enc = PngEncoder::new_with_quality(f, CompressionType::Best, FilterType::Adaptive);
            new_img.write_with_encoder(enc).unwrap();
        });
    }
}
