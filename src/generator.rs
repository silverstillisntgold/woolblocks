use crate::{HashMap, PNG_EXT, SIZE, Texture, VERSION_JSON};
use camino::Utf8PathBuf;
use image::codecs::png::*;
use image::*;
use rayon::prelude::*;
use std::fs;
use std::num::NonZeroUsize;
use vfs::{VfsFileType, VfsPath};

#[inline]
fn find_distance(a: &Rgba<u8>, b: &Rgba<u8>) -> i64 {
    let dr = a.0[0] as i64 - b.0[0] as i64;
    let dg = a.0[1] as i64 - b.0[1] as i64;
    let db = a.0[2] as i64 - b.0[2] as i64;
    dr * dr + dg * dg + db * db
}

pub trait TextureGenerator {
    fn resolution(&self) -> Option<NonZeroUsize>;

    fn compute_texture_avg_map(&self, textures: &[Texture]) -> HashMap<Rgba<u8>, RgbaImage>;

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

    fn write(
        &self,
        target_dir: &str,
        textures: Vec<Texture>,
        pixel_map: HashMap<Rgba<u8>, RgbaImage>,
    ) {
        // Faster than searching a HashMap.
        let pixel_map_flap = pixel_map.keys().cloned().collect::<Vec<_>>();
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
                    // Find pixel with smallest distance that we can use in
                    // our texture lookup table.
                    let new_pixel = pixel_map_flap
                        .par_iter()
                        .min_by_key(|pixel| find_distance(pixel, old_pixel))
                        .unwrap();
                    // Lookup closest valid texture.
                    let closest_block = pixel_map.get(new_pixel).unwrap();
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
