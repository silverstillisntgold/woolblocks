#![allow(unused)]

use std::{env, fs};
use woolblocks::*;

const DIR: &str = "tmp";

fn main() {
    if fs::exists(DIR).unwrap() {
        fs::remove_dir_all(DIR).unwrap();
    }
    let cj = ClientJar::new_release();
    cj.print_json_data();
    let root = cj.into_virt_mem();
    let wool = SingleTexture::new("cobblestone", 2);
    let (textures, version) = wool.extract_data(root);
    println!("resource pack version: {}", version);
    let map = wool.compute_texture_avg_map(&textures);
    println!("len: {}", map.len());
    wool.write(DIR, textures, map);
    return;

    let rp_dir = env::args().into_iter().nth(1).unwrap();
    let new_pack_name = env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());
}
