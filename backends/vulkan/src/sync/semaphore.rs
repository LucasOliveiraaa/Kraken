use std::sync::Arc;

use ash::vk;
use contract::GpuResult;

use crate::{core::VkContext, map_vk};

pub struct VkSemaphore {
    context: Arc<VkContext>,
    handle: vk::Semaphore,
}

impl VkSemaphore {
    pub fn new(context: Arc<VkContext>) -> GpuResult<Self> {
        let semaphore_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
            ..Default::default()
        };

        let handle = unsafe {
            context
                .device
                .handle()
                .create_semaphore(&semaphore_info, None)
                .map_err(map_vk)?
        };

        Ok(Self { context, handle })
    }

    pub fn handle(&self) -> vk::Semaphore {
        self.handle
    }
}

impl Drop for VkSemaphore {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .handle()
                .destroy_semaphore(self.handle, None);
        }
    }
}
