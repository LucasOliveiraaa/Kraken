mod buffer;
mod pool;

pub use buffer::*;
pub use pool::*;

pub use contract::command::{
    AccessFlags, BufferCopyRegion, BufferImageCopyRegion, ImageBlitRegion, ImageCopyRegion,
    MemoryBarrier, QueueType, ResourceStage, StagingResource,
};
use contract::{GpuResult, pipeline::PipelineStage};

use crate::{
    macros::ToContract,
    sync::{Fence, Semaphore},
};

pub struct SemaphoreWait {
    pub semaphore: Semaphore,
    pub stage: PipelineStage,
}
impl ToContract for SemaphoreWait {
    type Contract = contract::command::SemaphoreWait;

    fn to_contract(&self) -> Self::Contract {
        Self::Contract {
            semaphore: self.semaphore.handle(),
            stage: self.stage,
        }
    }
}

pub struct SubmitDesc {
    pub buffers: Vec<CommandBuffer>,
    pub waits: Vec<SemaphoreWait>,
    pub signals: Vec<Semaphore>,
}
impl ToContract for SubmitDesc {
    type Contract = contract::command::SubmitDesc;

    fn to_contract(&self) -> Self::Contract {
        Self::Contract {
            buffer_ids: self.buffers.iter().map(|b| b.handle()).collect(),
            waits: self.waits.iter().map(|w| w.to_contract()).collect(),
            signals: self.signals.iter().map(|s| s.handle()).collect(),
        }
    }
}

create_gateway! {
    pub struct CommandGateway {
        contract::command::CommandGateway
    }
}

impl CommandGateway {
    create_handle_methods! {
        fn create_command_pool(desc: &CommandPoolDesc) -> CommandPool;
        fn alloc_command_buffers(#[wrapper] pool: CommandPool, count: u32; forward: [pool]) -> [CommandBuffer];
    }

    pub fn free_command_buffers(
        &self,
        pool: CommandPool,
        buffers: Vec<CommandBuffer>,
    ) -> GpuResult<()> {
        let handles = buffers.iter().map(|b| b.handle()).collect::<Vec<_>>();
        self.raw().free_command_buffers(pool.handle(), &handles)
    }

    /// Submits command buffers to a queue for execution.
    /// 
    /// # Arguments
    /// * `queue_type` - The type of queue to submit to
    /// * `descs` - A slice of SubmitDesc, each describing a submission
    /// * `fence` - An optional fence to signal when the submission is complete
    /// 
    /// # Returns
    /// A GpuResult indicating success or failure of the submission.
    /// 
    /// Possible errors:
    /// * `GpuError::DeviceLost` - The device was lost during submission
    /// 
    /// # Examples
    /// ```ignore
    /// device.command().submit(
    ///     QueueType::Graphics,
    ///     &[SubmitDesc {
    ///         buffers: vec![cmd.clone()],
    ///         waits: vec![SemaphoreWait {
    ///             semaphore: wait_semaphore.clone(),
    ///             stage: PipelineStage::COLOR_ATTACHMENT_OUTPUT,
    ///         }],
    ///         signals: vec![signal_semaphore.clone()],
    ///     }],
    ///     None,
    /// )?;
    /// ```
    pub fn submit(
        &self,
        queue_type: QueueType,
        descs: &[SubmitDesc],
        fence: Option<Fence>,
    ) -> GpuResult<()> {
        self.raw().submit(
            queue_type,
            &descs
                .iter()
                .map(|desc| desc.to_contract())
                .collect::<Vec<_>>(),
            fence.map(|f| f.handle()),
        )
    }
}
