#![allow(unused)]

use std::env;
use woolblocks::client::*;

fn main() {
    let cj = ClientJar::new_release();
    cj.print_json_data();
    let root = cj.into_virt_mem();
    let wool = WhiteWool;
    let (textures, version) = wool.extract_data(root);
    println!("RP version: {}", version);
    return;

    let rp_dir = env::args().into_iter().nth(1).unwrap();
    let new_pack_name = env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());
    woolblocks::generate_texture_pack(rp_dir.into(), new_pack_name);
}
