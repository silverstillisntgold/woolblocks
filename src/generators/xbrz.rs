use super::{InternalGenerator, UpscalingGenerator};

pub struct Xbrz {
    scaling_factor: usize,
}

impl Xbrz {
    pub fn new(scaling_factor: usize) -> Self {
        Self { scaling_factor }
    }
}

impl InternalGenerator for Xbrz {
    fn modify_textures(&self, textures: Vec<crate::Texture>) -> Vec<crate::Texture> {
        _ = self.scaling_factor;
        todo!()
    }
}

impl UpscalingGenerator for Xbrz {
    fn upscale(&self, textures: Vec<crate::Texture>) -> Vec<crate::Texture> {
        todo!()
    }
}
