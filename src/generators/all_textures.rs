use crate::{
    FileData, SIZE, TextureData,
    generators::{InternalGenerator, MappingGenerator},
    kdmap::KdMap,
};
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
    "cartography_table",
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
    "target",
    "furnace",
];

pub struct AllTextures;

impl InternalGenerator for AllTextures {
    fn generator_name(&self) -> &'static str {
        "pixelblocks"
    }

    fn modify_textures(&self, textures: Box<[TextureData]>) -> Box<[TextureData]> {
        let map = self.create_rgb_map(&textures);
        self.map(map, textures)
    }
}

impl MappingGenerator for AllTextures {
    fn create_rgb_map(&self, textures: &[TextureData]) -> KdMap {
        textures
            .into_par_iter()
            .filter_map(|texture_data| match &texture_data.file {
                FileData::Texture(texture) => Some((texture, &texture_data.path)),
                _ => None,
            })
            .filter(|(texture, _)| texture.width() == SIZE && texture.height() == SIZE)
            .filter(|(_, path)| {
                // path.as_str().contains(INCLUSIONS[0])
                //     && LOCAL_EXCLUSIONS.iter().all(|t| !path.as_str().contains(t))
                LOCAL_EXCLUSIONS.iter().all(|exclusion| {
                    path.file_name()
                        .is_some_and(|file_name| !file_name.contains(exclusion))
                })
            })
            .filter_map(|(texture, _)| calculate_average(texture).map(|avg| (avg, texture.clone())))
            .into()
    }
}

fn calculate_average(texture: &RgbaImage) -> Option<[f64; 3]> {
    if texture.is_empty() {
        return None;
    }

    let mut r_sum = 0;
    let mut g_sum = 0;
    let mut b_sum = 0;

    for pixel in texture.pixels() {
        // Immediately terminate on transparent pixel.
        if pixel[3] == 0 {
            core::hint::cold_path();
            return None;
        }

        r_sum += pixel[0] as u64;
        g_sum += pixel[1] as u64;
        b_sum += pixel[2] as u64;
    }

    let pixel_count = texture.width() as f64 * texture.height() as f64;

    Some([
        r_sum as f64 / pixel_count,
        g_sum as f64 / pixel_count,
        b_sum as f64 / pixel_count,
    ])
}
