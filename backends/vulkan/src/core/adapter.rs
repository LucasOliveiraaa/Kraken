use ash::vk;
use contract::{DeviceFeatures, DeviceType, GpuResult, MemoryHeap, MemoryHeapFlags};

use crate::{FromContract, core::{FromVk, IntoContract, QueueFamilies, VkInstance}};

impl FromVk<vk::PhysicalDeviceType> for DeviceType {
    fn from_vk(value: vk::PhysicalDeviceType) -> Self {
        match value {
            vk::PhysicalDeviceType::CPU => DeviceType::Cpu,
            vk::PhysicalDeviceType::DISCRETE_GPU => DeviceType::DiscreteGpu,
            vk::PhysicalDeviceType::INTEGRATED_GPU => DeviceType::IntegratedGpu,
            vk::PhysicalDeviceType::VIRTUAL_GPU => DeviceType::VirtualGpu,
            _ => DeviceType::Other,
        }
    }
}

impl FromVk<vk::PhysicalDeviceFeatures> for DeviceFeatures {
    fn from_vk(value: vk::PhysicalDeviceFeatures) -> Self {
        let mut features = DeviceFeatures::empty();

        if value.robust_buffer_access != 0 {
            features |= DeviceFeatures::ROBUST_BUFFER_ACCESS;
        }
        if value.full_draw_index_uint32 != 0 {
            features |= DeviceFeatures::FULL_DRAW_INDEX_UINT32;
        }
        if value.image_cube_array != 0 {
            features |= DeviceFeatures::IMAGE_CUBE_ARRAY;
        }
        if value.independent_blend != 0 {
            features |= DeviceFeatures::INDEPENDENT_BLEND;
        }
        if value.geometry_shader != 0 {
            features |= DeviceFeatures::GEOMETRY_SHADER;
        }
        if value.tessellation_shader != 0 {
            features |= DeviceFeatures::TESSELLATION_SHADER;
        }
        if value.sample_rate_shading != 0 {
            features |= DeviceFeatures::SAMPLE_RATE_SHADING;
        }
        if value.dual_src_blend != 0 {
            features |= DeviceFeatures::DUAL_SRC_BLEND;
        }
        if value.logic_op != 0 {
            features |= DeviceFeatures::LOGIC_OP;
        }
        if value.multi_draw_indirect != 0 {
            features |= DeviceFeatures::MULTI_DRAW_INDIRECT;
        }

        features
    }
}

impl FromContract<MemoryHeapFlags> for vk::MemoryHeapFlags {
    fn from_contract(value: MemoryHeapFlags) -> Self {
        let mut flags = vk::MemoryHeapFlags::empty();

        if value.contains(MemoryHeapFlags::DEVICE_LOCAL) {
            flags |= vk::MemoryHeapFlags::DEVICE_LOCAL;
        }
        if value.contains(MemoryHeapFlags::MULTI_INSTANCE) {
            flags |= vk::MemoryHeapFlags::MULTI_INSTANCE;
        }

        flags
    }
}
impl FromVk<vk::MemoryHeapFlags> for MemoryHeapFlags {
    fn from_vk(value: vk::MemoryHeapFlags) -> Self {
        let mut flags = MemoryHeapFlags::empty();

        if value.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) {
            flags |= MemoryHeapFlags::DEVICE_LOCAL;
        }
        if value.contains(vk::MemoryHeapFlags::MULTI_INSTANCE) {
            flags |= MemoryHeapFlags::MULTI_INSTANCE;
        }

        flags
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AdapterInfo {
    pub name: String,
    pub vendor_id: u32,
    pub device_type: DeviceType,
    pub features: DeviceFeatures,
    pub memory_heaps: Vec<MemoryHeap>,
}

pub struct VkAdapter {
    handle: vk::PhysicalDevice,
    queue_families: QueueFamilies,
    info: AdapterInfo,
}

impl VkAdapter {
    pub fn from_handle(instance: &VkInstance, handle: vk::PhysicalDevice) -> GpuResult<Self> {
        let vk_instance = instance.handle();
        let features = unsafe { vk_instance.get_physical_device_features(handle) };
        let props = unsafe { vk_instance.get_physical_device_properties(handle) };
        let memory = unsafe { vk_instance.get_physical_device_memory_properties(handle) };
        let queue_families = QueueFamilies::find(vk_instance, handle)?;

        let info = AdapterInfo {
            name: unsafe { std::ffi::CStr::from_ptr(props.device_name.as_ptr()) }
                .to_string_lossy()
                .into_owned(),
            vendor_id: props.vendor_id,
            device_type: props.device_type.into_contract(),
            features: features.into_contract(),
            memory_heaps: memory
                .memory_heaps
                .iter()
                .map(|heap| MemoryHeap {
                    size: heap.size,
                    flags: heap.flags.into_contract(),
                })
                .collect(),
        };

        Ok(VkAdapter {
            handle,
            queue_families: queue_families,
            info,
        })
    }

    pub fn handle(&self) -> vk::PhysicalDevice {
        self.handle
    }

    pub fn queue_families(&self) -> &QueueFamilies {
        &self.queue_families
    }

    pub fn info(&self) -> &AdapterInfo {
        &self.info
    }
}
