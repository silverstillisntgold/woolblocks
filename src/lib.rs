#![allow(unused)]

mod client;
mod manifest;

use camino::*;
use image::{
    codecs::png::{CompressionType, FilterType, PngEncoder},
    *,
};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use walkdir::*;

pub use client::*;

pub const WORKING_DIR: &str = ".wool";
/// Where it all begins.
const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const HOME_DIR_LEN: usize = 17;
const SOURCE_DIR: &str = ".wool/client_data/assets/minecraft/textures/";
/// Directories which will have the majority of their textures replaced.
/// The first entry will be used as the source.
const TARGET_DIR: &[&str] = &["/block/", "/entity/", "/item/", "/trims/"];

const SIZE: u32 = 16;

#[derive(Clone)]
struct Texture {
    img: RgbaImage,
    path: Utf8PathBuf,
}

fn get_wool_map(wool_path: &Utf8Path) -> HashMap<u32, RgbaImage> {
    const RGB_MAX: u32 = 1 << 24;
    let mut img = ImageReader::open(wool_path.as_std_path()).unwrap();
    img.set_format(ImageFormat::Png);
    let base_woool_img = img.decode().unwrap().to_luma8();
    let (width, height) = base_woool_img.dimensions();

    (0..RGB_MAX)
        .into_par_iter()
        .map(|idx| {
            let mut rgba = Rgba::from(idx.to_be_bytes());

            let mut wool_img = RgbaImage::new(width, height);
            for (x, y, pixel) in base_woool_img.enumerate_pixels() {
                let luminance = pixel[0] as f64;

                rgba[0] = ((luminance / 255.0) * (rgba[0] as f64)).round() as u8;
                rgba[1] = ((luminance / 255.0) * (rgba[1] as f64)).round() as u8;
                rgba[2] = ((luminance / 255.0) * (rgba[2] as f64)).round() as u8;
                rgba[3] = u8::MAX;

                wool_img.put_pixel(x, y, rgba);
            }

            (idx, wool_img)
        })
        .collect()
}

pub fn generate_texture_pack(src_path: Utf8PathBuf, new_pack_name: String) {
    let cur_pack_name = src_path.file_name().unwrap().to_owned();
    WalkDir::new(src_path.as_std_path())
        .into_iter()
        .map(Result::unwrap)
        .for_each(|e| {
            let old_entry_location = e.path().to_str().unwrap();
            let new_entry_location = old_entry_location.replace(&cur_pack_name, &new_pack_name);
            if e.file_type().is_dir() {
                fs::create_dir_all(&new_entry_location).unwrap();
            } else if e.file_type().is_file() {
                fs::copy(&old_entry_location, &new_entry_location).unwrap();
            }
        });

    let dst_path = src_path
        .as_str()
        .replace(&cur_pack_name, &new_pack_name)
        .into();
    // All PNG image paths.
    let mut image_paths = get_image_paths(dst_path);

    // All PNG images.
    let images = image_paths
        .into_par_iter()
        .map(get_image)
        .collect::<Vec<_>>();

    let wool_path = images
        .iter()
        .find(|t| t.path.as_str().ends_with("white_wool.png"))
        .map(|t| &t.path)
        .unwrap()
        .as_path();
    let pixel_map = get_wool_map(wool_path);
    return;

    // These are the textures which can be used to substitute for individual pixels.
    let pixel_map = images
        .par_iter()
        .filter_map(|t| {
            let is_block = t.path.parent().unwrap().as_str().ends_with("block");
            let is_debug_or_test = t.path.file_name().unwrap().contains("debug")
                || t.path.file_name().unwrap().contains("test");
            if is_block
                && !is_debug_or_test
                && t.img.width() == SIZE
                && t.img.height() == SIZE
                && t.img
                    .pixels()
                    .all(|p| /*Must be fully opaque*/ p.0[3] == u8::MAX)
            {
                let avg = find_average(&t.img);
                let img = t.img.clone();
                Some((avg, img))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    println!("total images: {}", images.len());
    println!("usable mapping images: {}", pixel_map.len());
    println!("ratio: {:.3}", pixel_map.len() as f64 / images.len() as f64);

    // Pixelate all PNG images.
    images.into_par_iter().for_each(|t| {
        let old_width = t.img.width();
        let old_height = t.img.height();
        let new_width = old_width * SIZE;
        let new_height = old_height * SIZE;
        let mut new_img = RgbaImage::new(new_width, new_height);
        for x in 0..old_width {
            for y in 0..old_height {
                // Get current pixel from old texture.
                let old_pixel = t.img.get_pixel(x, y);
                // Find pixel with smallest distance that we can use in
                // our texture lookup table.
                let new_pixel = pixel_map
                    .par_iter()
                    .map(|(key, _)| key)
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

        let f = fs::File::create(t.path).unwrap();
        let enc = PngEncoder::new_with_quality(f, CompressionType::Best, FilterType::default());
        new_img.write_with_encoder(enc).unwrap();
    });
}

fn get_image_paths(src_path: Utf8PathBuf) -> Vec<Utf8PathBuf> {
    WalkDir::new(src_path.as_std_path())
        .into_iter()
        .map(|e| e.unwrap().into_path())
        .map(|path| Utf8PathBuf::try_from(path).unwrap())
        .filter_map(|path| {
            if path.extension().is_some_and(|s| s.eq("png")) {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

fn get_image(path: Utf8PathBuf) -> Texture {
    let mut img = ImageReader::open(path.as_std_path()).unwrap();
    img.set_format(ImageFormat::Png);
    let img = img.decode().unwrap().to_rgba8();
    Texture { img, path }
}

fn find_distance(a: &Rgba<u8>, b: &Rgba<u8>) -> i64 {
    let dr = a.0[0] as i64 - b.0[0] as i64;
    let dg = a.0[1] as i64 - b.0[1] as i64;
    let db = a.0[2] as i64 - b.0[2] as i64;
    dr * dr + dg * dg + db * db
}

/// Takes an entire `RgbaImage` and returns the average `Rgba` value of it.
fn find_average(img: &RgbaImage) -> Rgba<u8> {
    const PIXEL_COUNT: usize = 4;
    let mut sums = [0.0; PIXEL_COUNT];
    for pixel in img.pixels() {
        for i in 0..PIXEL_COUNT {
            sums[i] += pixel.0[i] as f64;
        }
    }
    let len = img.pixels().len() as f64;
    for i in 0..PIXEL_COUNT {
        sums[i] /= len;
    }
    let result = sums.map(|v| v.round() as u8);
    // We expect that this function is only called on fully opaque
    // blocks, which means that it's average opacity is expected to
    // always be the maximum possible opacity.
    assert_eq!(*result.last().unwrap(), u8::MAX);
    result.into()
}
