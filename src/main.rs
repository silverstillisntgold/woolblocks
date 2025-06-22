//use std::env;
use woolblocks::*;

fn main() {
    let cj = ClientJar::new_release();
    cj.print_json_data();
    let root = cj.into_virt_mem();
    let wool = SingleTexture::new("iron_block", 2);
    let (textures, version) = wool.extract_data(root);
    println!("RP version: {}", version);
    let (keys, values) = wool.compute_texture_avg_map(&textures);
    println!("{} -- {}", keys.size(), values.len());
    wool.write("tmp", textures, keys, values);

    /*let rp_dir = env::args().into_iter().nth(1).unwrap();
    let new_pack_name = env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());*/
}
