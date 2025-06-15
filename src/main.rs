#![allow(unused)]

use std::env;
use std::fs;

fn main() {
    let t = woolblocks::ClientJar::new_stable(None);
    t.print();
    return;

    let rp_dir = env::args().into_iter().nth(1).unwrap();
    let new_pack_name = env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());
    woolblocks::generate_texture_pack(rp_dir.into(), new_pack_name);
}
