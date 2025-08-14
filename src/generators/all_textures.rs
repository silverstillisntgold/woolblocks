use super::{InternalGenerator, MappingGenerator, SIZE};
use crate::INCLUSIONS;
use crate::types::{FileData, KdMap, TextureData};
use image::RgbaImage;
use rayon::prelude::*;

/// Textures which are shitty sources.
const LOCAL_EXCLUSIONS: &[&str] = &[
    "book",
    "bee_nest_front",
    "cauldron_side",
    "ghast",
    "lamp",
    "beehive",
    "glazed",
    "cartography_table_side1",
    "cartography_table_side2",
    "cartography_table_top",
    "door",
    "comparator",
    "pumpkin",
    "debug",
    "destroy",
    "dispenser",
    "dropper",
    "crafter",
    "crafting",
    "calibrated",
    "lectern",
    "loom_front",
    "trial",
    "loom_top",
    "bulb",
    "lantern",
    "furnace",
    "jigsaw",
    "repeater",
    "test",
    "observer",
];

pub struct AllTextures;

impl InternalGenerator for AllTextures {
    const GENERATOR_NAME: &str = "pixelblocks";

    fn modify_textures(&self, textures: Vec<TextureData>) -> Vec<TextureData> {
        let map = self.create_rgb_map(&textures);
        self.map(textures, map)
    }
}

impl MappingGenerator for AllTextures {
    fn create_rgb_map(&self, textures: &[TextureData]) -> KdMap {
        textures
            .into_par_iter()
            .filter_map(|texture_date| match &texture_date.file {
                FileData::Texture(texture) => Some((texture, &texture_date.path)),
                FileData::McMeta(_) => None,
            })
            .filter(|(texture, _)| texture.width() == SIZE && texture.height() == SIZE)
            .filter(|(_, path)| {
                path.contains(INCLUSIONS[0]) && LOCAL_EXCLUSIONS.iter().all(|t| !path.contains(t))
            })
            .filter_map(|(texture, _)| calculate_average(texture).map(|avg| (avg, texture.clone())))
            .into()
    }
}

fn calculate_average(texture: &RgbaImage) -> Option<[f64; 3]> {
    let pixel_count = (texture.width() * texture.height()) as f64;
    let mut r_sum = 0.0;
    let mut g_sum = 0.0;
    let mut b_sum = 0.0;
    for pixel in texture.pixels() {
        // Immediately terminate on transparent pixel.
        match pixel[3] != 0 {
            true => {
                r_sum += pixel[0] as f64;
                g_sum += pixel[1] as f64;
                b_sum += pixel[2] as f64;
            }
            false => return None,
        }
    }
    Some([
        r_sum / pixel_count,
        g_sum / pixel_count,
        b_sum / pixel_count,
    ])
}
