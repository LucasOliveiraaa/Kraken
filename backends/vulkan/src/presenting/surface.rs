use ash::{khr, vk};
use ash_window::create_surface;
use contract::{
    GpuResult,
    ffi::WindowHandles,
    resources::{SurfaceFormat, ImageUsage},
};
use kmath::Vec2u;

use crate::{
    TryFromVk, TryIntoContract, VkQueue, core::{FromContract, IntoContract, IntoVk, VkContext}, map_vk,
};
use std::sync::Arc;

impl FromContract<SurfaceFormat> for vk::SurfaceFormatKHR {
    fn from_contract(value: SurfaceFormat) -> Self {
        vk::SurfaceFormatKHR {
            format: value.format.into_vk(),
            color_space: value.color_space.into_vk(),
        }
    }
}
impl TryFromVk<vk::SurfaceFormatKHR> for SurfaceFormat {
    fn try_from_vk(value: vk::SurfaceFormatKHR) -> GpuResult<Self> {
        Ok(SurfaceFormat {
            format: value.format.try_into_contract()?,
            color_space: value.color_space.try_into_contract()?,
        })
    }
}

pub struct SurfaceCapabilities {
    pub min_image_count: u32,
    pub max_image_count: u32,
    pub current_extent: Vec2u,
    pub min_image_extent: Vec2u,
    pub max_image_extent: Vec2u,
    pub max_image_array_layers: u32,
    pub supported_usage_flags: ImageUsage,
    pub supported_composite_alpha: vk::CompositeAlphaFlagsKHR,
    pub supported_transforms: vk::SurfaceTransformFlagsKHR,
    pub current_transform: vk::SurfaceTransformFlagsKHR,
}

pub struct VkSurface {
    context: Arc<VkContext>,
    instance: Arc<khr::surface::Instance>,
    handle: vk::SurfaceKHR,
    caps: SurfaceCapabilities,
}

impl VkSurface {
    pub fn new(
        context: Arc<VkContext>,
        instance: Arc<khr::surface::Instance>,
        handles: &WindowHandles,
    ) -> GpuResult<Self> {
        let handle = unsafe {
            create_surface(
                context.instance.entry(),
                context.instance.handle(),
                handles.display_handle,
                handles.window_handle,
                None,
            )
            .map_err(map_vk)?
        };

        let capabilities = unsafe {
            instance
                .get_physical_device_surface_capabilities(context.adapter.handle(), handle)
                .map_err(map_vk)?
        };

        Ok(Self {
            context,
            instance,
            handle,
            caps: SurfaceCapabilities {
                min_image_count: capabilities.min_image_count,
                max_image_count: capabilities.max_image_count,
                current_extent: capabilities.current_extent.into_contract(),
                min_image_extent: capabilities.min_image_extent.into_contract(),
                max_image_extent: capabilities.max_image_extent.into_contract(),
                max_image_array_layers: capabilities.max_image_array_layers,
                supported_usage_flags: capabilities.supported_usage_flags.into_contract(),
                supported_composite_alpha: capabilities.supported_composite_alpha,
                supported_transforms: capabilities.supported_transforms,
                current_transform: capabilities.current_transform,
            },
        })
    }

    pub fn handle(&self) -> vk::SurfaceKHR {
        self.handle
    }

    pub fn caps(&self) -> &SurfaceCapabilities {
        &self.caps
    }

    pub fn test_surface_support(&self, queue: &VkQueue) -> GpuResult<bool> {
        let support = unsafe {
            self.instance
                .get_physical_device_surface_support(
                    self.context.adapter.handle(),
                    queue.family(),
                    self.handle,
                )
                .map_err(map_vk)?
        };

        Ok(support)
    }

    pub fn query_formats(&self) -> GpuResult<Vec<SurfaceFormat>> {
        let formats = unsafe {
            self.instance
                .get_physical_device_surface_formats(self.context.adapter.handle(), self.handle)
                .map_err(map_vk)?
        };

        Ok(formats.into_iter().map(SurfaceFormat::try_from_vk).collect::<GpuResult<Vec<_>>>()?)
    }
}

impl Drop for VkSurface {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_surface(self.handle, None);
        }
    }
}
