use contract::{GpuError, GpuResult, ffi::{GatewayBuilder, WindowHandles}};

use crate::core::{VkInstance, VkDeviceGateway};

pub struct VkGatewayBuilder {
    name: Option<String>,
    version: Option<u32>,
    engine_version: Option<u32>,
    main_window: Option<WindowHandles>,
}

impl VkGatewayBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            engine_version: None,
            main_window: None,
        }
    }
}

impl GatewayBuilder for VkGatewayBuilder {
    fn set_name(&mut self, name: &str) -> &mut dyn GatewayBuilder {
        self.name = Some(name.to_string());
        self
    }

    fn set_version(&mut self, version: u32) -> &mut dyn GatewayBuilder {
        self.version = Some(version);
        self
    }

    fn set_engine_version(&mut self, engine_version: u32) -> &mut dyn GatewayBuilder {
        self.engine_version = Some(engine_version);
        self
    }

    fn set_main_window(&mut self, window_handles: &WindowHandles) -> &mut dyn GatewayBuilder {
        self.main_window = Some(window_handles.clone());
        self
    }

    fn build(self: Box<Self>) -> GpuResult<Box<dyn contract::DeviceGateway>> {
        let name = self.name.ok_or_else(|| {
            GpuError::InitializationFailed { reason: "Name not set".to_string() }
        })?;

        let version = self.version.ok_or_else(|| {
            GpuError::InitializationFailed { reason: "Version not set".to_string() }
        })?;

        let engine_version = self.engine_version.ok_or_else(|| {
            GpuError::InitializationFailed { reason: "Engine version not set".to_string() }
        })?;

        let main_window = self.main_window.ok_or_else(|| {
            GpuError::InitializationFailed { reason: "Main window not set".to_string() }
        })?;

        let instance = VkInstance::new(name, version, engine_version, main_window)?;
        let device_gateway = VkDeviceGateway::new(instance)?;

        Ok(Box::new(device_gateway))
    }
}
