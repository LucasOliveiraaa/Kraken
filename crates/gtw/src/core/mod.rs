mod device;

pub use device::*;

use std::sync::Arc;

use contract::GpuResult;
pub use contract::{DeviceCreationDesc, DeviceProps, DeviceFeatures, DeviceType, MemoryHeap, MemoryHeapFlags};

use crate::macros::GatewayWrapper;

pub struct DeviceGateway {
    _lib: libloading::Library,
    raw: Box<dyn contract::DeviceGateway>,
}

impl GatewayWrapper for DeviceGateway {
    type Raw = dyn contract::DeviceGateway;
}

impl DeviceGateway {
    pub fn from_raw(_lib: libloading::Library, raw: Box<dyn contract::DeviceGateway>) -> Arc<Self> {
        Arc::new(Self { _lib, raw })
    }

    pub(crate) fn raw(&self) -> &dyn contract::DeviceGateway {
        self.raw.as_ref()
    }

    /// Enumerate all available devices and return their properties.
    pub fn enumerate_devices(&self) -> Vec<DeviceProps> {
        self.raw.enumerate_devices()
    }

    /// Create a new device given its properties ID.
    pub fn create_device(self: &Arc<Self>, desc: &DeviceCreationDesc) -> GpuResult<Device> {
        let handle = self.raw.create_device(desc)?;

        Device::from_handle(self.clone(), handle)
    }
}
