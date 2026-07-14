use std::sync::Arc;

use ash::vk;

use crate::VkContext;

pub struct VkDescriptorSet {
    context: Arc<VkContext>,
    handle: vk::DescriptorSet,
}

impl VkDescriptorSet {
    pub fn from_handle(context: Arc<VkContext>, handle: vk::DescriptorSet) -> Self {
        Self { context, handle }
    }

    pub fn handle(&self) -> vk::DescriptorSet {
        self.handle
    }
}