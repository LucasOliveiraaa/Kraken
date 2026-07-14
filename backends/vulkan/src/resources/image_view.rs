use std::sync::Arc;

use ash::vk;
use contract::{
    GpuError, GpuResult,
    resources::{ImageAspect, ImageViewDesc, ImageViewType},
};

use crate::{
    TryFromVk,
    command::ImageRef,
    core::{FromContract, FromVk, IntoVk, VkContext},
    map_vk,
};

impl FromContract<ImageViewType> for vk::ImageViewType {
    fn from_contract(value: ImageViewType) -> Self {
        match value {
            ImageViewType::Image1D => vk::ImageViewType::TYPE_1D,
            ImageViewType::Image2D => vk::ImageViewType::TYPE_2D,
            ImageViewType::Image3D => vk::ImageViewType::TYPE_3D,
        }
    }
}
impl TryFromVk<vk::ImageViewType> for ImageViewType {
    fn try_from_vk(value: vk::ImageViewType) -> GpuResult<Self> {
        match value {
            vk::ImageViewType::TYPE_1D => Ok(ImageViewType::Image1D),
            vk::ImageViewType::TYPE_2D => Ok(ImageViewType::Image2D),
            vk::ImageViewType::TYPE_3D => Ok(ImageViewType::Image3D),
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("ImageViewType {:?}", value),
            }),
        }
    }
}

impl FromContract<ImageAspect> for vk::ImageAspectFlags {
    fn from_contract(value: ImageAspect) -> Self {
        let mut flags = vk::ImageAspectFlags::empty();
        if value.contains(ImageAspect::COLOR) {
            flags |= vk::ImageAspectFlags::COLOR;
        }
        if value.contains(ImageAspect::DEPTH) {
            flags |= vk::ImageAspectFlags::DEPTH;
        }
        if value.contains(ImageAspect::STENCIL) {
            flags |= vk::ImageAspectFlags::STENCIL;
        }
        flags
    }
}
impl FromVk<vk::ImageAspectFlags> for ImageAspect {
    fn from_vk(value: vk::ImageAspectFlags) -> Self {
        let mut aspect = ImageAspect::empty();
        if value.contains(vk::ImageAspectFlags::COLOR) {
            aspect |= ImageAspect::COLOR;
        }
        if value.contains(vk::ImageAspectFlags::DEPTH) {
            aspect |= ImageAspect::DEPTH;
        }
        if value.contains(vk::ImageAspectFlags::STENCIL) {
            aspect |= ImageAspect::STENCIL;
        }
        aspect
    }
}

pub struct VkImageView {
    context: Arc<VkContext>,
    handle: vk::ImageView,
    _image: ImageRef,
}

impl VkImageView {
    pub fn new(context: Arc<VkContext>, image: ImageRef, desc: &ImageViewDesc) -> GpuResult<Self> {
        let view_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            image: image.handle(),
            format: desc.format.into_vk(),
            view_type: desc.view_type.into_vk(),
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: desc.aspect.into_vk(),
                base_mip_level: desc.base_mip,
                level_count: desc.level_count,
                base_array_layer: desc.base_layer,
                layer_count: desc.layer_count,
            },
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            ..Default::default()
        };

        let handle = unsafe {
            context
                .device
                .handle()
                .create_image_view(&view_info, None)
                .map_err(map_vk)?
        };

        Ok(Self {
            context,
            handle,
            _image: image,
        })
    }

    pub fn handle(&self) -> vk::ImageView {
        self.handle
    }
}

impl Drop for VkImageView {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .handle()
                .destroy_image_view(self.handle, None);
        }
    }
}
