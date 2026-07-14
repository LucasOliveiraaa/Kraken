use kmath::Vec3u;

use crate::{presenting::SwapchainImageId, resources::MemoryLocation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageId(pub u32);
impl std::fmt::Display for ImageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImageId({})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageIdRef {
    Image(ImageId),
    SwapchainImage(SwapchainImageId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Format {
    Rgba8Unorm,
    Rgba8Srgb,
    Bgra8Unorm,
    Bgra8Srgb,
    R32Float,
    R32Uint,
    R32Sint,
    R16Float,
    R16Uint,
    R16Sint,
    R8Unorm,
    R8Uint,
    R8Sint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ColorSpace {
    SrgbNonlinear,
    DisplayP3Nonlinear,
    ExtendedSrgbLinear,
    Hdr10St2084,
    Hdr10Hlg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceFormat {
    pub format: Format,
    pub color_space: ColorSpace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageType {
    Image1D,
    Image2D,
    Image3D,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ImageUsage: u32 {
        const SAMPLED = 1 << 0;
        const STORAGE = 1 << 1;
        const COLOR_ATTACHMENT = 1 << 2;
        const DEPTH_STENCIL_ATTACHMENT = 1 << 3;
        const TRANSFER_SRC = 1 << 4;
        const TRANSFER_DST = 1 << 5;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SampleCount {
    One,
    Two,
    Four,
    Eight,
    Sixteen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageLayout {
    Undefined,
    General,
    ColorAttachmentOptimal,
    DepthStencilAttachmentOptimal,
    DepthStencilReadOnlyOptimal,
    ShaderReadOnlyOptimal,
    TransferSrcOptimal,
    TransferDstOptimal,
    PresentSrc,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct ImageAspect: u32 {
        const COLOR = 1 << 0;
        const DEPTH = 1 << 1;
        const STENCIL = 1 << 2;
        const METADATA = 1 << 3;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageSubresourceRange {
    pub aspect_mask: ImageAspect,
    pub base_mip: u32,
    pub level_count: u32,
    pub base_array: u32,
    pub layer_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageDesc {
    pub name: String,
    pub image_type: ImageType,
    pub size: Vec3u,
    pub format: Format,
    pub usage: ImageUsage,
    pub location: MemoryLocation,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub sample_count: SampleCount,
}
