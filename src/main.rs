#![allow(unused)]

use image::codecs::png::*;
use image::*;
use std::{env, fs};
use woolblocks::client::*;

fn main() {
    let cj = ClientJar::new_release();
    cj.print_json_data();
    let root = cj.into_virt_mem();
    let wool = WhiteWool;
    let (textures, version) = wool.extract_data(root);
    println!("RP version: {}", version);
    let map = wool.compute_texture_avg_map(&textures);
    println!("{}", map.len());
    let v = map.into_values().collect::<Vec<_>>();
    dump_textures(v);
    return;

    let rp_dir = env::args().into_iter().nth(1).unwrap();
    let new_pack_name = env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());
    woolblocks::generate_texture_pack(rp_dir.into(), new_pack_name);
}

fn dump_textures(textures: Vec<RgbaImage>) {
    const DIR: &str = "tmp_imgs/";
    if fs::exists(DIR).unwrap() {
        fs::remove_dir_all(DIR).unwrap();
    }
    fs::create_dir(DIR).unwrap();
    for i in 0..textures.len() {
        let path = DIR.to_string() + &i.to_string() + ".png";
        let f = fs::File::create_new(path).unwrap();
        let enc = PngEncoder::new_with_quality(f, CompressionType::Best, FilterType::default());
        textures[i].write_with_encoder(enc).unwrap();
    }
}
