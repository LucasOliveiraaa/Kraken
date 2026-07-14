mod surface;
mod swapchain;

use kmath::Vec2u;
pub use surface::*;
pub use swapchain::*;

use contract::{
    ffi::WindowHandles,
    resources::{ImageUsage, SurfaceFormat},
};

pub use contract::presenting::PresentMode;

use crate::macros::ToContract;

pub struct SwapchainDesc {
    pub surface: Surface,
    pub extent: Vec2u,
    pub format: SurfaceFormat,
    pub usage: ImageUsage,
    pub present_mode: PresentMode,
    pub image_count: u32,
}

impl ToContract for SwapchainDesc {
    type Contract = contract::presenting::SwapchainDesc;

    fn to_contract(&self) -> Self::Contract {
        Self::Contract {
            surface_id: self.surface.handle(),
            extent: self.extent,
            format: self.format,
            usage: self.usage,
            present_mode: self.present_mode,
            image_count: self.image_count,
        }
    }
}

create_gateway! {
    pub struct PresentingGateway {
        contract::presenting::PresentingGateway
    }
}

impl PresentingGateway {
    create_handle_methods! {
        fn create_surface(handles: &WindowHandles) -> Surface;
        fn create_swapchain(#[desc] desc: &SwapchainDesc) -> Swapchain;
    }
}
