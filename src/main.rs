#![allow(unused)]

use image::codecs::png::*;
use image::*;
use rayon::prelude::*;
use std::{env, fs};
use woolblocks::*;

const DIR: &str = "tmp";

fn main() {
    if fs::exists(DIR).unwrap() {
        fs::remove_dir_all(DIR).unwrap();
    }
    let cj = ClientJar::new_release();
    cj.print_version_json();
    let root = cj.into_virt_mem();
    let wool = SingleTexture::new("white_wool", 2);
    //let wool = AllTextures;
    let (textures, version) = wool.extract_data(root);
    println!("resource pack version: {}", version);
    let map = wool.compute_texture_avg_map(&textures);
    println!("{} distinct textures", map.len());
    //dump_textures(map.textures().to_vec());
    println!("len: {}", map.len());
    wool.write(DIR, textures, map);
    return;

    let rp_dir = env::args().into_iter().nth(1).unwrap();
    let new_pack_name = env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());
}

fn dump_textures(mut textures: Vec<RgbaImage>) {
    const DIR: &str = "tmp_imgs/";
    if fs::exists(DIR).unwrap() {
        fs::remove_dir_all(DIR).unwrap();
    }
    fs::create_dir(DIR).unwrap();
    textures.sort_unstable_by(|a, b| {
        let a = avg_lum(&a);
        let b = avg_lum(&b);
        a.total_cmp(&b)
    });
    textures
        .into_par_iter()
        .enumerate()
        .for_each(|(i, texture)| {
            let path = DIR.to_string() + &i.to_string() + ".png";
            let f = fs::File::create_new(path).unwrap();
            let enc = PngEncoder::new_with_quality(f, CompressionType::Best, FilterType::default());
            texture.write_with_encoder(enc).unwrap();
        });
}

pub fn avg_lum(image: &RgbaImage) -> f32 {
    let (width, height) = image.dimensions();
    let num_pixels = (width as f32) * (height as f32);
    // Sum up each pixel's luminance (ignoring alpha), normalized to [0.0, 1.0].
    let total_lum: f32 = image
        .pixels()
        .map(|px| {
            let channels = px.channels();
            let r = channels[0] as f32;
            let g = channels[1] as f32;
            let b = channels[2] as f32;
            // Rec. 709
            ((0.2126 * r) + (0.7152 * g) + (0.0722 * b)) / 255.0
        })
        .sum();

    total_lum / num_pixels
}
