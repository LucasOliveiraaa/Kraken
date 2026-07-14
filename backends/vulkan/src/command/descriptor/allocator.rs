use std::sync::Arc;

use contract::GpuResult;

use crate::{
    VkContext,
    command::descriptor::{AllocationRequest, VkDescriptorPool, VkDescriptorSet},
};

pub struct VkDescriptorAllocator {
    context: Arc<VkContext>,
    pools: Vec<VkDescriptorPool>,
}

impl VkDescriptorAllocator {
    pub fn new(context: Arc<VkContext>) -> Self {
        Self {
            context,
            pools: Vec::new(),
        }
    }

    /// Allocates descriptor sets based on the provided requests.
    /// 
    /// # Note
    /// This funtion will not verify if the requests are valid for the set layouts, so it is the caller's responsibility to ensure that the requests are valid.
    pub fn allocate_unchecked(&mut self, requests: &[AllocationRequest]) -> GpuResult<Vec<VkDescriptorSet>> {
        for pool in self.pools.iter_mut().rev() {
            if pool.can_allocate(requests) {
                return pool.allocate_sets_unchecked(requests);
            }
        }
        
        let mut new_pool = VkDescriptorPool::new(self.context.clone())?;
        let sets = new_pool.allocate_sets_unchecked(requests)?;

        self.pools.push(new_pool);

        Ok(sets)
    }
}
