use std::sync::Arc;

use ash::vk;
use contract::pipeline::{Scissor, Viewport};

use crate::{pipeline::VkComputePipeline, IntoVk};

use super::VkCommandBuffer;

pub enum BoundPipeline {
    Compute(Arc<VkComputePipeline>),
}

impl BoundPipeline {
    pub fn handle(&self) -> vk::Pipeline {
        match self {
            BoundPipeline::Compute(pipeline) => pipeline.handle(),
        }
    }

    pub fn layout_info(&self) -> &crate::pipeline::LayoutInfo {
        match self {
            BoundPipeline::Compute(pipeline) => pipeline.layout_info(),
        }
    }

    pub fn layout(&self) -> &Arc<crate::pipeline::VkPipelineLayout> {
        match self {
            BoundPipeline::Compute(pipeline) => pipeline.layout(),
        }
    }
}

impl VkCommandBuffer {
    pub fn bind_compute_pipeline(&self, pipeline: Arc<VkComputePipeline>) {
        let mut current = self.current_bound_pipeline.lock();

        unsafe {
            self.context.device.handle().cmd_bind_pipeline(
                self.handle,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.handle(),
            );
        }

        *current = Some(BoundPipeline::Compute(pipeline));
    }

    pub fn set_viewport(&self, first: u32, viewports: &[Viewport]) {
        let viewports = viewports
            .iter()
            .cloned()
            .map(|vp| vp.into_vk())
            .collect::<Vec<_>>();

        unsafe {
            self.context
                .device
                .handle()
                .cmd_set_viewport(self.handle, first, &viewports);
        }
    }

    pub fn set_scissor(&self, first: u32, scissors: &[Scissor]) {
        let scissors = scissors
            .iter()
            .cloned()
            .map(|sc| sc.into_vk())
            .collect::<Vec<_>>();

        unsafe {
            self.context
                .device
                .handle()
                .cmd_set_scissor(self.handle, first, &scissors);
        }
    }
}