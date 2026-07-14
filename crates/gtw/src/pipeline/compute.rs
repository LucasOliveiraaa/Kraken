use contract::pipeline::ComputePipelineId;

use crate::pipeline::PipelineGateway;

create_handle_wrapper!(ComputePipeline, PipelineGateway, ComputePipelineId, destroy_compute_pipeline);