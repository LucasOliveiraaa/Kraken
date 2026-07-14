mod compute;
mod graphics;

pub use compute::*;
pub use graphics::*;
use kmath::{Vec2f, Vec2u};

use crate::GpuResult;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PipelineStage: u32 {
        const TOP_OF_PIPE = 1 << 0;
        const DRAW_INDIRECT = 1 << 1;
        const VERTEX_INPUT = 1 << 2;
        const VERTEX_SHADER = 1 << 3;
        const FRAGMENT_SHADER = 1 << 4;
        const EARLY_FRAGMENT_TESTS = 1 << 5;
        const LATE_FRAGMENT_TESTS = 1 << 6;
        const COLOR_ATTACHMENT_OUTPUT = 1 << 7;
        const COMPUTE_SHADER = 1 << 8;
        const TRANSFER = 1 << 9;
        const BOTTOM_OF_PIPE = 1 << 10;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub position: Vec2f,
    pub size: Vec2f,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scissor {
    pub position: Vec2u,
    pub size: Vec2u,
}

pub trait PipelineGateway: Sync + Send {
    /// Create a new compute pipeline given its description.
    fn create_compute_pipelines(
        &self,
        descs: &[ComputePipelineDesc],
    ) -> GpuResult<Vec<ComputePipelineId>>;

    /// Destroy a compute pipeline given its ID.
    fn destroy_compute_pipeline(&self, pipeline_id: ComputePipelineId) -> GpuResult<()>;

    fn get_pipeline_cache_blob(&self) -> GpuResult<Vec<u8>>;
}
