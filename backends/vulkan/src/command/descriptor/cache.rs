pub struct VkDescriptorSetCache {
    sets: HashMap<ResourceStage, Arc<VkDescriptorSet>>,
}

impl VkDescriptorSetCache {
    pub fn new() -> Self {
        Self {
            sets: HashMap::new(),
        }
    }

    pub fn get(&self, stage: &ResourceStage) -> Option<Arc<VkDescriptorSet>> {
        self.sets.get(stage).cloned()
    }

    pub fn insert(&mut self, stage: ResourceStage, set: Arc<VkDescriptorSet>) {
        self.sets.insert(stage, set);
    }
}