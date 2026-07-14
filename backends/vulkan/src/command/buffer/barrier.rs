use ash::vk;
use contract::{command::Barriers, GpuResult};

use crate::{resources::VkResourcesGateway, IntoVk};

use super::VkCommandBuffer;

impl VkCommandBuffer {
    pub fn pipeline_barrier(
        &self,
        resources_gateway: &VkResourcesGateway,
        barriers: &Barriers,
    ) -> GpuResult<()> {
        let memory_barriers = barriers
            .memory
            .iter()
            .map(|barrier| vk::MemoryBarrier2 {
                s_type: vk::StructureType::MEMORY_BARRIER_2,
                p_next: std::ptr::null(),
                src_stage_mask: barrier.src_stages.into_vk(),
                src_access_mask: barrier.src_access.into_vk(),
                dst_stage_mask: barrier.dst_stages.into_vk(),
                dst_access_mask: barrier.dst_access.into_vk(),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let buffer_barriers = barriers
            .buffers
            .iter()
            .map(|barrier| {
                let buffer = resources_gateway.get_buffer(barrier.buffer)?;
                Ok(vk::BufferMemoryBarrier2 {
                    s_type: vk::StructureType::BUFFER_MEMORY_BARRIER_2,
                    p_next: std::ptr::null(),
                    src_stage_mask: barrier.src_stages.into_vk(),
                    src_access_mask: barrier.src_access.into_vk(),
                    dst_stage_mask: barrier.dst_stages.into_vk(),
                    dst_access_mask: barrier.dst_access.into_vk(),
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    buffer: buffer.handle(),
                    offset: barrier.offset as u64,
                    size: barrier.size as u64,
                    ..Default::default()
                })
            })
            .collect::<GpuResult<Vec<_>>>()?;

        let image_barriers = barriers
            .images
            .iter()
            .map(|barrier| {
                let image = resources_gateway.get_image(barrier.image)?;
                Ok(vk::ImageMemoryBarrier2 {
                    s_type: vk::StructureType::IMAGE_MEMORY_BARRIER_2,
                    p_next: std::ptr::null(),
                    src_stage_mask: barrier.src_stages.into_vk(),
                    src_access_mask: barrier.src_access.into_vk(),
                    dst_stage_mask: barrier.dst_stages.into_vk(),
                    dst_access_mask: barrier.dst_access.into_vk(),
                    old_layout: barrier.old_layout.into_vk(),
                    new_layout: barrier.new_layout.into_vk(),
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    image: image.handle(),
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: barrier.subresource_range.aspect_mask.into_vk(),
                        base_mip_level: barrier.subresource_range.base_mip,
                        level_count: barrier.subresource_range.level_count,
                        base_array_layer: barrier.subresource_range.base_array,
                        layer_count: barrier.subresource_range.layer_count,
                    },
                    ..Default::default()
                })
            })
            .collect::<GpuResult<Vec<_>>>()?;

        unsafe {
            self.context.device.handle().cmd_pipeline_barrier2(
                self.handle,
                &vk::DependencyInfo {
                    s_type: vk::StructureType::DEPENDENCY_INFO,
                    p_next: std::ptr::null(),
                    dependency_flags: vk::DependencyFlags::empty(),
                    memory_barrier_count: memory_barriers.len() as u32,
                    p_memory_barriers: memory_barriers.as_ptr(),
                    buffer_memory_barrier_count: buffer_barriers.len() as u32,
                    p_buffer_memory_barriers: buffer_barriers.as_ptr(),
                    image_memory_barrier_count: image_barriers.len() as u32,
                    p_image_memory_barriers: image_barriers.as_ptr(),
                    ..Default::default()
                },
            );
        }

        Ok(())
    }
}