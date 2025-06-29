#![allow(unused)]

use std::{env, fs};
use woolblocks::*;

const DIR: &str = "tmp";

fn main() {
    if fs::exists(DIR).unwrap() {
        fs::remove_dir_all(DIR).unwrap();
    }
    let client_jar = ClientJar::new_release();
    Xbrz.generate(DIR, client_jar, true);
    //SingleTexture::new("white_wool", 2).generate(DIR, client_jar, true);
    //AllTextures.generate(DIR, client_jar, true);
    return;

    let rp_dir = env::args().into_iter().nth(1).unwrap();
    let new_pack_name = env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());
}
