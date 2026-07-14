use kmath::Vec2u;

use crate::{presenting::SurfaceId, resources::{ImageUsage, SurfaceFormat}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SwapchainId(pub u32);
impl std::fmt::Display for SwapchainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SwapchainId({})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SwapchainImageId(pub u32);
impl std::fmt::Display for SwapchainImageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SwapchainImageId({})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PresentMode {
    Immediate,
    Mailbox,
    Fifo,
    FifoRelaxed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwapchainDesc {
    pub surface_id: SurfaceId,
    pub extent: Vec2u,
    pub format: SurfaceFormat,
    pub usage: ImageUsage,
    pub present_mode: PresentMode,
    pub image_count: u32,
}
