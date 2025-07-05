#![allow(unused)]

use std::{env, fs};
use woolblocks::*;

const OUTPUT_DIR: &str = "generated/";

fn main() {
    if fs::exists(OUTPUT_DIR).unwrap() {
        fs::remove_dir_all(OUTPUT_DIR).unwrap();
    }
    Xbrz.generate(OUTPUT_DIR, Version::Release, true, false);
    //SingleTexture::new("white_wool", 2).generate(OUTPUT_DIR, Version::Release, true, false);
    //AllTextures.generate(OUTPUT_DIR, Version::Release, true, false);
    return;

    let rp_dir = env::args().into_iter().nth(1).unwrap();
    let new_pack_name = env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());
}
