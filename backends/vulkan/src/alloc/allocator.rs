use std::sync::Arc;

use ash::vk;
use contract::{GpuResult, resources::MemoryLocation};
use gpu_allocator::vulkan::{
    AllocationCreateDesc, AllocationScheme, Allocator, AllocatorCreateDesc,
};

use crate::{
    core::{FromVk, VkContext},
    map_alloc,
};

pub struct AllocatorDesc {}

impl FromVk<MemoryLocation> for vk::MemoryPropertyFlags {
    fn from_vk(value: MemoryLocation) -> Self {
        match value {
            MemoryLocation::GpuOnly => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            MemoryLocation::CpuToGpu => {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            }
            MemoryLocation::GpuToCpu => {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            }
        }
    }
}

fn into_gpu_allocator(value: MemoryLocation) -> gpu_allocator::MemoryLocation {
    match value {
        MemoryLocation::GpuOnly => gpu_allocator::MemoryLocation::GpuOnly,
        MemoryLocation::CpuToGpu => gpu_allocator::MemoryLocation::CpuToGpu,
        MemoryLocation::GpuToCpu => gpu_allocator::MemoryLocation::GpuToCpu,
    }
}

pub struct AllocationDesc<'a> {
    pub name: &'a str,
    pub linear: bool,
    pub location: MemoryLocation,
    pub requirements: vk::MemoryRequirements,
}

pub struct VkAllocator {
    _context: Arc<VkContext>,
    alloc: parking_lot::Mutex<Allocator>,
}

impl VkAllocator {
    pub fn new(context: Arc<VkContext>, _desc: AllocatorDesc) -> Self {
        let alloc = Allocator::new(&AllocatorCreateDesc {
            instance: context.instance.handle().clone(),
            device: context.device.handle().clone(),
            physical_device: context.adapter.handle(),
            debug_settings: Default::default(),
            buffer_device_address: false,
            allocation_sizes: Default::default(),
        })
        .unwrap();

        Self {
            _context: context,
            alloc: parking_lot::Mutex::new(alloc),
        }
    }

    pub fn allocate(&self, desc: AllocationDesc) -> GpuResult<VkAllocation> {
        let mut allocator_guard = self.alloc.lock();

        let allocation = allocator_guard
            .allocate(&AllocationCreateDesc {
                name: desc.name,
                linear: desc.linear,
                location: into_gpu_allocator(desc.location),
                requirements: desc.requirements,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            })
            .map_err(map_alloc)?;

        Ok(VkAllocation { allocation })
    }

    pub fn deallocate(&self, allocation: VkAllocation) -> GpuResult<()> {
        let mut allocator_guard = self.alloc.lock();
        allocator_guard
            .free(allocation.allocation)
            .map_err(map_alloc)?;

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct VkAllocation {
    allocation: gpu_allocator::vulkan::Allocation,
}

impl VkAllocation {
    pub fn memory(&self) -> vk::DeviceMemory {
        unsafe { self.allocation.memory() }
    }

    pub fn offset(&self) -> vk::DeviceSize {
        self.allocation.offset()
    }
}
