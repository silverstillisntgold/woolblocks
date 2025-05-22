fn main() {
    let rp_dir = std::env::args().into_iter().nth(1).unwrap();
    let new_pack_name = std::env::args()
        .into_iter()
        .nth(2)
        .unwrap_or("pixelized_wool".to_string());
    woolblocks::generate_texture_pack(rp_dir.into(), new_pack_name);
}
