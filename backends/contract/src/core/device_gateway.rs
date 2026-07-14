use std::sync::Arc;

use crate::{
    DeviceCreationDesc, DeviceId, DeviceProps, GpuResult, command::CommandGateway, pipeline::PipelineGateway, presenting::PresentingGateway, resources::ResourcesGateway, sync::SyncGateway,
};

pub trait DeviceGateway {
    /// Enumerate all available devices and return their properties.
    fn enumerate_devices(&self) -> Vec<DeviceProps>;

    /// Create a new device given its properties ID.
    fn create_device(&self, desc: &DeviceCreationDesc) -> GpuResult<DeviceId>;

    /// Destroy a device given its ID.
    fn destroy_device(&self, device_id: DeviceId) -> GpuResult<()>;

    fn resources(&self, device_id: DeviceId) -> GpuResult<Arc<dyn ResourcesGateway>>;
    fn presenting(&self, device_id: DeviceId) -> GpuResult<Arc<dyn PresentingGateway>>;
    fn sync(&self, device_id: DeviceId) -> GpuResult<Arc<dyn SyncGateway>>;
    fn pipeline(&self, device_id: DeviceId) -> GpuResult<Arc<dyn PipelineGateway>>;
    fn command(&self, device_id: DeviceId) -> GpuResult<Arc<dyn CommandGateway>>;
}
