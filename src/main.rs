#![allow(unused)]

use image::codecs::png::*;
use image::*;
use rayon::prelude::*;
use std::{env, fs};
use woolblocks::*;

const DIR: &str = "tmp/";

fn main() {
    if fs::exists(DIR).unwrap() {
        fs::remove_dir_all(DIR).unwrap();
    }
    let client_jar = ClientJar::new_release();
    //SingleTexture::new("white_wool", 2).run(client_jar);
    AllTextures.run(DIR, client_jar);
    return;

    let rp_dir = env::args().into_iter().nth(1).unwrap();
    let new_pack_name = env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());
}
