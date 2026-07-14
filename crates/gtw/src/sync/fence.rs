use contract::{GpuResult, sync::{FenceId, FenceState}};

use crate::sync::SyncGateway;

create_handle_wrapper!(Fence, SyncGateway, FenceId, destroy_fence);

impl Fence {
    pub fn wait(&self, timeout_ns: Option<u64>) -> GpuResult<()> {
        self.raw_gtw().wait_for_fences(&[self.handle()], true, timeout_ns)
    }

    pub fn reset(&self) -> GpuResult<()> {
        self.raw_gtw().reset_fences(&[self.handle()])
    }

    pub fn get_state(&self) -> GpuResult<FenceState> {
        self.raw_gtw().get_fence_state(self.handle())
    }
}