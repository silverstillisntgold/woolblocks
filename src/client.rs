use crate::manifest::{Version, get_client_jar_as_bytes};
use crate::{SIZE, TARGET_DIR, Texture};
use camino::Utf8PathBuf;
use image::codecs::png::*;
use image::*;
use rayon::prelude::*;
use std::collections::HashMap as HashMapCore;
use std::fs;
use std::io::{Cursor, copy};
use vfs::{MemoryFS, VfsFileType, VfsPath};
use zip::ZipArchive;

type HashMap<K, V> = HashMapCore<K, V, foldhash::quality::RandomState>;

/// Wraps the raw bytes of a client jar.
pub struct ClientJar(Box<[u8]>);

impl From<Box<[u8]>> for ClientJar {
    fn from(value: Box<[u8]>) -> Self {
        Self(value)
    }
}

impl ClientJar {
    /// Create a new [`ClientJar`] from the user-provided `version_id`.
    ///
    /// The program will panic if `version_id` match isn't found.
    pub fn new(version_id: &str) -> Self {
        get_client_jar_as_bytes(Version::Custom(version_id)).into()
    }

    /// Create a new [`ClientJar`] from the latest release version available.
    pub fn new_release() -> Self {
        get_client_jar_as_bytes(Version::Release).into()
    }

    /// Create a new [`ClientJar`] from the latest snapshot version available.
    pub fn new_snapshot() -> Self {
        get_client_jar_as_bytes(Version::Snapshot).into()
    }

    /// Convert contents of `self` into an in-memory, virtual filesystem.
    /// The returned [`VfsPath`] represents the root of said filesystem.
    pub fn into_virt_mem(self) -> VfsPath {
        self.into()
    }

    pub fn print_json_data(&self) {
        let copy = ClientJar::from(self.0.clone());
        let json = copy
            .into_virt_mem()
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .find(|path| path.as_str().ends_with("version.json"))
            .unwrap()
            .read_to_string()
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        println!("{}", json);
        println!("{:#?}", parsed.as_object().unwrap());
    }
}

impl From<ClientJar> for VfsPath {
    fn from(value: ClientJar) -> Self {
        // Why worry about your operating system caching your filesystem
        // when you can just force everything into RAM :).
        let virt_root = VfsPath::new(MemoryFS::new());
        let reader = Cursor::new(value.0);
        let mut zip = ZipArchive::new(reader).unwrap();
        for file_number in 0..zip.len() {
            let mut zipped_file = zip.by_index(file_number).unwrap();
            assert!(
                zipped_file.is_file(),
                "`ZipArchive::by_index` should only provide files"
            );
            if let Some(path) = zipped_file
                .enclosed_name()
                // Strings are just simpler to work with here.
                .map(|path| path.into_os_string().into_string().unwrap())
                .filter(|path| {
                    let is_png = path.ends_with(".png");
                    let is_in_target_dir = TARGET_DIR
                        .into_iter()
                        .any(|target_dir| path.contains(target_dir));
                    let is_version_json = path.ends_with("version.json");
                    (is_png && is_in_target_dir) || is_version_json
                })
                // Eliminate some goofy ah textures.
                .filter(|path| !path.contains("test") && !path.contains("debug"))
            {
                let path = virt_root.join(path).unwrap();
                path.parent().create_dir_all().unwrap();
                let mut virt_file = path.create_file().unwrap();
                copy(&mut zipped_file, &mut virt_file).unwrap();
            }
        }
        virt_root
    }
}

pub trait TextureGenerator {
    fn compute_texture_avg_map(&self, textures: &[Texture]) -> HashMap<Rgba<u8>, RgbaImage>;

    /// Convert the raw data of all images within `virt_root` into textures,
    /// and extract the resource pack version from its `version.json`.
    fn extract_data(&self, virt_root: VfsPath) -> (Vec<Texture>, u64) {
        let textures = virt_root
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .filter_map(|path| {
                let md = path.metadata().unwrap();
                let is_file = md.file_type == VfsFileType::File;
                let is_png = path.as_str().ends_with(".png");
                (is_file && is_png).then(|| {
                    let capacity = md.len as usize;
                    let mut buf = Vec::with_capacity(capacity);
                    let file_size = path.open_file().unwrap().read_to_end(&mut buf).unwrap();
                    // Want this to always be true to guarantee no reallocations are made.
                    assert!(file_size == capacity);
                    let img = load_from_memory_with_format(&buf, ImageFormat::Png)
                        .unwrap()
                        .to_rgba8();
                    let path = Utf8PathBuf::from(path.as_str());
                    Texture { img, path }
                })
            })
            .collect();

        // It's fucking beautiful.
        let json_string = virt_root
            .walk_dir()
            .unwrap()
            .map(Result::unwrap)
            .find(|path| path.as_str().ends_with("version.json"))
            .unwrap()
            .read_to_string()
            .unwrap();
        let resource_pack_version = serde_json::from_str::<serde_json::Value>(&json_string)
            .unwrap()
            .as_object()
            .unwrap()
            .get("pack_version")
            .unwrap()
            .as_object()
            .unwrap()
            .get("resource")
            .unwrap()
            .as_number()
            .unwrap()
            .as_u64()
            .unwrap();

        (textures, resource_pack_version)
    }

    fn write(
        &self,
        target_dir: &str,
        textures: Vec<Texture>,
        pixel_map: HashMap<Rgba<u8>, RgbaImage>,
    ) {
        textures.into_par_iter().for_each(|texture| {
            let old_width = texture.img.width();
            let old_height = texture.img.height();
            let new_width = old_width * SIZE;
            let new_height = old_height * SIZE;
            let mut new_img = RgbaImage::new(new_width, new_height);
            for x in 0..old_width {
                for y in 0..old_height {
                    // Get current pixel from old texture.
                    let old_pixel = texture.img.get_pixel(x, y);
                    // Find pixel with smallest distance that we can use in
                    // our texture lookup table.
                    let new_pixel = pixel_map
                        .par_iter()
                        .map(|(key, _)| key)
                        .min_by_key(|pixel| find_distance(pixel, old_pixel))
                        .unwrap();
                    // Lookup closest valid texture.
                    let closest_block = pixel_map.get(new_pixel).unwrap();
                    let x_offset = x * SIZE;
                    let y_offset = y * SIZE;
                    // Iterate over sub-pixel group.
                    for dx in 0..SIZE {
                        for dy in 0..SIZE {
                            let mut pixel = closest_block.get_pixel(dx, dy).clone();
                            if old_pixel.0[3] == 0 {
                                for val in &mut pixel.0 {
                                    *val = 0;
                                }
                            } else {
                                pixel.0[3] = old_pixel.0[3];
                            }
                            new_img.put_pixel(x_offset + dx, y_offset + dy, pixel);
                        }
                    }
                }
            }
            let f = fs::File::create(texture.path).unwrap();
            let enc = PngEncoder::new_with_quality(f, CompressionType::Best, FilterType::default());
            new_img.write_with_encoder(enc).unwrap();
        });
    }
}

fn find_distance(a: &Rgba<u8>, b: &Rgba<u8>) -> i64 {
    let dr = a.0[0] as i64 - b.0[0] as i64;
    let dg = a.0[1] as i64 - b.0[1] as i64;
    let db = a.0[2] as i64 - b.0[2] as i64;
    dr * dr + dg * dg + db * db
}

pub struct WhiteWool;
impl TextureGenerator for WhiteWool {
    fn compute_texture_avg_map(&self, textures: &[Texture]) -> HashMap<Rgba<u8>, RgbaImage> {
        const STEP: usize = 3;
        let white_wool = textures
            .into_iter()
            .find(|t| t.path.ends_with("white_wool.png"))
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
