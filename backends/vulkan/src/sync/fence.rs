use std::sync::Arc;

use ash::vk;
use contract::{GpuResult, sync::FenceState};

use crate::{core::VkContext, map_vk};

pub struct VkFence {
    context: Arc<VkContext>,
    handle: vk::Fence,
}

impl VkFence {
    pub fn new(context: Arc<VkContext>) -> GpuResult<Self> {
        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::FenceCreateFlags::empty(),
            ..Default::default()
        };

        let handle = unsafe {
            context
                .device
                .handle()
                .create_fence(&fence_info, None)
                .map_err(map_vk)?
        };

        Ok(Self { context, handle })
    }

    pub fn handle(&self) -> vk::Fence {
        self.handle
    }

    pub fn wait(
        context: Arc<VkContext>,
        fences: &[Arc<VkFence>],
        wait_for_all: bool,
        timeout: Option<u64>,
    ) -> GpuResult<()> {
        let fence_handles: Vec<vk::Fence> = fences.iter().map(|f| f.handle()).collect();
        unsafe {
            context
                .device
                .handle()
                .wait_for_fences(&fence_handles, wait_for_all, timeout.unwrap_or(u64::MAX))
                .map_err(map_vk)?;
        }
        Ok(())
    }

    pub fn reset(context: Arc<VkContext>, fences: &[Arc<VkFence>]) -> GpuResult<()> {
        let fence_handles: Vec<vk::Fence> = fences.iter().map(|f| f.handle()).collect();
        unsafe {
            context
                .device
                .handle()
                .reset_fences(&fence_handles)
                .map_err(map_vk)?;
        }
        Ok(())
    }

    pub fn get_state(&self) -> GpuResult<FenceState> {
        let status = unsafe {
            self.context
                .device
                .handle()
                .get_fence_status(self.handle)
                .map_err(map_vk)?
        };

        if status {
            Ok(FenceState::Signaled)
        } else {
            Ok(FenceState::Unsignaled)
        }
    }
}

impl Drop for VkFence {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .handle()
                .destroy_fence(self.handle, None);
        }
    }
}
