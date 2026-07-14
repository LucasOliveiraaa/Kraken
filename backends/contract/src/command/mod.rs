mod buffer;
mod pool;

use std::hash::Hash;

pub use buffer::*;
use kmath::{Vec3i, Vec3u};
pub use pool::*;

use crate::{
    GpuResult, pipeline::{ComputePipelineId, GraphicsPipelineId, PipelineStage, Scissor, Viewport}, resources::{BufferId, Filter, ImageId, ImageIdRef, ImageLayout, ImageSubresourceRange, ImageViewId, SamplerId}, sync::{FenceId, SemaphoreId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum QueueType {
    Graphics,
    Compute,
    Transfer,
}

pub struct SemaphoreWait {
    pub semaphore: SemaphoreId,
    pub stage: PipelineStage,
}

pub struct SubmitDesc {
    pub buffer_ids: Vec<CommandBufferId>,
    pub waits: Vec<SemaphoreWait>,
    pub signals: Vec<SemaphoreId>,
}

pub struct BufferCopyRegion {
    pub src_offset: u64,
    pub dst_offset: u64,
    pub size: u64,
}

pub struct BufferImageCopyRegion {
    pub buffer_offset: u64,
    pub buffer_row_length: u32,
    pub buffer_image_height: u32,
    pub image_subresource: ImageSubresourceRange,
    pub image_offset: Vec3u,
    pub image_extent: Vec3u,
}

pub struct ImageCopyRegion {
    pub src_subresource: ImageSubresourceRange,
    pub src_offset: Vec3u,
    pub dst_subresource: ImageSubresourceRange,
    pub dst_offset: Vec3u,
    pub extent: Vec3u,
}

pub struct ImageBlitRegion {
    pub src_subresource: ImageSubresourceRange,
    pub src_offsets: [Vec3i; 2],
    pub dst_subresource: ImageSubresourceRange,
    pub dst_offsets: [Vec3i; 2],
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AccessFlags: u64 {
        const INDIRECT_COMMAND_READ         = 1 << 0;
        const INDEX_READ                    = 1 << 1;
        const VERTEX_ATTRIBUTE_READ         = 1 << 2;
        const UNIFORM_READ                  = 1 << 3;
        const SHADER_READ                   = 1 << 4;
        const SHADER_WRITE                  = 1 << 5;
        const COLOR_ATTACHMENT_READ         = 1 << 6;
        const COLOR_ATTACHMENT_WRITE        = 1 << 7;
        const DEPTH_STENCIL_ATTACHMENT_READ = 1 << 8;
        const DEPTH_STENCIL_ATTACHMENT_WRITE= 1 << 9;
        const TRANSFER_READ                 = 1 << 10;
        const TRANSFER_WRITE                = 1 << 11;
        const HOST_READ                     = 1 << 12;
        const HOST_WRITE                    = 1 << 13;
        const MEMORY_READ                   = 1 << 14;
        const MEMORY_WRITE                  = 1 << 15;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryBarrier {
    pub src_stages: PipelineStage,
    pub src_access: AccessFlags,

    pub dst_stages: PipelineStage,
    pub dst_access: AccessFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferBarrier {
    pub buffer: BufferId,

    pub src_stages: PipelineStage,
    pub src_access: AccessFlags,

    pub dst_stages: PipelineStage,
    pub dst_access: AccessFlags,

    pub offset: u64,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageBarrier {
    pub image: ImageId,

    pub src_stages: PipelineStage,
    pub src_access: AccessFlags,

    pub dst_stages: PipelineStage,
    pub dst_access: AccessFlags,

    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,

    pub subresource_range: ImageSubresourceRange,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Barriers {
    pub memory: Vec<MemoryBarrier>,
    pub buffers: Vec<BufferBarrier>,
    pub images: Vec<ImageBarrier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StagingResource {
    Sampler(SamplerId),
    SampledTexture(ImageViewId, SamplerId),
    StorageImage(ImageViewId),

    StorageBuffer(BufferId),
    UniformBuffer(BufferId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceStage {
    // sets[set_idx][binding_idx][array_index]
    pub sets: Vec<Vec<Vec<StagingResource>>>,
}

pub trait CommandGateway: Send + Sync {
    /// Creates a new command pool with the specified description.
    fn create_command_pool(&self, desc: &CommandPoolDesc) -> GpuResult<CommandPoolId>;

    /// Destroys the command pool with the specified ID.
    fn destroy_command_pool(&self, pool_id: CommandPoolId) -> GpuResult<()>;

    /// Resets all the command buffers allocated from the specified command pool.
    fn reset_command_pool(&self, pool_id: CommandPoolId) -> GpuResult<()>;

    /// Creates a specified number of command buffers from the specified command pool.
    fn alloc_command_buffers(
        &self,
        pool_id: CommandPoolId,
        count: u32,
    ) -> GpuResult<Vec<CommandBufferId>>;

    /// Frees the specified command buffers from the specified command pool.
    fn free_command_buffers(
        &self,
        pool_id: CommandPoolId,
        buffer_ids: &[CommandBufferId],
    ) -> GpuResult<()>;

    /// Begins recording commands into the specified command buffer.
    fn begin_command_buffer(&self, buffer_id: CommandBufferId) -> GpuResult<()>;
    /// Ends recording commands into the specified command buffer.
    fn end_command_buffer(&self, buffer_id: CommandBufferId) -> GpuResult<()>;

    /// Resets the specified command buffer, making it ready for recording again.
    /// The command pool must have been created with the `ResetIndividually` reset mode for this to be valid.
    fn reset_command_buffer(
        &self,
        pool_id: CommandPoolId,
        buffer_id: CommandBufferId,
    ) -> GpuResult<()>;

    /// Submits the specified command buffers to the specified queue type,
    /// with optional wait and signal semaphores and an optional fence.
    fn submit(
        &self,
        queue_type: QueueType,
        descs: &[SubmitDesc],
        fence: Option<FenceId>,
    ) -> GpuResult<()>;

    fn bind_graphics_pipeline(
        &self,
        cmd: CommandBufferId,
        pipeline: GraphicsPipelineId,
    ) -> GpuResult<()>;
    fn bind_compute_pipeline(
        &self,
        cmd: CommandBufferId,
        pipeline: ComputePipelineId,
    ) -> GpuResult<()>;

    fn set_viewports(
        &self,
        cmd: CommandBufferId,
        first: u32,
        viewports: &[Viewport],
    ) -> GpuResult<()>;
    fn set_scissors(&self, cmd: CommandBufferId, first: u32, scissors: &[Scissor])
    -> GpuResult<()>;

    fn dispatch(&self, cmd: CommandBufferId, groups: Vec3u) -> GpuResult<()>;

    fn copy_buffer(
        &self,
        cmd: CommandBufferId,
        src: BufferId,
        dst: BufferId,
        regions: &[BufferCopyRegion],
    ) -> GpuResult<()>;
    fn copy_buffer_to_image(
        &self,
        cmd: CommandBufferId,
        src: BufferId,
        dst: ImageId,
        regions: &[BufferImageCopyRegion],
    ) -> GpuResult<()>;
    fn copy_image(
        &self,
        cmd: CommandBufferId,
        src: ImageId,
        src_layout: ImageLayout,
        dst: ImageId,
        dst_layout: ImageLayout,
        regions: &[ImageCopyRegion],
    ) -> GpuResult<()>;
    fn blit(
        &self,
        cmd: CommandBufferId,
        src: ImageIdRef,
        src_layout: ImageLayout,
        dst: ImageIdRef,
        dst_layout: ImageLayout,
        regions: &[ImageBlitRegion],
        filter: Filter,
    ) -> GpuResult<()>;

    fn fill_buffer(&self, cmd: CommandBufferId, buffer: BufferId, offset: u64, size: u64, value: u8) -> GpuResult<()>;

    /// Inserts a pipeline barrier into the specified command buffer, with the specified memory, buffer, and image barriers.
    fn pipeline_barrier(&self, cmd: CommandBufferId, barriers: &Barriers) -> GpuResult<()>;

    /// Commits the specified resource stage to the specified command buffer.
    /// Needs a bound pipeline.
    fn commit_resource_stage(&self, cmd: CommandBufferId, stage: &ResourceStage) -> GpuResult<()>;

    /// Pushes the specified data to the specified command buffer at the given offset.
    /// Needs a bound pipeline.
    fn push_constants(&self, cmd: CommandBufferId, offset: u32, data: &[u8]) -> GpuResult<()>;
}
