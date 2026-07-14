use std::{collections::HashMap, sync::Arc};

use contract::{
    GpuResult,
    command::{
        AccessFlags, BufferCopyRegion, BufferImageCopyRegion, CommandBufferId, ImageBlitRegion,
        ImageCopyRegion, MemoryBarrier, ResourceStage, StagingResource,
    },
    pipeline::{PipelineStage, Scissor, Viewport},
    resources::{Filter, ImageLayout, ImageSubresourceRange, SamplerId},
};
use kmath::Vec3u;

use crate::{
    command::{CommandGateway, CommandPool},
    macros::ToContract,
    pipeline::{ComputePipeline, GraphicsPipeline},
    resources::{Buffer, Image, ImageRef, ImageView},
};

pub struct StagePoint {
    pub set: u32,
    pub binding: u32,
}

impl StagePoint {
    pub fn new(set: u32, binding: u32) -> Self {
        Self { set, binding }
    }
}

pub struct Stage {
    cmd: CommandBuffer,
    stage: HashMap<(u32, u32), Vec<StagingResource>>,
}

impl Stage {
    pub fn new(cmd: CommandBuffer) -> Self {
        Self {
            cmd,
            stage: HashMap::new(),
        }
    }

    pub fn stage_storage_buffer(mut self, point: StagePoint, buffer: Buffer) -> Self {
        self.stage
            .entry((point.set, point.binding))
            .or_insert_with(Vec::new)
            .push(StagingResource::StorageBuffer(buffer.handle()));
        self
    }

    pub fn stage_uniform_buffer(mut self, point: StagePoint, buffer: Buffer) -> Self {
        self.stage
            .entry((point.set, point.binding))
            .or_insert_with(Vec::new)
            .push(StagingResource::UniformBuffer(buffer.handle()));
        self
    }

    pub fn stage_sampler(mut self, point: StagePoint, sampler: SamplerId) -> Self {
        self.stage
            .entry((point.set, point.binding))
            .or_insert_with(Vec::new)
            .push(StagingResource::Sampler(sampler));
        self
    }

    pub fn stage_storage_image(mut self, point: StagePoint, image_view: ImageView) -> Self {
        self.stage
            .entry((point.set, point.binding))
            .or_insert_with(Vec::new)
            .push(StagingResource::StorageImage(image_view.handle()));
        self
    }

    pub fn stage_sampled_image(
        mut self,
        point: StagePoint,
        image_view: ImageView,
        sampler: SamplerId,
    ) -> Self {
        self.stage
            .entry((point.set, point.binding))
            .or_insert_with(Vec::new)
            .push(StagingResource::SampledTexture(
                image_view.handle(),
                sampler,
            ));
        self
    }

    fn build_stage(&self) -> ResourceStage {
        // Determine required dimensions.
        let max_set = self.stage.keys().map(|(set, _)| *set).max().unwrap_or(0);

        let mut sets = vec![Vec::new(); max_set as usize + 1];

        for ((set, binding), resources) in &self.stage {
            let set = *set as usize;
            let binding = *binding as usize;

            let bindings = &mut sets[set];

            if bindings.len() <= binding {
                bindings.resize_with(binding + 1, Vec::new);
            }

            bindings[binding] = resources.clone();
        }

        ResourceStage { sets }
    }

    pub fn commit(self) -> GpuResult<()> {
        let stage = self.build_stage();

        self.cmd
            .raw_gtw()
            .commit_resource_stage(self.cmd.handle(), &stage)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BufferBarrier {
    pub buffer: Buffer,

    pub src_stages: PipelineStage,
    pub src_access: AccessFlags,

    pub dst_stages: PipelineStage,
    pub dst_access: AccessFlags,

    pub offset: u64,
    pub size: u64,
}
impl ToContract for BufferBarrier {
    type Contract = contract::command::BufferBarrier;

    fn to_contract(&self) -> Self::Contract {
        contract::command::BufferBarrier {
            buffer: self.buffer.handle(),
            src_stages: self.src_stages,
            src_access: self.src_access,
            dst_stages: self.dst_stages,
            dst_access: self.dst_access,
            offset: self.offset,
            size: self.size,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageBarrier {
    pub image: Image,

    pub src_stages: PipelineStage,
    pub src_access: AccessFlags,

    pub dst_stages: PipelineStage,
    pub dst_access: AccessFlags,

    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,

    pub subresource_range: ImageSubresourceRange,
}
impl ToContract for ImageBarrier {
    type Contract = contract::command::ImageBarrier;

    fn to_contract(&self) -> Self::Contract {
        contract::command::ImageBarrier {
            image: self.image.handle(),
            src_stages: self.src_stages,
            src_access: self.src_access,
            dst_stages: self.dst_stages,
            dst_access: self.dst_access,
            old_layout: self.old_layout,
            new_layout: self.new_layout,
            subresource_range: self.subresource_range,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Barriers {
    pub memory: Vec<MemoryBarrier>,
    pub buffers: Vec<BufferBarrier>,
    pub images: Vec<ImageBarrier>,
}
impl ToContract for Barriers {
    type Contract = contract::command::Barriers;

    fn to_contract(&self) -> Self::Contract {
        contract::command::Barriers {
            memory: self.memory.clone(),
            buffers: self.buffers.iter().map(|b| b.to_contract()).collect(),
            images: self.images.iter().map(|i| i.to_contract()).collect(),
        }
    }
}

impl Barriers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_memory(&mut self, barrier: MemoryBarrier) {
        self.memory.push(barrier);
    }

    pub fn add_buffer(&mut self, barrier: BufferBarrier) {
        self.buffers.push(barrier);
    }

    pub fn add_image(&mut self, barrier: ImageBarrier) {
        self.images.push(barrier);
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct InnerCommandBuffer {
    gtw: CommandGateway,
    pool: CommandPool,
    handle: CommandBufferId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct CommandBuffer {
    inner: Arc<InnerCommandBuffer>,
}

impl CommandBuffer {
    pub fn from_handle(gtw: CommandGateway, pool: CommandPool, handle: CommandBufferId) -> Self {
        Self { inner: Arc::new(InnerCommandBuffer { gtw, pool, handle }) }
    }

    #[allow(dead_code)]
    pub(crate) fn handle(&self) -> CommandBufferId {
        self.inner.handle
    }

    #[allow(dead_code)]
    fn gtw(&self) -> &CommandGateway {
        &self.inner.gtw
    }

    #[allow(dead_code)]
    fn raw_gtw(&self) -> std::sync::Arc<<CommandGateway as crate::macros::GatewayWrapper>::Raw> {
        self.inner.gtw.raw()
    }

    fn pool(&self) -> &CommandPool {
        &self.inner.pool
    }
}

impl CommandBuffer {
    pub fn begin(&self) -> GpuResult<()> {
        self.raw_gtw().begin_command_buffer(self.handle())
    }

    pub fn end(&self) -> GpuResult<()> {
        self.raw_gtw().end_command_buffer(self.handle())
    }

    pub fn reset(&self) -> GpuResult<()> {
        self.raw_gtw()
            .reset_command_buffer(self.pool().handle(), self.handle())
    }

    pub fn bind_graphics_pipeline(&self, pipeline: GraphicsPipeline) -> GpuResult<()> {
        self.raw_gtw()
            .bind_graphics_pipeline(self.handle(), pipeline.handle())
    }

    pub fn bind_compute_pipeline(&self, pipeline: ComputePipeline) -> GpuResult<()> {
        self.raw_gtw()
            .bind_compute_pipeline(self.handle(), pipeline.handle())
    }

    pub fn set_viewports(&self, first: u32, viewports: &[Viewport]) -> GpuResult<()> {
        self.raw_gtw().set_viewports(self.handle(), first, viewports)
    }

    pub fn set_scissors(&self, first: u32, scissors: &[Scissor]) -> GpuResult<()> {
        self.raw_gtw().set_scissors(self.handle(), first, scissors)
    }

    pub fn dispatch(&self, groups: Vec3u) -> GpuResult<()> {
        self.raw_gtw().dispatch(self.handle(), groups)
    }

    pub fn copy_buffer(
        &self,
        src: Buffer,
        dst: Buffer,
        regions: &[BufferCopyRegion],
    ) -> GpuResult<()> {
        self.raw_gtw()
            .copy_buffer(self.handle(), src.handle(), dst.handle(), regions)
    }
    pub fn copy_buffer_to_image(
        &self,
        src: Buffer,
        dst: Image,
        regions: &[BufferImageCopyRegion],
    ) -> GpuResult<()> {
        self.raw_gtw()
            .copy_buffer_to_image(self.handle(), src.handle(), dst.handle(), regions)
    }
    pub fn copy_image(
        &self,
        src: Image,
        src_layout: ImageLayout,
        dst: Image,
        dst_layout: ImageLayout,
        regions: &[ImageCopyRegion],
    ) -> GpuResult<()> {
        self.raw_gtw().copy_image(
            self.handle(),
            src.handle(),
            src_layout,
            dst.handle(),
            dst_layout,
            regions,
        )
    }

    pub fn blit(
        &self,
        src: ImageRef,
        src_layout: ImageLayout,
        dst: ImageRef,
        dst_layout: ImageLayout,
        regions: &[ImageBlitRegion],
        filter: Filter,
    ) -> GpuResult<()> {
        self.raw_gtw().blit(
            self.handle(),
            src.to_contract(),
            src_layout,
            dst.to_contract(),
            dst_layout,
            regions,
            filter,
        )
    }

    pub fn fill_buffer(&self, buffer: Buffer, offset: u64, size: u64, value: u8) -> GpuResult<()> {
        self.raw_gtw()
            .fill_buffer(self.handle(), buffer.handle(), offset, size, value)
    }

    pub fn pipeline_barrier(&self, barriers: &Barriers) -> GpuResult<()> {
        self.raw_gtw()
            .pipeline_barrier(self.handle(), &barriers.to_contract())
    }

    pub fn stage(&self) -> Stage {
        Stage::new(self.clone())
    }

    pub fn push_constants(&self, offset: u32, data: &[u8]) -> GpuResult<()> {
        self.raw_gtw().push_constants(self.handle(), offset, data)
    }
}
