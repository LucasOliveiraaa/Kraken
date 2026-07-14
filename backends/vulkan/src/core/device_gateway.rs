use std::sync::{Arc, Weak};

use contract::{
    DeviceCreationDesc, DeviceGateway, DeviceId, DeviceProps, DevicePropsId, GpuError, GpuResult,
    command::CommandGateway, pipeline::PipelineGateway, presenting::PresentingGateway,
    resources::ResourcesGateway, sync::SyncGateway,
};

use crate::{
    Queues, VkAdapter, VkDevice, VkInstance, alloc::Slab, command::VkCommandGateway, map_vk,
    pipeline::VkPipelineGateway, presenting::VkPresentingGateway, resources::VkResourcesGateway,
    sync::VkSyncGateway,
};

struct AdapterPair {
    adapter: Arc<VkAdapter>,
    props: DeviceProps,
}

pub struct VkContext {
    pub instance: Arc<VkInstance>,
    pub adapter: Arc<VkAdapter>,
    pub device: VkDevice,
    pub queues: Queues,
}

pub struct StrongGateways {
    pub resources: Arc<VkResourcesGateway>,
    pub presenting: Arc<VkPresentingGateway>,
    pub sync: Arc<VkSyncGateway>,
    pub pipeline: Arc<VkPipelineGateway>,
    pub command: Arc<VkCommandGateway>,
}

#[derive(Debug, Clone)]
pub struct WeakGateways {
    pub resources: Weak<VkResourcesGateway>,
    pub presenting: Weak<VkPresentingGateway>,
    pub sync: Weak<VkSyncGateway>,
    pub pipeline: Weak<VkPipelineGateway>,
    pub command: Weak<VkCommandGateway>,
}

pub struct Device {
    pub context: Arc<VkContext>,
    pub strong_gateways: StrongGateways,
}

pub struct VkDeviceGateway {
    instance: Arc<VkInstance>,
    adapters: Vec<AdapterPair>,
    devices: parking_lot::Mutex<Slab<Device>>,
}

impl VkDeviceGateway {
    pub fn new(instance: Arc<VkInstance>) -> GpuResult<Self> {
        let physical_devices = unsafe {
            instance
                .handle()
                .enumerate_physical_devices()
                .map_err(map_vk)?
        };

        let mut adapters = Vec::new();
        for physical_device in physical_devices {
            if let Ok(adapter) = VkAdapter::from_handle(&instance, physical_device) {
                let id = adapters.len();
                adapters.push(AdapterPair {
                    props: DeviceProps {
                        id: DevicePropsId(id as u32),
                        name: Some(adapter.info().name.clone()),
                        vendor_id: adapter.info().vendor_id,
                        device_type: adapter.info().device_type,
                        features: adapter.info().features,
                        memory_heaps: adapter.info().memory_heaps.clone(),
                    },
                    adapter: Arc::new(adapter),
                });
            }
        }

        Ok(Self {
            instance,
            devices: parking_lot::Mutex::new(Slab::with_capacity(adapters.len())),
            adapters,
        })
    }
}

impl DeviceGateway for VkDeviceGateway {
    fn enumerate_devices(&self) -> Vec<DeviceProps> {
        self.adapters
            .iter()
            .map(|pair| pair.props.clone())
            .collect()
    }

    fn create_device(&self, desc: &DeviceCreationDesc) -> GpuResult<DeviceId> {
        let adapter = self
            .adapters
            .iter()
            .find(|pair| pair.props.id == desc.id)
            .ok_or(GpuError::InvalidDevicePropsId(desc.id))?
            .adapter
            .clone();

        let device = VkDevice::new(self.instance.clone(), &adapter)?;
        let queues = Queues::retrieve(&device, adapter.queue_families());

        let context = Arc::new(VkContext {
            instance: self.instance.clone(),
            adapter,
            device,
            queues,
        });

        let strong = StrongGateways {
            resources: Arc::new(VkResourcesGateway::new(context.clone())),
            presenting: Arc::new(VkPresentingGateway::new(context.clone())),
            sync: Arc::new(VkSyncGateway::new(context.clone())),
            pipeline: Arc::new(VkPipelineGateway::new(
                context.clone(),
                desc.pipeline_cache_data.as_deref(),
            )?),
            command: Arc::new(VkCommandGateway::new(context.clone())?),
        };

        let weak = WeakGateways {
            resources: Arc::downgrade(&strong.resources),
            presenting: Arc::downgrade(&strong.presenting),
            sync: Arc::downgrade(&strong.sync),
            pipeline: Arc::downgrade(&strong.pipeline),
            command: Arc::downgrade(&strong.command),
        };

        strong.presenting.set_gateways(weak.clone());
        strong.resources.set_gateways(weak.clone());
        strong.sync.set_gateways(weak.clone());
        strong.pipeline.set_gateways(weak.clone());
        strong.command.set_gateways(weak);

        let mut devices = self.devices.lock();
        let device_id = DeviceId(devices.insert(Device {
            context: context.clone(),
            strong_gateways: strong,
        }) as u32);

        Ok(device_id)
    }

    fn destroy_device(&self, device_id: DeviceId) -> GpuResult<()> {
        let mut devices = self.devices.lock();
        if devices.remove(device_id.0 as usize).is_none() {
            return Err(GpuError::InvalidDeviceId(device_id));
        }

        Ok(())
    }

    fn resources(&self, device_id: DeviceId) -> GpuResult<Arc<dyn ResourcesGateway>> {
        let devices = self.devices.lock();
        let device = devices
            .get(device_id.0 as usize)
            .ok_or(GpuError::InvalidDeviceId(device_id))?;

        Ok(device.strong_gateways.resources.clone())
    }

    fn presenting(&self, device_id: DeviceId) -> GpuResult<Arc<dyn PresentingGateway>> {
        let devices = self.devices.lock();
        let device = devices
            .get(device_id.0 as usize)
            .ok_or(GpuError::InvalidDeviceId(device_id))?;

        Ok(device.strong_gateways.presenting.clone())
    }

    fn sync(&self, device_id: DeviceId) -> GpuResult<Arc<dyn SyncGateway>> {
        let devices = self.devices.lock();
        let device = devices
            .get(device_id.0 as usize)
            .ok_or(GpuError::InvalidDeviceId(device_id))?;

        Ok(device.strong_gateways.sync.clone())
    }

    fn pipeline(&self, device_id: DeviceId) -> GpuResult<Arc<dyn PipelineGateway>> {
        let devices = self.devices.lock();
        let device = devices
            .get(device_id.0 as usize)
            .ok_or(GpuError::InvalidDeviceId(device_id))?;

        Ok(device.strong_gateways.pipeline.clone())
    }

    fn command(&self, device_id: DeviceId) -> GpuResult<Arc<dyn CommandGateway>> {
        let devices = self.devices.lock();
        let device = devices
            .get(device_id.0 as usize)
            .ok_or(GpuError::InvalidDeviceId(device_id))?;

        Ok(device.strong_gateways.command.clone())
    }
}
