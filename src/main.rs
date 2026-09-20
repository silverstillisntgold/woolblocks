#![allow(unused)]

use std::{env, fs};
use woolblocks::*;

const OUTPUT_DIR: &str = "generated/";

fn main() {
    let _x = client::ClientFetcher::default().fetch().unwrap();

    println!("{} | {}", _x.0.len(), _x.1);

    // let x = testing().unwrap();
    // fs::write(OUTPUT_DIR.to_owned() + "yes.jar", x).unwrap();

    //Xbrz.generate(OUTPUT_DIR, Version::Release, true, false);
    //SingleTexture::new("white_wool", 2).generate(OUTPUT_DIR, Version::Release, true, false);
    // AllTextures.generate(OUTPUT_DIR, Version::Release, true, false);
    // return;

    // let rp_dir = env::args().nth(1).unwrap();
    // let new_pack_name = env::args().nth(2).unwrap_or("pixelized_wool".to_string());
}
