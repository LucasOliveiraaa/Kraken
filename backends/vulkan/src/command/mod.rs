mod buffer;
pub mod descriptor;
mod pool;

pub use buffer::*;
pub use pool::*;

use ash::vk;

use std::sync::{Arc, OnceLock};

use contract::{
    GpuError, GpuResult,
    command::{
        AccessFlags, Barriers, CommandBufferId, CommandGateway, CommandPoolDesc, CommandPoolId,
        ImageBlitRegion, ImageCopyRegion, QueueType, ResourceStage, SubmitDesc,
    },
    pipeline::{ComputePipelineId, Scissor, Viewport},
    resources::{BufferId, Filter, ImageId, ImageIdRef, ImageLayout},
    sync::FenceId,
};

use crate::{
    FromContract, FromVk, VkContext, VkSemaphoreWait, VkSubmitDesc, WeakGateways, alloc::Slab,
    command::descriptor::VkDescriptorAllocator,
};

impl FromContract<AccessFlags> for vk::AccessFlags2 {
    fn from_contract(value: AccessFlags) -> Self {
        vk::AccessFlags2::from_raw(value.bits())
    }
}
impl FromVk<vk::AccessFlags2> for AccessFlags {
    fn from_vk(value: vk::AccessFlags2) -> Self {
        AccessFlags::from_bits(value.as_raw()).unwrap()
    }
}

pub struct VkCommandGateway {
    context: Arc<VkContext>,
    gateways: OnceLock<WeakGateways>,

    descriptor_allocator: parking_lot::Mutex<VkDescriptorAllocator>,

    pools: parking_lot::Mutex<Slab<Arc<VkCommandPool>>>,
    buffers: parking_lot::Mutex<Slab<Arc<VkCommandBuffer>>>,
}

impl VkCommandGateway {
    pub fn new(context: Arc<VkContext>) -> GpuResult<Self> {
        Ok(Self {
            context: context.clone(),
            gateways: OnceLock::new(),

            descriptor_allocator: parking_lot::Mutex::new(VkDescriptorAllocator::new(
                context.clone(),
            )),

            pools: parking_lot::Mutex::new(Slab::new()),
            buffers: parking_lot::Mutex::new(Slab::new()),
        })
    }

    pub fn set_gateways(&self, gateways: WeakGateways) {
        self.gateways.set(gateways).unwrap();
    }

    fn gateways(&self) -> GpuResult<WeakGateways> {
        self.gateways.get().cloned().ok_or(GpuError::GatewaysNotSet)
    }

    pub fn get_pool(&self, pool_id: CommandPoolId) -> GpuResult<Arc<VkCommandPool>> {
        let pools = self.pools.lock();
        pools
            .get(pool_id.0 as usize)
            .cloned()
            .ok_or_else(|| GpuError::InvalidCommandPoolId(pool_id))
    }

    pub fn get_buffer(&self, buffer_id: CommandBufferId) -> GpuResult<Arc<VkCommandBuffer>> {
        let buffers = self.buffers.lock();
        buffers
            .get(buffer_id.0 as usize)
            .cloned()
            .ok_or_else(|| GpuError::InvalidCommandBufferId(buffer_id))
    }
}

impl CommandGateway for VkCommandGateway {
    fn create_command_pool(&self, desc: &CommandPoolDesc) -> GpuResult<CommandPoolId> {
        let vk_desc = VkCommandPoolDesc {
            queue: match desc.queue_type {
                QueueType::Graphics => self.context.queues.graphics.clone(),
                QueueType::Compute => self.context.queues.compute.clone(),
                QueueType::Transfer => self.context.queues.transfer.clone(),
            },
            reset_mode: desc.reset_mode,
            buffer_lifetime: desc.buffer_lifetime,
        };

        let pool = Arc::new(VkCommandPool::new(self.context.clone(), vk_desc)?);

        let mut pools = self.pools.lock();
        let pool_id = CommandPoolId(pools.insert(pool) as u32);
        Ok(pool_id)
    }

    fn destroy_command_pool(&self, pool_id: CommandPoolId) -> GpuResult<()> {
        let mut pools = self.pools.lock();
        if pools.remove(pool_id.0 as usize).is_none() {
            return Err(GpuError::InvalidCommandPoolId(pool_id));
        }

        Ok(())
    }

    fn reset_command_pool(&self, pool_id: CommandPoolId) -> GpuResult<()> {
        let pool = self.get_pool(pool_id)?;
        pool.reset()
    }

    fn alloc_command_buffers(
        &self,
        pool_id: CommandPoolId,
        count: u32,
    ) -> GpuResult<Vec<CommandBufferId>> {
        let pool = self.get_pool(pool_id)?;
        let buffers = VkCommandBuffer::various(self.context.clone(), pool, count)?;

        let mut buffer_ids = Vec::with_capacity(buffers.len());
        let mut buffer_slab = self.buffers.lock();
        for buffer in buffers {
            let buffer_id = CommandBufferId(buffer_slab.insert(Arc::new(buffer)) as u32);
            buffer_ids.push(buffer_id);
        }

        Ok(buffer_ids)
    }

    fn free_command_buffers(
        &self,
        pool_id: CommandPoolId,
        buffer_ids: &[CommandBufferId],
    ) -> GpuResult<()> {
        let pool = self.get_pool(pool_id)?;
        let buffer_slab = self.buffers.lock();

        let buffers = buffer_ids
            .iter()
            .map(|&buffer_id| {
                buffer_slab
                    .get(buffer_id.0 as usize)
                    .cloned()
                    .ok_or_else(|| GpuError::InvalidCommandBufferId(buffer_id))
            })
            .collect::<GpuResult<Vec<_>>>()?;
        pool.free_command_buffers(&buffers);

        Ok(())
    }

    fn begin_command_buffer(&self, buffer_id: CommandBufferId) -> GpuResult<()> {
        let buffer = self.get_buffer(buffer_id)?;
        buffer.begin()
    }

    fn end_command_buffer(&self, buffer_id: CommandBufferId) -> GpuResult<()> {
        let buffer = self.get_buffer(buffer_id)?;
        buffer.end()
    }

    fn reset_command_buffer(
        &self,
        pool_id: CommandPoolId,
        buffer_id: CommandBufferId,
    ) -> GpuResult<()> {
        let pool = self.get_pool(pool_id)?;
        let buffer = self.get_buffer(buffer_id)?;
        pool.reset_command_buffer(buffer)
    }

    fn submit(
        &self,
        queue_type: QueueType,
        descs: &[SubmitDesc],
        fence: Option<FenceId>,
    ) -> GpuResult<()> {
        let queue = match queue_type {
            QueueType::Graphics => self.context.queues.graphics.clone(),
            QueueType::Compute => self.context.queues.compute.clone(),
            QueueType::Transfer => self.context.queues.transfer.clone(),
        };

        let gateways = self.gateways.get().ok_or(GpuError::GatewaysNotSet)?;
        let sync_gateway = gateways.sync.upgrade().ok_or(GpuError::GatewaysNotSet)?;

        let fence = fence
            .map(|fence_id| sync_gateway.get_fence(fence_id))
            .transpose()?;

        let submits = descs
            .into_iter()
            .map(|desc| {
                let buffers = desc
                    .buffer_ids
                    .iter()
                    .cloned()
                    .map(|buffer_id| self.get_buffer(buffer_id))
                    .collect::<GpuResult<Vec<_>>>()?;

                let waits = desc
                    .waits
                    .iter()
                    .map(|wait| {
                        Ok(VkSemaphoreWait {
                            semaphore: sync_gateway.get_semaphore(wait.semaphore)?,
                            stage_mask: wait.stage,
                        })
                    })
                    .collect::<GpuResult<Vec<_>>>()?;

                let signals = desc
                    .signals
                    .iter()
                    .cloned()
                    .map(|semaphore_id| sync_gateway.get_semaphore(semaphore_id))
                    .collect::<GpuResult<Vec<_>>>()?;

                Ok(VkSubmitDesc {
                    buffers,
                    waits,
                    signals,
                })
            })
            .collect::<GpuResult<Vec<_>>>()?;

        queue.submit(&self.context.device, &submits, fence)
    }

    fn bind_graphics_pipeline(
        &self,
        cmd: CommandBufferId,
        pipeline: contract::pipeline::GraphicsPipelineId,
    ) -> GpuResult<()> {
        todo!()
    }

    fn bind_compute_pipeline(
        &self,
        cmd: CommandBufferId,
        pipeline: ComputePipelineId,
    ) -> GpuResult<()> {
        let buffer = self.get_buffer(cmd)?;

        let gateways = self.gateways()?;
        let pipeline_gateway = gateways
            .pipeline
            .upgrade()
            .ok_or(GpuError::GatewaysNotSet)?;
        let pipeline = pipeline_gateway.get_compute_pipeline(pipeline)?;

        buffer.bind_compute_pipeline(pipeline);

        Ok(())
    }

    fn set_viewports(
        &self,
        cmd: CommandBufferId,
        first: u32,
        viewports: &[Viewport],
    ) -> GpuResult<()> {
        let buffer = self.get_buffer(cmd)?;
        buffer.set_viewport(first, viewports);

        Ok(())
    }

    fn set_scissors(
        &self,
        cmd: CommandBufferId,
        first: u32,
        scissors: &[Scissor],
    ) -> GpuResult<()> {
        let buffer = self.get_buffer(cmd)?;
        buffer.set_scissor(first, scissors);

        Ok(())
    }

    fn copy_buffer(
        &self,
        cmd: CommandBufferId,
        src: contract::resources::BufferId,
        dst: contract::resources::BufferId,
        regions: &[contract::command::BufferCopyRegion],
    ) -> GpuResult<()> {
        todo!()
    }

    fn copy_buffer_to_image(
        &self,
        cmd: CommandBufferId,
        src: contract::resources::BufferId,
        dst: contract::resources::ImageId,
        regions: &[contract::command::BufferImageCopyRegion],
    ) -> GpuResult<()> {
        todo!()
    }

    fn copy_image(
        &self,
        cmd: CommandBufferId,
        src: ImageId,
        src_layout: ImageLayout,
        dst: ImageId,
        dst_layout: ImageLayout,
        regions: &[ImageCopyRegion],
    ) -> GpuResult<()> {
        todo!()
    }

    fn blit(
        &self,
        cmd: CommandBufferId,
        src: ImageIdRef,
        src_layout: ImageLayout,
        dst: ImageIdRef,
        dst_layout: ImageLayout,
        regions: &[ImageBlitRegion],
        filter: Filter,
    ) -> GpuResult<()> {
        let buffer = self.get_buffer(cmd)?;
        let gateways = self.gateways()?;

        let resources_gateway = gateways
            .resources
            .upgrade()
            .ok_or(GpuError::GatewaysNotSet)?;

        let presenting_gateway = gateways
            .presenting
            .upgrade()
            .ok_or(GpuError::GatewaysNotSet)?;

        let src = ImageRef::from_id_ref(src, &resources_gateway, &presenting_gateway)?;
        let dst = ImageRef::from_id_ref(dst, &resources_gateway, &presenting_gateway)?;

        buffer.blit(src, src_layout, dst, dst_layout, regions, filter)
    }

    fn fill_buffer(
        &self,
        cmd: CommandBufferId,
        buffer_id: BufferId,
        offset: u64,
        size: u64,
        value: u8,
    ) -> GpuResult<()> {
        let cmd_buffer = self.get_buffer(cmd)?;
        let gateways = self.gateways()?;

        let resources_gateway = gateways
            .resources
            .upgrade()
            .ok_or(GpuError::GatewaysNotSet)?;

        let buffer = resources_gateway.get_buffer(buffer_id)?;

        cmd_buffer.fill_buffer(buffer, offset, size, value)
    }

    fn dispatch(&self, cmd: CommandBufferId, groups: kmath::Vec3u) -> GpuResult<()> {
        let buffer = self.get_buffer(cmd)?;

        unsafe {
            self.context.device.handle().cmd_dispatch(
                buffer.handle(),
                groups.x(),
                groups.y(),
                groups.z(),
            );
        }

        Ok(())
    }

    fn pipeline_barrier(&self, cmd: CommandBufferId, barriers: &Barriers) -> GpuResult<()> {
        let buffer = self.get_buffer(cmd)?;

        let gateways = self.gateways()?;
        let resources_gateway = gateways
            .resources
            .upgrade()
            .ok_or(GpuError::GatewaysNotSet)?;

        buffer.pipeline_barrier(&resources_gateway, barriers)
    }

    fn commit_resource_stage(&self, cmd: CommandBufferId, stage: &ResourceStage) -> GpuResult<()> {
        let buffer = self.get_buffer(cmd)?;
        let gateways = self.gateways()?;

        let resources_gateway = gateways
            .resources
            .upgrade()
            .ok_or(GpuError::GatewaysNotSet)?;

        let mut allocator = self.descriptor_allocator.lock();
        buffer.commit_resource_stage(&mut allocator, &resources_gateway, stage)
    }

    fn push_constants(&self, cmd: CommandBufferId, offset: u32, data: &[u8]) -> GpuResult<()> {
        todo!()
    }
}
