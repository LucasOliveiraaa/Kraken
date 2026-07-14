use std::sync::Arc;

use ash::vk;
use contract::GpuResult;

use crate::{command::VkCommandPool, map_vk, VkContext};

mod barrier;
mod convert;
mod descriptor;
mod state;
mod copy;

pub use copy::ImageRef;
pub use state::BoundPipeline;

pub struct VkCommandBuffer {
    context: Arc<VkContext>,
    command_pool: Arc<VkCommandPool>,
    handle: vk::CommandBuffer,

    current_bound_pipeline: parking_lot::Mutex<Option<BoundPipeline>>,
    // The most-recently committed/materialized descriptor sets for this
    // command buffer (one per set index). Stored as raw `vk::DescriptorSet`
    // handles so they can be rebound into the command buffer later when
    // recording draw/dispatch calls.
    current_bound_descriptor_sets: parking_lot::Mutex<Vec<vk::DescriptorSet>>,
}

impl VkCommandBuffer {
    pub fn various(
        context: Arc<VkContext>,
        command_pool: Arc<VkCommandPool>,
        count: u32,
    ) -> GpuResult<Vec<Self>> {
        let alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            command_buffer_count: count,
            command_pool: command_pool.handle(),
            level: vk::CommandBufferLevel::PRIMARY,
            ..Default::default()
        };

        let handles = unsafe {
            context
                .device
                .handle()
                .allocate_command_buffers(&alloc_info)
                .map_err(map_vk)?
        };

        Ok(handles
            .into_iter()
            .map(|handle| Self::from_handle(context.clone(), command_pool.clone(), handle))
            .collect::<Vec<_>>())
    }

    pub fn from_handle(
        context: Arc<VkContext>,
        command_pool: Arc<VkCommandPool>,
        handle: vk::CommandBuffer,
    ) -> Self {
        Self {
            context,
            command_pool,
            handle,

            current_bound_pipeline: parking_lot::Mutex::new(None),
            current_bound_descriptor_sets: parking_lot::Mutex::new(Vec::new()),
        }
    }

    pub fn pool(&self) -> Arc<VkCommandPool> {
        self.command_pool.clone()
    }

    pub fn handle(&self) -> vk::CommandBuffer {
        self.handle
    }

    pub fn begin(&self) -> GpuResult<()> {
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: std::ptr::null(),
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            p_inheritance_info: std::ptr::null(),
            ..Default::default()
        };

        unsafe {
            self.context
                .device
                .handle()
                .begin_command_buffer(self.handle, &begin_info)
                .map_err(map_vk)
        }
    }

    pub fn end(&self) -> GpuResult<()> {
        unsafe {
            self.context
                .device
                .handle()
                .end_command_buffer(self.handle)
                .map_err(map_vk)
        }
    }
}