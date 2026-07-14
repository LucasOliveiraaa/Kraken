mod compute;
mod layout;
mod reflection;

pub use compute::*;
pub use layout::*;
pub use reflection::*;

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use ash::vk;
use contract::{
    GpuError, GpuResult,
    pipeline::{ComputePipelineDesc, ComputePipelineId, PipelineGateway},
};

use crate::{VkContext, alloc::Slab, core::WeakGateways, map_vk};

pub struct VkPipelineLayoutCache {
    layout: parking_lot::Mutex<HashMap<LayoutInfo, Arc<VkPipelineLayout>>>,
}

impl VkPipelineLayoutCache {
    pub fn new() -> Self {
        Self {
            layout: parking_lot::Mutex::new(HashMap::new()),
        }
    }

    pub fn register(
        &self,
        context: Arc<VkContext>,
        layout_info: LayoutInfo,
    ) -> GpuResult<Arc<VkPipelineLayout>> {
        let mut layout_cache = self.layout.lock();

        if let Some(layout) = layout_cache.get(&layout_info) {
            return Ok(layout.clone());
        }

        let layout = VkPipelineLayout::new(context, &layout_info)?;
        let layout_arc = Arc::new(layout);

        layout_cache.insert(layout_info, layout_arc.clone());

        Ok(layout_arc)
    }

    pub fn get(&self, layout_info: &LayoutInfo) -> Option<Arc<VkPipelineLayout>> {
        let layout_cache = self.layout.lock();
        layout_cache.get(layout_info).cloned()
    }

    pub fn get_or_register(
        &self,
        context: Arc<VkContext>,
        layout_info: LayoutInfo,
    ) -> GpuResult<Arc<VkPipelineLayout>> {
        if let Some(layout) = self.get(&layout_info) {
            return Ok(layout);
        }

        self.register(context, layout_info)
    }
}

pub struct VkPipelineGateway {
    context: Arc<VkContext>,
    gateways: OnceLock<WeakGateways>,

    pipeline_cache: vk::PipelineCache,
    layout_cache: VkPipelineLayoutCache,
    compute: parking_lot::Mutex<Slab<Arc<VkComputePipeline>>>,
}

impl VkPipelineGateway {
    pub fn new(context: Arc<VkContext>, pipeline_cache_data: Option<&[u8]>) -> GpuResult<Self> {
        let cache_info = if let Some(data) = pipeline_cache_data {
            vk::PipelineCacheCreateInfo {
                s_type: vk::StructureType::PIPELINE_CACHE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::PipelineCacheCreateFlags::empty(),
                p_initial_data: data.as_ptr() as *const std::os::raw::c_void,
                initial_data_size: data.len(),
                ..Default::default()
            }
        } else {
            vk::PipelineCacheCreateInfo {
                s_type: vk::StructureType::PIPELINE_CACHE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::PipelineCacheCreateFlags::empty(),
                p_initial_data: std::ptr::null(),
                initial_data_size: 0,
                ..Default::default()
            }
        };

        let pipeline_cache = unsafe {
            context
                .device
                .handle()
                .create_pipeline_cache(&cache_info, None)
                .map_err(|err| map_vk(err))?
        };

        Ok(Self {
            context: context.clone(),
            gateways: OnceLock::new(),

            pipeline_cache,
            layout_cache: VkPipelineLayoutCache::new(),
            compute: parking_lot::Mutex::new(Slab::new()),
        })
    }

    pub fn set_gateways(&self, gateways: WeakGateways) {
        self.gateways.set(gateways).unwrap();
    }

    pub fn get_layout_cache(&self) -> &VkPipelineLayoutCache {
        &self.layout_cache
    }

    pub fn get_compute_pipeline(
        &self,
        pipeline_id: ComputePipelineId,
    ) -> GpuResult<Arc<VkComputePipeline>> {
        let compute = self.compute.lock();
        let pipeline = compute
            .get(pipeline_id.0 as usize)
            .ok_or(GpuError::InvalidComputePipelineId(pipeline_id))?;

        Ok(pipeline.clone())
    }
}

impl PipelineGateway for VkPipelineGateway {
    fn create_compute_pipelines(
        &self,
        descs: &[ComputePipelineDesc],
    ) -> GpuResult<Vec<ComputePipelineId>> {
        let vk_descs = descs
            .iter()
            .map(|desc| {
                let gateways = self.gateways.get().unwrap();

                let resources = gateways.resources.upgrade().unwrap();

                Ok(VkComputePipelineDesc {
                    shader: resources.get_shader(desc.shader_id)?,
                })
            })
            .collect::<GpuResult<Vec<_>>>()?;

        let pipelines = VkComputePipeline::create_various(
            self.context.clone(),
            &self.layout_cache,
            self.pipeline_cache,
            &vk_descs,
        )?;

        Ok(pipelines
            .into_iter()
            .map(|compute_pipeline| {
                let mut compute = self.compute.lock();
                ComputePipelineId(compute.insert(Arc::new(compute_pipeline)) as u32)
            })
            .collect::<Vec<_>>())
    }

    fn destroy_compute_pipeline(&self, pipeline_id: ComputePipelineId) -> GpuResult<()> {
        let mut compute = self.compute.lock();
        if compute.remove(pipeline_id.0 as usize).is_none() {
            return Err(GpuError::InvalidComputePipelineId(pipeline_id));
        }

        Ok(())
    }

    fn get_pipeline_cache_blob(&self) -> GpuResult<Vec<u8>> {
        let data = unsafe {
            self.context
                .device
                .handle()
                .get_pipeline_cache_data(self.pipeline_cache)
                .map_err(map_vk)?
        };

        Ok(data)
    }
}

impl Drop for VkPipelineGateway {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .handle()
                .destroy_pipeline_cache(self.pipeline_cache, None);
        }
    }
}
