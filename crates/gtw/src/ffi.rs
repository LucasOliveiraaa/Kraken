use std::sync::Arc;

use contract::{
    GpuError, GpuResult, ffi::{GATEWAY_ENTRYPOINT_NAME, GatewayBuilderEntrypoint, WindowHandles},
};
use libloading::{Library, Symbol};

use crate::DeviceGateway;

fn map_libloading(err: libloading::Error) -> GpuError {
    match err {
        libloading::Error::DlOpen { source } => GpuError::LibraryLoadError {
            reason: source.to_string(),
        },
        libloading::Error::DlSym { source } => GpuError::LibraryLoadError {
            reason: source.to_string(),
        },
        _ => GpuError::LibraryLoadError {
            reason: format!("Unknown error: {:?}", err),
        },
    }
}

pub struct GatewayBuilder {
    lib: Library,
    raw: Box<dyn contract::ffi::GatewayBuilder>,
}

impl GatewayBuilder {
    pub fn load(path: &str) -> GpuResult<Self> {
        let lib = unsafe { Library::new(path).map_err(map_libloading)? };

        let gateway_entrypoint: Symbol<GatewayBuilderEntrypoint> =
            unsafe { lib.get(GATEWAY_ENTRYPOINT_NAME).map_err(map_libloading)? };

        let raw = unsafe { Box::from_raw(gateway_entrypoint()) };

        Ok(Self { lib, raw })
    }

    pub fn set_name(mut self, name: &str) -> Self {
        self.raw.set_name(name);
        self
    }
    pub fn set_version(mut self, version: u32) -> Self {
        self.raw.set_version(version);
        self
    }
    pub fn set_engine_version(mut self, engine_version: u32) -> Self {
        self.raw.set_engine_version(engine_version);
        self
    }
    pub fn set_main_window(mut self, window: &WindowHandles) -> Self {
        self.raw.set_main_window(window);
        self
    }

    pub fn build(self) -> GpuResult<Arc<DeviceGateway>> {
        let raw = self.raw.build()?;

        Ok(DeviceGateway::from_raw(self.lib, raw))
    }
}
