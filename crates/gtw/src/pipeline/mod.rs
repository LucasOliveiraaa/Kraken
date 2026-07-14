mod graphics;
mod compute;

use contract::GpuResult;
pub use graphics::*;
pub use compute::*;

pub use contract::pipeline::{PipelineStage, Viewport, Scissor};

use crate::{macros::ToContract, resources::Shader};

pub struct ComputePipelineDesc {
    pub shader: Shader,
}

impl ToContract for ComputePipelineDesc {
    type Contract = contract::pipeline::ComputePipelineDesc;

    fn to_contract(&self) -> Self::Contract {
        Self::Contract {
            shader_id: self.shader.handle(),
        }
    }
}

create_gateway! {
    pub struct PipelineGateway {
        contract::pipeline::PipelineGateway
    }
}

impl PipelineGateway {
    create_handle_methods! {
        fn create_compute_pipelines(#[desc_slice] descs: &[ComputePipelineDesc]) -> [ComputePipeline];
    }

    pub fn get_pipeline_cache_blob(&self) -> GpuResult<Vec<u8>> {
        self.raw().get_pipeline_cache_blob()
    }
}