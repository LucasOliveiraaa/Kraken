use std::{collections::HashMap, sync::Arc};

use ash::vk;
use contract::GpuResult;

use crate::{
    TryIntoContract, VkContext,
    command::descriptor::VkDescriptorSet,
    map_vk,
    pipeline::{DescriptorType, VkDescriptorSetLayout},
};

pub struct AllocationRequest {
    pub set_layout: Arc<VkDescriptorSetLayout>,
    pub count: HashMap<DescriptorType, u32>,
}

pub struct VkDescriptorPool {
    context: Arc<VkContext>,
    handle: vk::DescriptorPool,
    available_sets: u32,
    available_descriptors: HashMap<DescriptorType, u32>,
}

impl VkDescriptorPool {
    pub fn new(context: Arc<VkContext>) -> GpuResult<Self> {
        let sizes = vec![
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLER,
                descriptor_count: 100,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 100,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLED_IMAGE,
                descriptor_count: 100,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: 100,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 100,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 100,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
                descriptor_count: 100,
            },
        ];

        let pool_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET,
            pool_size_count: sizes.len() as u32,
            p_pool_sizes: sizes.as_ptr(),
            max_sets: 100,
            ..Default::default()
        };

        let handle = unsafe {
            context
                .device
                .handle()
                .create_descriptor_pool(&pool_info, None)
                .map_err(map_vk)?
        };

        Ok(Self {
            context,
            handle,
            available_sets: pool_info.max_sets,
            available_descriptors: sizes
                .iter()
                .map(|size| Ok((size.ty.try_into_contract()?, size.descriptor_count)))
                .collect::<GpuResult<HashMap<_, _>>>()?,
        })
    }

    pub fn handle(&self) -> vk::DescriptorPool {
        self.handle
    }

    pub fn is_full(&self) -> bool {
        self.available_sets == 0 || self.available_descriptors.values().all(|&count| count == 0)
    }

    pub fn can_allocate(&self, requests: &[AllocationRequest]) -> bool {
        if self.available_sets == 1 {
            return false;
        }

        for request in requests {
            for (ty, count) in &request.count {
                let available = self.available_descriptors.get(ty).unwrap_or(&0);
                if *available < *count {
                    return false;
                }
            }
        }

        true
    }

    pub fn allocate_sets_unchecked(
        &mut self,
        requests: &[AllocationRequest],
    ) -> GpuResult<Vec<VkDescriptorSet>> {
        let mut required_descriptors: HashMap<DescriptorType, u32> = HashMap::new();
        for request in requests {
            for (ty, count) in &request.count {
                *required_descriptors.entry(*ty).or_insert(0) += *count;
            }
        }

        let layout_handles: Vec<vk::DescriptorSetLayout> = requests
            .iter()
            .map(|request| request.set_layout.handle())
            .collect();

        let alloc_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            descriptor_pool: self.handle,
            descriptor_set_count: layout_handles.len() as u32,
            p_set_layouts: layout_handles.as_ptr(),
            ..Default::default()
        };

        let set_handles = unsafe {
            self.context
                .device
                .handle()
                .allocate_descriptor_sets(&alloc_info)
                .map_err(map_vk)?
        };

        for (ty, count) in required_descriptors {
            *self.available_descriptors.get_mut(&ty).unwrap() -= count;
        }
        self.available_sets -= requests.len() as u32;

        let sets = set_handles
            .into_iter()
            .map(|handle| VkDescriptorSet::from_handle(self.context.clone(), handle))
            .collect();

        Ok(sets)
    }
}
