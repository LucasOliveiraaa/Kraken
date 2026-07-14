use std::{collections::HashMap, sync::Arc};

use ash::{khr, vk};
use contract::{
    GpuError, GpuResult,
    presenting::{PresentMode, SwapchainDesc, SwapchainImageId},
};

use crate::{
    TryFromVk, VkQueue,
    core::{FromContract, IntoVk, VkContext},
    map_vk,
    presenting::surface::VkSurface,
    sync::{VkFence, VkSemaphore},
};

impl FromContract<PresentMode> for vk::PresentModeKHR {
    fn from_contract(present_mode: PresentMode) -> Self {
        match present_mode {
            PresentMode::Immediate => vk::PresentModeKHR::IMMEDIATE,
            PresentMode::Mailbox => vk::PresentModeKHR::MAILBOX,
            PresentMode::Fifo => vk::PresentModeKHR::FIFO,
            PresentMode::FifoRelaxed => vk::PresentModeKHR::FIFO_RELAXED,
        }
    }
}
impl TryFromVk<vk::PresentModeKHR> for PresentMode {
    fn try_from_vk(present_mode: vk::PresentModeKHR) -> GpuResult<Self> {
        match present_mode {
            vk::PresentModeKHR::IMMEDIATE => Ok(PresentMode::Immediate),
            vk::PresentModeKHR::MAILBOX => Ok(PresentMode::Mailbox),
            vk::PresentModeKHR::FIFO => Ok(PresentMode::Fifo),
            vk::PresentModeKHR::FIFO_RELAXED => Ok(PresentMode::FifoRelaxed),
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("PresentMode {:?}", present_mode),
            }),
        }
    }
}

pub struct VkSwapchainImage {
    handle: vk::Image,
}

impl VkSwapchainImage {
    pub fn new(handle: vk::Image) -> Self {
        Self { handle: handle }
    }

    pub fn handle(&self) -> vk::Image {
        self.handle
    }
}

fn pick_present_queue(context: &VkContext, surface: &VkSurface) -> GpuResult<Arc<VkQueue>> {
    if surface.test_surface_support(&context.queues.graphics)? {
        Ok(context.queues.graphics.clone())
    } else if surface.test_surface_support(&context.queues.compute)? {
        Ok(context.queues.compute.clone())
    } else if surface.test_surface_support(&context.queues.transfer)? {
        Ok(context.queues.transfer.clone())
    } else {
        Err(GpuError::InitializationFailed {
            reason: "No queue supports presenting".to_string(),
        })
    }
}

pub struct VkSwapchain {
    _context: Arc<VkContext>,
    device: Arc<khr::swapchain::Device>,
    handle: vk::SwapchainKHR,
    present_queue: Arc<VkQueue>,
    images: HashMap<SwapchainImageId, Arc<VkSwapchainImage>>,
}

impl VkSwapchain {
    pub fn new(
        context: Arc<VkContext>,
        device: Arc<khr::swapchain::Device>,
        surface: &VkSurface,
        desc: &SwapchainDesc,
    ) -> GpuResult<(Self, Vec<Arc<VkSwapchainImage>>)> {
        let surface_caps = surface.caps();

        let extent = if surface_caps.current_extent.x() == std::u32::MAX {
            desc.extent
                .clamp(surface_caps.min_image_extent, surface_caps.max_image_extent)
        } else {
            surface_caps.current_extent
        };

        let image_count = if surface_caps.max_image_count == 0 {
            desc.image_count.max(surface_caps.min_image_count)
        } else {
            desc.image_count
                .clamp(surface_caps.min_image_count, surface_caps.max_image_count)
        };

        let composite_alpha = if surface_caps
            .supported_composite_alpha
            .contains(vk::CompositeAlphaFlagsKHR::OPAQUE)
        {
            vk::CompositeAlphaFlagsKHR::OPAQUE
        } else if surface_caps
            .supported_composite_alpha
            .contains(vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED)
        {
            vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED
        } else if surface_caps
            .supported_composite_alpha
            .contains(vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED)
        {
            vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED
        } else {
            vk::CompositeAlphaFlagsKHR::INHERIT
        };

        let swapchain_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: std::ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: surface.handle(),
            image_format: desc.format.format.into_vk(),
            image_color_space: desc.format.color_space.into_vk(),
            image_usage: desc.usage.into_vk(),
            image_extent: extent.into_vk(),
            composite_alpha,
            clipped: vk::TRUE,
            image_array_layers: 1,
            min_image_count: image_count,
            present_mode: desc.present_mode.into_vk(),
            pre_transform: surface_caps.current_transform,
            old_swapchain: vk::SwapchainKHR::null(),
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            p_queue_family_indices: std::ptr::null(),
            queue_family_index_count: 0,
            ..Default::default()
        };

        let handle = unsafe {
            device
                .create_swapchain(&swapchain_info, None)
                .map_err(map_vk)?
        };

        let images = unsafe {
            device
                .get_swapchain_images(handle)
                .map_err(map_vk)?
                .into_iter()
                .map(|image| Arc::new(VkSwapchainImage::new(image)))
                .collect::<Vec<Arc<VkSwapchainImage>>>()
        };

        Ok((
            Self {
                _context: context.clone(),
                device,
                handle,
                present_queue: pick_present_queue(&context, surface)?,
                images: HashMap::new(),
            },
            images,
        ))
    }

    pub fn handle(&self) -> vk::SwapchainKHR {
        self.handle
    }

    pub fn image_count(&self) -> u32 {
        self.images.len() as u32
    }

    pub fn register_image_id(
        &mut self,
        image: Arc<VkSwapchainImage>,
        image_id: SwapchainImageId,
    ) -> GpuResult<()> {
        self.images.insert(image_id, image);
        Ok(())
    }

    pub fn get_image(&self, image_id: SwapchainImageId) -> GpuResult<Arc<VkSwapchainImage>> {
        self.images
            .get(&image_id)
            .cloned()
            .ok_or(GpuError::InvalidSwapchainImageId(image_id))
    }

    pub fn acquire_next_image(
        &self,
        semaphore: Option<&VkSemaphore>,
        fence: Option<&VkFence>,
    ) -> GpuResult<(SwapchainImageId, bool)> {
        let (image_index, suboptimal) = unsafe {
            self.device
                .acquire_next_image(
                    self.handle,
                    std::u64::MAX,
                    semaphore
                        .map(VkSemaphore::handle)
                        .unwrap_or(vk::Semaphore::null()),
                    fence.map(VkFence::handle).unwrap_or(vk::Fence::null()),
                )
                .map_err(map_vk)?
        };

        Ok((SwapchainImageId(image_index), suboptimal))
    }

    pub fn present_image(
        self: Arc<Self>,
        image_id: SwapchainImageId,
        wait_semaphores: &[Arc<VkSemaphore>],
    ) -> GpuResult<bool> {
        self.present_queue
            .present(&self.device, &[self.clone()], &[image_id], wait_semaphores)
    }
}

impl Drop for VkSwapchain {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_swapchain(self.handle, None);
        }
    }
}
