#![allow(unused)]

use std::env;
use woolblocks::client::*;

fn main() {
    let t = ClientJar::new_release();
    let x = t.extract_to_virt_fs();
    let w = woolblocks::Wool;
    let v = w.get_dst_textures(x);
    let wowie = w.get_src_textures(&v);
    println!("{}", v.len());
    println!("{}", wowie.len());
    println!("{:#?}", v);
    return;

    let rp_dir = env::args().into_iter().nth(1).unwrap();
    let new_pack_name = env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());
    woolblocks::generate_texture_pack(rp_dir.into(), new_pack_name);
}
