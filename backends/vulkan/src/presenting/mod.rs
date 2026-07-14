mod surface;
mod swapchain;

pub use surface::*;
pub use swapchain::*;

use std::sync::{Arc, OnceLock};

use ash::khr;
use contract::{
    GpuError, GpuResult,
    ffi::WindowHandles,
    presenting::{
        PresentingGateway, SurfaceId, SwapchainAcquiredImage, SwapchainDesc, SwapchainId,
        SwapchainImageId,
    },
    sync::{FenceId, SemaphoreId},
};

use crate::{
    alloc::Slab,
    core::{VkContext, WeakGateways},
};

pub struct VkPresentingGateway {
    context: Arc<VkContext>,
    gateways: OnceLock<WeakGateways>,

    surface_instance: Arc<khr::surface::Instance>,
    swapchain_device: Arc<khr::swapchain::Device>,

    surfaces: parking_lot::Mutex<Slab<Arc<VkSurface>>>,
    swapchains: parking_lot::Mutex<Slab<Arc<VkSwapchain>>>,
    swapchain_images: parking_lot::Mutex<Slab<Arc<VkSwapchainImage>>>,
}

impl VkPresentingGateway {
    pub fn new(context: Arc<VkContext>) -> Self {
        Self {
            context: context.clone(),
            gateways: OnceLock::new(),

            surface_instance: Arc::new(khr::surface::Instance::new(
                context.instance.entry(),
                context.instance.handle(),
            )),
            swapchain_device: Arc::new(khr::swapchain::Device::new(
                context.instance.handle(),
                context.device.handle(),
            )),
            surfaces: parking_lot::Mutex::new(Slab::new()),
            swapchains: parking_lot::Mutex::new(Slab::new()),
            swapchain_images: parking_lot::Mutex::new(Slab::new()),
        }
    }

    pub fn set_gateways(&self, gateways: WeakGateways) {
        self.gateways.set(gateways).unwrap();
    }

    pub fn get_surface(&self, surface_id: SurfaceId) -> GpuResult<Arc<VkSurface>> {
        let surfaces = self.surfaces.lock();
        let surface = surfaces
            .get(surface_id.0 as usize)
            .ok_or(GpuError::InvalidSurfaceId(surface_id))?;

        Ok(surface.clone())
    }

    pub fn get_swapchain(&self, swapchain_id: SwapchainId) -> GpuResult<Arc<VkSwapchain>> {
        let swapchains = self.swapchains.lock();
        let swapchain = swapchains
            .get(swapchain_id.0 as usize)
            .ok_or(GpuError::InvalidSwapchainId(swapchain_id))?;

        Ok(swapchain.clone())
    }

    pub fn get_swapchain_image(
        &self,
        swapchain_image_id: SwapchainImageId,
    ) -> GpuResult<Arc<VkSwapchainImage>> {
        let swapchain_images = self.swapchain_images.lock();
        let swapchain_image = swapchain_images
            .get(swapchain_image_id.0 as usize)
            .ok_or(GpuError::InvalidSwapchainImageId(swapchain_image_id))?;

        Ok(swapchain_image.clone())
    }
}

impl PresentingGateway for VkPresentingGateway {
    fn create_surface(&self, handles: &WindowHandles) -> GpuResult<SurfaceId> {
        let surface = VkSurface::new(self.context.clone(), self.surface_instance.clone(), handles)?;

        let mut surfaces = self.surfaces.lock();
        Ok(SurfaceId(surfaces.insert(Arc::new(surface)) as u32))
    }

    fn destroy_surface(&self, surface_id: SurfaceId) -> GpuResult<()> {
        let mut surfaces = self.surfaces.lock();
        if surfaces.remove(surface_id.0 as usize).is_none() {
            return Err(GpuError::InvalidSurfaceId(surface_id));
        }
        Ok(())
    }

    fn query_surface_formats(
        &self,
        surface_id: SurfaceId,
    ) -> GpuResult<Vec<contract::resources::SurfaceFormat>> {
        let surface = self.get_surface(surface_id)?;

        surface.query_formats()
    }

    fn create_swapchain(&self, desc: &SwapchainDesc) -> GpuResult<SwapchainId> {
        let surfaces = self.surfaces.lock();
        let surface = surfaces
            .get(desc.surface_id.0 as usize)
            .ok_or(GpuError::InvalidSurfaceId(desc.surface_id))?;

        let (mut swapchain, images) = VkSwapchain::new(
            self.context.clone(),
            self.swapchain_device.clone(),
            surface,
            desc,
        )?;

        let mut swapchain_images = self.swapchain_images.lock();
        for image in &images {
            let handle = swapchain_images.insert(image.clone());
            swapchain.register_image_id(image.clone(), SwapchainImageId(handle as u32))?;
        }

        let mut swapchains = self.swapchains.lock();
        Ok(SwapchainId(swapchains.insert(Arc::new(swapchain)) as u32))
    }

    fn destroy_swapchain(&self, swapchain_id: SwapchainId) -> GpuResult<()> {
        let mut swapchains = self.swapchains.lock();
        if swapchains.remove(swapchain_id.0 as usize).is_none() {
            return Err(GpuError::InvalidSwapchainId(swapchain_id));
        }
        Ok(())
    }

    fn acquire_next_image(
        &self,
        swapchain_id: SwapchainId,
        semaphore: Option<SemaphoreId>,
        fence: Option<FenceId>,
    ) -> GpuResult<SwapchainAcquiredImage> {
        let swapchain = self.get_swapchain(swapchain_id)?;

        let sync = self.gateways.get().unwrap().sync.upgrade().unwrap();
        let semaphore = match semaphore {
            Some(semaphore_id) => {
                let semaphore = sync.get_semaphore(semaphore_id)?;
                Some(semaphore)
            }
            None => None,
        };
        let fence = match fence {
            Some(fence_id) => {
                let fence = sync.get_fence(fence_id)?;
                Some(fence)
            }
            None => None,
        };

        let (image_id, suboptimal) =
            swapchain.acquire_next_image(semaphore.as_deref(), fence.as_deref())?;
        Ok(SwapchainAcquiredImage {
            image_id,
            suboptimal,
        })
    }

    fn present_image(
        &self,
        swapchain_id: SwapchainId,
        image_id: SwapchainImageId,
        wait_semaphores: Vec<SemaphoreId>,
    ) -> GpuResult<bool> {
        let swapchain = self.get_swapchain(swapchain_id)?;

        let sync = self.gateways.get().unwrap().sync.upgrade().unwrap();
        let wait_semaphores = wait_semaphores
            .iter()
            .map(|&semaphore_id| sync.get_semaphore(semaphore_id))
            .collect::<GpuResult<Vec<_>>>()?;

        swapchain.present_image(image_id, &wait_semaphores)
    }
}
