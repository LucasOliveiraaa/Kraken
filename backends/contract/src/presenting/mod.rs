mod surface;
mod swapchain;

pub use surface::*;
pub use swapchain::*;

use crate::{GpuResult, ffi::WindowHandles, resources::SurfaceFormat, sync::{FenceId, SemaphoreId}};

pub struct SwapchainAcquiredImage {
    pub image_id: SwapchainImageId,
    pub suboptimal: bool,
}

pub trait PresentingGateway: Sync + Send {
    /// Create a new surface given the window handles.
    fn create_surface(&self, handles: &WindowHandles) -> GpuResult<SurfaceId>;

    /// Destroy a surface given its ID.
    fn destroy_surface(&self, surface_id: SurfaceId) -> GpuResult<()>;

    /// Query the supported surface formats for a given surface.
    fn query_surface_formats(&self, surface_id: SurfaceId) -> GpuResult<Vec<SurfaceFormat>>;

    /// Create a new swapchain given its description.
    fn create_swapchain(&self, desc: &SwapchainDesc) -> GpuResult<SwapchainId>;

    /// Destroy a swapchain given its ID.
    fn destroy_swapchain(&self, swapchain_id: SwapchainId) -> GpuResult<()>;

    /// Acquire the next image from the swapchain.
    fn acquire_next_image(&self, swapchain_id: SwapchainId, semaphore: Option<SemaphoreId>, fence: Option<FenceId>) -> GpuResult<SwapchainAcquiredImage>;

    /// Present the image to the screen.
    fn present_image(&self, swapchain_id: SwapchainId, image_id: SwapchainImageId, wait_semaphores: Vec<SemaphoreId>) -> GpuResult<bool>;
}