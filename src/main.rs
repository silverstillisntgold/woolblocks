#![allow(unused)]

use std::env;
use std::fs;
use woolblocks::WORKING_DIR;

fn main() {
    if fs::exists(WORKING_DIR).unwrap() {
        println!("clearing old dir");
        fs::remove_dir_all(WORKING_DIR).unwrap();
    }
    println!("creating new dir");
    fs::create_dir(WORKING_DIR).unwrap();
    println!("downloading client jar");
    let bytes = woolblocks::get_client_jar_as_bytes(None);
    println!("writing to fs (just for lols for now)");
    fs::write(WORKING_DIR.to_string() + "/client.jar", &bytes).unwrap();
    println!("extracting");
    woolblocks::output_directories(bytes);
    return;

    let rp_dir = env::args().into_iter().nth(1).unwrap();
    let new_pack_name = env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());
    woolblocks::generate_texture_pack(rp_dir.into(), new_pack_name);
}
