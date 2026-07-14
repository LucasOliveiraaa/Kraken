use std::{ffi::CString, sync::Arc};

use ash::vk;
use contract::{GpuError, GpuResult, resources::ShaderStage};

use crate::{
    core::VkContext, map_vk, pipeline::{LayoutInfo, VkPipelineLayout, VkPipelineLayoutCache, get_layout_info}, resources::VkShader,
};

pub struct VkComputePipelineDesc {
    pub shader: Arc<VkShader>,
}

pub struct VkComputePipeline {
    context: Arc<VkContext>,
    handle: vk::Pipeline,

    layout_info: LayoutInfo,
    layout: Arc<VkPipelineLayout>,
}

impl VkComputePipeline {
    pub fn create_various(
        context: Arc<VkContext>,
        layout_cache: &VkPipelineLayoutCache,
        pipeline_cache: vk::PipelineCache,
        descs: &[VkComputePipelineDesc],
    ) -> GpuResult<Vec<Self>> {
        let mut name_vec = vec![];
        let mut layout_infos = vec![];

        let pipeline_infos = descs
            .iter()
            .map(|desc| {
                let compute_pass = desc.shader.get_pass(ShaderStage::Compute).ok_or(
                    GpuError::InvalidShaderStage {
                        reason: "missing compute pass".to_string(),
                    },
                )?;

                let name = CString::new(compute_pass.entry_point.clone()).unwrap();

                let layout_info = get_layout_info(desc.shader.clone(), &[compute_pass.clone()])?;
                let layout = layout_cache.get_or_register(context.clone(), layout_info.clone())?;

                let stage = vk::PipelineShaderStageCreateInfo {
                    s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                    p_next: std::ptr::null(),
                    flags: vk::PipelineShaderStageCreateFlags::empty(),
                    module: desc.shader.handle(),
                    p_name: name.as_ptr(),
                    stage: vk::ShaderStageFlags::COMPUTE,
                    p_specialization_info: std::ptr::null(),
                    ..Default::default()
                };

                let info = vk::ComputePipelineCreateInfo {
                    s_type: vk::StructureType::COMPUTE_PIPELINE_CREATE_INFO,
                    p_next: std::ptr::null(),
                    flags: vk::PipelineCreateFlags::empty(),
                    stage,
                    layout: layout.handle(),
                    base_pipeline_handle: vk::Pipeline::null(),
                    base_pipeline_index: -1,
                    ..Default::default()
                };

                name_vec.push(name);

                layout_infos.push((layout_info, layout));
                Ok(info)
            })
            .collect::<GpuResult<Vec<_>>>()?;

        let result = unsafe {
            context
                .device
                .handle()
                .create_compute_pipelines(pipeline_cache, &pipeline_infos, None)
                .map_err(|(pipelines, err)| {
                    for pipeline in pipelines {
                        context.device.handle().destroy_pipeline(pipeline, None);
                    }
                    map_vk(err)
                })?
        };

        Ok(result
            .into_iter()
            .zip(layout_infos)
            .map(|(handle, (layout_info, layout))| Self::new(context.clone(), handle, layout_info, layout))
            .collect::<GpuResult<Vec<_>>>()?)
    }

    pub fn new(context: Arc<VkContext>, handle: vk::Pipeline, layout_info: LayoutInfo, layout: Arc<VkPipelineLayout>) -> GpuResult<Self> {
        Ok(Self { context, handle, layout_info, layout })
    }

    pub fn handle(&self) -> vk::Pipeline {
        self.handle
    }

    pub fn layout_info(&self) -> &LayoutInfo {
        &self.layout_info
    }

    pub fn layout(&self) -> &Arc<VkPipelineLayout> {
        &self.layout
    }
}

impl Drop for VkComputePipeline {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .handle()
                .destroy_pipeline(self.handle, None);
        }
    }
}
