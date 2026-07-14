mod resolve;
mod write;

use contract::{command::ResourceStage, GpuError, GpuResult};

use crate::{command::descriptor::VkDescriptorAllocator, resources::VkResourcesGateway};

use super::VkCommandBuffer;

impl VkCommandBuffer {
    pub fn commit_resource_stage(
        &self,
        allocator: &mut VkDescriptorAllocator,
        resources_gateway: &VkResourcesGateway,
        stage: &ResourceStage,
    ) -> GpuResult<()> {
        // Bug #5 (review): previously `current_bound_pipeline.lock()` was
        // held from here all the way through resource resolution,
        // allocation, and the `vkUpdateDescriptorSets` call — far longer
        // than needed, since only `pipeline.layout()` is actually used.
        // Grab just the layout `Arc` (cheap clone) and drop the guard
        // immediately, rather than holding the mutex across unrelated,
        // potentially-slow work (resource lookups, descriptor allocation).
        let layout = {
            let curr_pipeline = self.current_bound_pipeline.lock();
            let Some(pipeline) = curr_pipeline.as_ref() else {
                return Err(GpuError::NoPipelineBound);
            };
            pipeline.layout().clone()
        };

        // Resolve staged resources against the layout, with full count/type
        // validation (bugs #1, #2, #3, #6 — see resolve.rs).
        let sets = resolve::resolve_sets(&layout, stage, resources_gateway)?;

        // Allocate descriptor sets and build the vk::WriteDescriptorSet array.
        let (writes, _image_infos, _buffer_infos, sets_handles) =
            write::build_writes(allocator, &sets)?;

        // `_image_infos`/`_buffer_infos` must outlive this call, since `writes`
        // holds raw pointers into them — they're kept alive as local bindings
        // until the end of this function.
        unsafe {
            self.context
                .device
                .handle()
                .update_descriptor_sets(&writes, &[]);
        }

        // Record the resulting vk::DescriptorSet handles on the command buffer.
        let mut current_sets = self.current_bound_descriptor_sets.lock();
        *current_sets = sets_handles.iter().map(|s| s.handle()).collect();

        Ok(())
    }
}