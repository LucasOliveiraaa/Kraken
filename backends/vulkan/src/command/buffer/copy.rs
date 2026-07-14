use std::sync::Arc;

use ash::vk;
use contract::{
    GpuResult,
    command::ImageBlitRegion,
    resources::{Filter, ImageIdRef, ImageLayout},
};

use crate::{IntoVk, command::VkCommandBuffer, presenting::{VkPresentingGateway, VkSwapchainImage}, resources::{VkBuffer, VkImage, VkResourcesGateway}};

pub enum ImageRef {
    Image(Arc<VkImage>),
    SwapchainImage(Arc<VkSwapchainImage>),
}

impl ImageRef {
    pub fn handle(&self) -> vk::Image {
        match self {
            ImageRef::Image(image) => image.handle(),
            ImageRef::SwapchainImage(swapchain_image) => swapchain_image.handle(),
        }
    }

    pub fn from_id_ref(
        id_ref: ImageIdRef,
        resources_gateway: &VkResourcesGateway,
        presenting_gateway: &VkPresentingGateway,
    ) -> GpuResult<Self> {
        match id_ref {
            ImageIdRef::Image(image_id) => {
                Ok(ImageRef::Image(resources_gateway.get_image(image_id)?))
            }
            ImageIdRef::SwapchainImage(swapchain_image_id) => Ok(ImageRef::SwapchainImage(
                presenting_gateway.get_swapchain_image(swapchain_image_id)?,
            )),
        }
    }
}

impl VkCommandBuffer {
    pub fn blit(
        &self,
        src_image: ImageRef,
        src_layout: ImageLayout,
        dst_image: ImageRef,
        dst_layout: ImageLayout,
        regions: &[ImageBlitRegion],
        filter: Filter,
    ) -> GpuResult<()> {
        let regions = regions
            .iter()
            .map(|region| vk::ImageBlit2 {
                s_type: vk::StructureType::IMAGE_BLIT_2,
                p_next: std::ptr::null(),
                src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: region.src_subresource.aspect_mask.into_vk(),
                    mip_level: region.src_subresource.base_mip,
                    base_array_layer: region.src_subresource.base_array,
                    layer_count: region.src_subresource.layer_count,
                },
                src_offsets: [
                    region.src_offsets[0].into_vk(),
                    region.src_offsets[1].into_vk(),
                ],
                dst_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: region.dst_subresource.aspect_mask.into_vk(),
                    mip_level: region.dst_subresource.base_mip,
                    base_array_layer: region.dst_subresource.base_array,
                    layer_count: region.dst_subresource.layer_count,
                },
                dst_offsets: [
                    region.dst_offsets[0].into_vk(),
                    region.dst_offsets[1].into_vk(),
                ],
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let blit_info = vk::BlitImageInfo2 {
            s_type: vk::StructureType::BLIT_IMAGE_INFO_2,
            p_next: std::ptr::null(),
            src_image: src_image.handle(),
            src_image_layout: src_layout.into_vk(),
            dst_image: dst_image.handle(),
            dst_image_layout: dst_layout.into_vk(),
            region_count: regions.len() as u32,
            p_regions: regions.as_ptr(),
            filter: filter.into_vk(),
            ..Default::default()
        };

        unsafe {
            self.context
                .device
                .handle()
                .cmd_blit_image2(self.handle, &blit_info);
        }

        Ok(())
    }

    pub fn fill_buffer(&self, buffer: Arc<VkBuffer>, offset: u64, size: u64, value: u8) -> GpuResult<()> {
        unsafe {
            self.context
                .device
                .handle()
                .cmd_fill_buffer(self.handle, buffer.handle(), offset, size, value as u32);
        }

        Ok(())
    }
}
