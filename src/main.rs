#![allow(unused)]

use std::{env, fs};
use woolblocks::*;

const GENERATED: &str = "generated/";
const DIR: &str = "generated/all";

fn main() {
    /*if fs::exists(GENERATED).unwrap() {
        fs::remove_dir_all(GENERATED).unwrap();
    }*/
    let version_id = Version::Release;
    //Xbrz.generate(DIR, version_id, true);
    //SingleTexture::new("white_wool", 2).generate(DIR, version_id, true);
    AllTextures.generate(DIR, version_id, true);
    return;

    let rp_dir = env::args().into_iter().nth(1).unwrap();
    let new_pack_name = env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());
}
