use std::{ptr, sync::Arc};

use ash::vk;
use contract::GpuResult;

use crate::{VkInstance, core::VkAdapter, map_vk};

pub struct VkDevice {
    instance: Arc<VkInstance>,
    handle: ash::Device,
}

impl VkDevice {
    pub fn new(instance: Arc<VkInstance>, adapter: &VkAdapter) -> GpuResult<Self> {
        let priority = 1.0f32;

        let families = adapter.queue_families();
        let mut unique_families = vec![families.graphics, families.compute, families.transfer];
        unique_families.sort();
        unique_families.dedup();

        let queue_create_infos = unique_families
            .iter()
            .map(|&family| vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: family,
                queue_count: 1,
                p_queue_priorities: &priority,
                ..Default::default()
            })
            .collect::<Vec<vk::DeviceQueueCreateInfo>>();

        let desired_extensions = [ash::khr::swapchain::NAME.as_ptr()];

        let device_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            pp_enabled_extension_names: desired_extensions.as_ptr(),
            enabled_extension_count: desired_extensions.len() as u32,
            ..Default::default()
        };

        let handle = unsafe {
            instance
                .handle()
                .create_device(adapter.handle(), &device_info, None)
                .map_err(map_vk)?
        };

        eprintln!("Created VkDevice: {:?}", handle.handle());

        Ok(Self { instance, handle })
    }

    pub fn handle(&self) -> &ash::Device {
        &self.handle
    }
}

impl Drop for VkDevice {
    fn drop(&mut self) {
        eprintln!("Dropping VkDevice: {:?}", self.handle.handle());
        unsafe {
            self.handle.destroy_device(None);
        }
    }
}
