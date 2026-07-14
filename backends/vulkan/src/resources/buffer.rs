use std::{mem, sync::Arc};

use ash::vk;
use contract::{
    GpuError, GpuResult,
    resources::{BufferDesc, BufferUsage, MemoryLocation},
};

use crate::{
    alloc::{AllocationDesc, VkAllocation, VkAllocator},
    core::{FromContract, FromVk, IntoVk, VkContext},
    map_vk,
};

impl FromContract<BufferUsage> for vk::BufferUsageFlags {
    fn from_contract(value: BufferUsage) -> Self {
        let mut flags = vk::BufferUsageFlags::empty();

        if value.contains(BufferUsage::TRANSFER_SRC) {
            flags |= vk::BufferUsageFlags::TRANSFER_SRC;
        }
        if value.contains(BufferUsage::TRANSFER_DST) {
            flags |= vk::BufferUsageFlags::TRANSFER_DST;
        }
        if value.contains(BufferUsage::UNIFORM) {
            flags |= vk::BufferUsageFlags::UNIFORM_BUFFER;
        }
        if value.contains(BufferUsage::STORAGE) {
            flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
        }
        if value.contains(BufferUsage::INDEX) {
            flags |= vk::BufferUsageFlags::INDEX_BUFFER;
        }
        if value.contains(BufferUsage::VERTEX) {
            flags |= vk::BufferUsageFlags::VERTEX_BUFFER;
        }

        flags
    }
}

impl FromVk<vk::BufferUsageFlags> for BufferUsage {
    fn from_vk(value: vk::BufferUsageFlags) -> Self {
        let mut flags = BufferUsage::empty();

        if value.contains(vk::BufferUsageFlags::TRANSFER_SRC) {
            flags |= BufferUsage::TRANSFER_SRC;
        }
        if value.contains(vk::BufferUsageFlags::TRANSFER_DST) {
            flags |= BufferUsage::TRANSFER_DST;
        }
        if value.contains(vk::BufferUsageFlags::UNIFORM_BUFFER) {
            flags |= BufferUsage::UNIFORM;
        }
        if value.contains(vk::BufferUsageFlags::STORAGE_BUFFER) {
            flags |= BufferUsage::STORAGE;
        }
        if value.contains(vk::BufferUsageFlags::INDEX_BUFFER) {
            flags |= BufferUsage::INDEX;
        }
        if value.contains(vk::BufferUsageFlags::VERTEX_BUFFER) {
            flags |= BufferUsage::VERTEX;
        }

        flags
    }
}

pub struct VkBuffer {
    context: Arc<VkContext>,
    handle: vk::Buffer,
    allocator: Arc<VkAllocator>,
    allocation: VkAllocation,
    desc: BufferDesc,
}

impl VkBuffer {
    pub fn new(
        context: Arc<VkContext>,
        allocator: Arc<VkAllocator>,
        desc: &BufferDesc,
    ) -> GpuResult<Self> {
        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size: desc.size,
            usage: desc.usage.into_vk(),
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
            ..Default::default()
        };

        let handle = unsafe {
            context
                .device
                .handle()
                .create_buffer(&buffer_info, None)
                .map_err(map_vk)?
        };

        let requirements = unsafe {
            context
                .device
                .handle()
                .get_buffer_memory_requirements(handle)
        };

        let mut allocation;
        unsafe {
            allocation = allocator
                .allocate(AllocationDesc {
                    name: &desc.name,
                    linear: true,
                    location: desc.location,
                    requirements,
                })
                .map_err(|e| {
                    context.device.handle().destroy_buffer(handle, None);
                    e
                })?;

            context
                .device
                .handle()
                .bind_buffer_memory(handle, allocation.memory(), allocation.offset())
                .map_err(|e| {
                    allocator.deallocate(mem::take(&mut allocation)).ok();
                    context.device.handle().destroy_buffer(handle, None);
                    map_vk(e)
                })?;
        }

        Ok(Self {
            context,
            handle,
            allocator,
            allocation,
            desc: desc.clone(),
        })
    }

    pub fn handle(&self) -> vk::Buffer {
        self.handle
    }

    pub fn desc(&self) -> &BufferDesc {
        &self.desc
    }

    fn map_memory(&self) -> GpuResult<*mut u8> {
        if self.desc.location == MemoryLocation::GpuOnly {
            return Err(GpuError::InvalidOperation {
                reason: "Cannot map a GPU-only buffer".to_string(),
            });
        }

        let mapped_data = unsafe {
            self.context
                .device
                .handle()
                .map_memory(
                    self.allocation.memory(),
                    self.allocation.offset(),
                    self.desc.size,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(map_vk)?
        };

        Ok(mapped_data as *mut u8)
    }

    fn unmap_memory(&self) {
        unsafe {
            self.context
                .device
                .handle()
                .unmap_memory(self.allocation.memory());
        }
    }

    pub fn write(&self, offset: u64, data: &[u8]) -> GpuResult<()> {
        if self.desc.size < offset + data.len() as u64 {
            return Err(GpuError::InvalidOperation {
                reason: "Write operation exceeds buffer size".to_string(),
            });
        }

        let mapped_data = self.map_memory()?;

        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                mapped_data.add(offset as usize) as *mut u8,
                data.len(),
            );
        }

        self.unmap_memory();

        Ok(())
    }

    pub fn read(&self, offset: u64, data: &mut [u8]) -> GpuResult<()> {
        if self.desc.size < offset + data.len() as u64 {
            return Err(GpuError::InvalidOperation {
                reason: "Read operation exceeds buffer size".to_string(),
            });
        }

        let mapped_data = self.map_memory()?;

        unsafe {
            std::ptr::copy_nonoverlapping(
                mapped_data.add(offset as usize) as *const u8,
                data.as_mut_ptr(),
                data.len(),
            );
        }

        self.unmap_memory();

        Ok(())
    }
}

impl Drop for VkBuffer {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .handle()
                .destroy_buffer(self.handle, None);
        }
        self.allocator
            .deallocate(mem::take(&mut self.allocation))
            .ok();
    }
}
