use std::{ffi::CString, sync::Arc};

use ash::vk;
use ash_window::enumerate_required_extensions;
use contract::{GpuResult, ffi::WindowHandles};

use crate::{map_ash_load, map_vk};

pub struct VkInstance {
    entry: ash::Entry,
    handle: ash::Instance,
}

impl VkInstance {
    pub fn new(
        name: String,
        version: u32,
        engine_version: u32,
        main_window: WindowHandles,
    ) -> GpuResult<Arc<Self>> {
        let entry = unsafe { ash::Entry::load().map_err(map_ash_load)? };

        let extensions =
            enumerate_required_extensions(main_window.display_handle).map_err(map_vk)?;

        let app_name = CString::new(name.clone()).unwrap();
        let engine_name = CString::new("Kraken Engine").unwrap();

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .engine_name(&engine_name)
            .application_version(version)
            .engine_version(engine_version)
            .api_version(vk::API_VERSION_1_3);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions);

        let handle = unsafe { entry.create_instance(&create_info, None).map_err(map_vk)? };

        Ok(Arc::new(Self { entry, handle }))
    }

    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn handle(&self) -> &ash::Instance {
        &self.handle
    }
}

impl Drop for VkInstance {
    fn drop(&mut self) {
        eprintln!("Dropping VkInstance with handle: {:?}", self.handle.handle());
        unsafe {
            self.handle.destroy_instance(None);
        }
    }
}
