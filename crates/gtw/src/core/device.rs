use std::sync::Arc;

use contract::{DeviceId, GpuResult};

use crate::{
    DeviceGateway, command::CommandGateway, pipeline::PipelineGateway,
    presenting::PresentingGateway, resources::ResourcesGateway, sync::SyncGateway,
};

struct InnerDevice {
    device_gateway: Arc<DeviceGateway>,
    handle: DeviceId,

    resources_gtw: Arc<dyn contract::resources::ResourcesGateway>,
    presenting_gtw: Arc<dyn contract::presenting::PresentingGateway>,
    sync_gtw: Arc<dyn contract::sync::SyncGateway>,
    pipeline_gtw: Arc<dyn contract::pipeline::PipelineGateway>,
    command_gtw: Arc<dyn contract::command::CommandGateway>,
}

#[derive(Clone)]
pub struct Device {
    inner: Arc<InnerDevice>,
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("handle", &self.inner.handle)
            .finish()
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for Device {}
impl std::hash::Hash for Device {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::sync::Arc::as_ptr(&self.inner).hash(state);
    }
}

impl Device {
    pub fn from_handle(
        device_gateway: Arc<DeviceGateway>,
        handle: DeviceId,
    ) -> GpuResult<Self> {
        Ok(Self {
            inner: Arc::new(InnerDevice {
                device_gateway: device_gateway.clone(),
                handle,
                resources_gtw: device_gateway.raw().resources(handle)?,
                presenting_gtw: device_gateway.raw().presenting(handle)?,
                sync_gtw: device_gateway.raw().sync(handle)?,
                pipeline_gtw: device_gateway.raw().pipeline(handle)?,
                command_gtw: device_gateway.raw().command(handle)?,
            }),
        })
    }

    pub(crate) fn handle(&self) -> DeviceId {
        self.inner.handle
    }

    /// Returns a resources gateway for this device.
    ///
    /// The Resources Gateway is used to create and manage GPU resources such as buffers, images, and samplers.
    ///
    /// # Returns
    /// A `ResourcesGateway` instance for this device.
    ///
    /// # Examples
    /// ```ignore
    /// let buffer = device.resources().create_buffer(desc)?;
    /// ```
    pub fn resources(&self) -> ResourcesGateway {
        ResourcesGateway::from_raw(self.clone(), self.inner.resources_gtw.clone())
    }

    /// Returns a presenting gateway for this device.
    ///
    /// The Presenting Gateway is used to create swapchains and manage the presentation
    /// of rendered images to the screen.
    ///
    /// # Returns
    /// A `PresentingGateway` instance for this device.
    ///
    /// # Examples
    /// ```ignore
    /// let swapchain = device.presenting().create_swapchain(desc)?;
    /// ```
    pub fn presenting(&self) -> PresentingGateway {
        PresentingGateway::from_raw(self.clone(), self.inner.presenting_gtw.clone())
    }

    /// Returns a sync gateway for this device.
    ///
    /// The Sync Gateway is used to create fences and semaphores,
    /// which are used to synchronize GPU operations and coordinate between the CPU and GPU.
    ///
    /// # Returns
    /// A `SyncGateway` instance for this device.
    ///
    /// # Examples
    /// ```ignore
    /// let fence = device.sync().create_fence()?;
    /// ```
    pub fn sync(&self) -> SyncGateway {
        SyncGateway::from_raw(self.clone(), self.inner.sync_gtw.clone())
    }

    /// Returns a pipeline gateway for this device.
    ///
    /// The Pipeline Gateway is used to create graphics and compute pipelines,
    /// which are used to configure the GPU for rendering and compute operations.
    ///
    /// # Returns
    /// A `PipelineGateway` instance for this device.
    ///
    /// # Examples
    /// ```ignore
    /// let graphics_pipelines = device.pipeline().create_graphics_pipelines(descs)?;
    /// ```
    pub fn pipeline(&self) -> PipelineGateway {
        PipelineGateway::from_raw(self.clone(), self.inner.pipeline_gtw.clone())
    }

    /// Returns a command gateway for this device.
    ///
    /// The Command Gateway is used to create command pools and command buffers,
    /// which are used to record and submit commands to the GPU.
    ///
    /// # Returns
    /// A `CommandGateway` instance for this device.
    ///
    /// # Examples
    /// ```ignore
    /// let command_buffers = device.command().alloc_command_buffers(pool, 1)?;
    /// ```
    pub fn command(&self) -> CommandGateway {
        CommandGateway::from_raw(self.clone(), self.inner.command_gtw.clone())
    }
}

impl Drop for InnerDevice {
    fn drop(&mut self) {
        let _ = self.device_gateway.raw().destroy_device(self.handle);
    }
}
