use crate::generator::TextureGenerator;
use crate::*;
use image::*;

pub struct WhiteWool;
impl TextureGenerator for WhiteWool {
    fn compute_texture_avg_map(&self, textures: &[Texture]) -> HashMap<Rgba<u8>, RgbaImage> {
        const STEP: usize = 8;
        let white_wool = textures
            .into_iter()
            .find(|t| t.path.ends_with("cobblestone.png"))
            .unwrap()
            .img
            .clone();
        let (width, height) = white_wool.dimensions();
        assert!(width == SIZE && height == SIZE);

        (0..u8::MAX)
            .rev()
            .step_by(STEP)
            .flat_map(move |r| (0..u8::MAX).rev().step_by(STEP).map(move |g| (r, g)))
            .flat_map(move |(r, g)| (0..u8::MAX).rev().step_by(STEP).map(move |b| (r, g, b)))
            .map(|(r, g, b)| {
                let rgba_src = Rgba::from([r, g, b, u8::MAX]);
                let mut new_wool = RgbaImage::new(width, height);
                for (x, y, pixel) in white_wool.enumerate_pixels() {
                    let luminance = pixel.to_luma_alpha()[0] as f64 / (u8::MAX as f64);
                    let mut new_pixel = rgba_src.clone();
                    for i in 0..(new_pixel.0.len() - 1) {
                        let new_val = new_pixel[i] as f64 * luminance;
                        new_pixel[i] = new_val.round() as u8;
                    }
                    new_wool.put_pixel(x, y, new_pixel);
                }
                (rgba_src, new_wool)
            })
            .collect()
    }
}
