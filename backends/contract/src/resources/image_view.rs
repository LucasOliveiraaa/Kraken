use crate::resources::{Format, ImageAspect, ImageIdRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImageViewId(pub u32);
impl std::fmt::Display for ImageViewId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImageViewId({})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ImageViewType {
    Image1D,
    Image2D,
    Image3D,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageViewDesc {
    pub image_id: ImageIdRef,

    pub format: Format,
    pub view_type: ImageViewType,

    pub aspect: ImageAspect,

    pub base_mip: u32,
    pub level_count: u32,
    pub base_layer: u32,
    pub layer_count: u32,
}
