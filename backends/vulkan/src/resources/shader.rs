use std::sync::Arc;

use ash::vk;
use contract::{
    GpuError, GpuResult,
    resources::{ShaderDesc, ShaderPass, ShaderStage},
};
use spirq::{entry_point::EntryPoint, spirv::ExecutionModel};

use crate::{VkContext, map_spirq, map_vk};

pub struct VkShader {
    _context: Arc<VkContext>,
    handle: vk::ShaderModule,
    passes: Vec<ShaderPass>,
    spirv: Vec<u32>,
}

impl VkShader {
    pub fn new(context: Arc<VkContext>, desc: &ShaderDesc) -> GpuResult<Self> {
        let size = desc.spirv.len() * std::mem::size_of::<u32>();
        if size == 0 {
            return Err(GpuError::InvalidSPIRV {
                reason: "code is empty".to_string(),
            });
        }
        if size % 4 == 0 {
            return Err(GpuError::InvalidSPIRV {
                reason: format!("code size must be a multiple of 4, got {}", size),
            });
        }

        let module_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: desc.spirv.len() * std::mem::size_of::<u32>(),
            p_code: desc.spirv.as_ptr(),
            ..Default::default()
        };

        let handle = unsafe {
            context
                .device
                .handle()
                .create_shader_module(&module_info, None)
                .map_err(map_vk)?
        };

        let mut stages = desc
            .passes
            .iter()
            .map(|pass| pass.stage)
            .collect::<Vec<_>>();
        stages.dedup();
        if stages.len() != desc.passes.len() {
            return Err(GpuError::InvalidShaderPasses {
                reason: "duplicate stages found".to_string(),
            });
        }

        Ok(Self {
            _context: context,
            handle,
            passes: desc.passes.clone(),
            spirv: desc.spirv.clone(),
        })
    }

    pub fn handle(&self) -> vk::ShaderModule {
        self.handle
    }

    pub fn spirv(&self) -> &[u32] {
        &self.spirv
    }

    pub fn passes(&self) -> &[ShaderPass] {
        &self.passes
    }

    pub fn get_pass(&self, stage: ShaderStage) -> Option<&ShaderPass> {
        self.passes.iter().find(|pass| pass.stage == stage)
    }

    pub fn reflect(&self, passes: &[ShaderPass]) -> GpuResult<Vec<EntryPoint>> {
        let entry_points = spirq::ReflectConfig::new()
            .spv(self.spirv.as_slice())
            .reflect()
            .map_err(map_spirq)?;

        Ok(entry_points
            .into_iter()
            .filter(|entry| {
                let stage = match entry.exec_model {
                    ExecutionModel::Vertex => ShaderStage::Vertex,
                    ExecutionModel::Fragment => ShaderStage::Fragment,
                    ExecutionModel::GLCompute => ShaderStage::Compute,
                    _ => return false,
                };
                passes.iter().any(|pass| pass.stage == stage)
            })
            .collect())
    }
}
