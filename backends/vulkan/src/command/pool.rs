use std::sync::Arc;

use ash::vk;
use contract::{
    GpuResult,
    command::{CommandBufferLifetime, CommandPoolResetMode},
};

use crate::{VkContext, VkQueue, command::VkCommandBuffer, map_vk};

pub struct VkCommandPoolDesc {
    pub queue: Arc<VkQueue>,
    pub reset_mode: CommandPoolResetMode,
    pub buffer_lifetime: CommandBufferLifetime,
}

pub struct VkCommandPool {
    context: Arc<VkContext>,
    handle: vk::CommandPool,
}

impl VkCommandPool {
    pub fn new(context: Arc<VkContext>, desc: VkCommandPoolDesc) -> GpuResult<Self> {
        let mut flags = vk::CommandPoolCreateFlags::empty();
        if desc.reset_mode == CommandPoolResetMode::ResetIndividually {
            flags |= vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        }
        if desc.buffer_lifetime == CommandBufferLifetime::OneTimeSubmit {
            flags |= vk::CommandPoolCreateFlags::TRANSIENT;
        }

        let pool_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: flags,
            queue_family_index: desc.queue.family(),
            ..Default::default()
        };

        let handle = unsafe {
            context
                .device
                .handle()
                .create_command_pool(&pool_info, None)
                .map_err(map_vk)?
        };

        Ok(Self { context, handle })
    }

    pub fn handle(&self) -> vk::CommandPool {
        self.handle
    }

    pub fn reset(&self) -> GpuResult<()> {
        unsafe {
            self.context
                .device
                .handle()
                .reset_command_pool(self.handle, vk::CommandPoolResetFlags::empty())
                .map_err(map_vk)
        }
    }

    pub fn free_command_buffers(&self, command_buffers: &[Arc<VkCommandBuffer>]) {
        let command_buffers = command_buffers
            .iter()
            .map(|cb| cb.handle())
            .collect::<Vec<_>>();

        unsafe {
            self.context
                .device
                .handle()
                .free_command_buffers(self.handle, &command_buffers);
        }
    }

    pub fn reset_command_buffer(&self, command_buffer: Arc<VkCommandBuffer>) -> GpuResult<()> {
        unsafe {
            self.context
                .device
                .handle()
                .reset_command_buffer(
                    command_buffer.handle(),
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .map_err(map_vk)
        }
    }
}

impl Drop for VkCommandPool {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .handle()
                .destroy_command_pool(self.handle, None);
        }
    }
}
