use super::{InternalGenerator, MappingGenerator};
use crate::{KdMap, SIZE, TARGET_DIR, Texture};
use image::RgbaImage;
use rayon::prelude::*;

/// Textures which are shitty sources.
const TEXTURE_EXCLUSION_LIST: &[&str] = &[
    "book",
    "bee_nest_front",
    "cauldron_side",
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
    fn modify_textures(&self, textures: Vec<Texture>) -> Vec<Texture> {
        let map = self.create_rgb_map(&textures);
        self.map(textures, map)
    }
}

impl MappingGenerator for AllTextures {
    fn create_rgb_map(&self, textures: &[Texture]) -> KdMap {
        textures
            .into_par_iter()
            .filter(|texture| texture.img.width() == SIZE && texture.img.height() == SIZE)
            .filter(|texture| {
                let s = texture.path.as_str();
                s.contains(TARGET_DIR[0])
                    && TEXTURE_EXCLUSION_LIST.into_iter().all(|t| !s.contains(t))
            })
            .filter_map(|texture| {
                calculate_average(&texture.img).map(|avg| (avg, texture.img.clone()))
            })
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
