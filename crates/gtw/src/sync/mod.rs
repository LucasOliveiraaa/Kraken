mod fence;
mod semaphore;

use contract::GpuResult;
pub use fence::*;
pub use semaphore::*;

pub use contract::sync::FenceState;

create_gateway! {
    pub struct SyncGateway {
        contract::sync::SyncGateway
    }
}

impl SyncGateway {
    create_handle_methods! {
        fn create_fence() -> Fence;
        fn create_semaphore() -> Semaphore;
    }

    pub fn wait_for_fences(
        &self,
        fences: &[Fence],
        wait_for_all: bool,
        timeout_ns: Option<u64>,
    ) -> GpuResult<()> {
        let fence_ids: Vec<_> = fences.iter().map(|f| f.handle()).collect();
        self.raw()
            .wait_for_fences(&fence_ids, wait_for_all, timeout_ns)
    }

    pub fn reset_fences(&self, fences: &[Fence]) -> GpuResult<()> {
        let fence_ids: Vec<_> = fences.iter().map(|f| f.handle()).collect();
        self.raw().reset_fences(&fence_ids)
    }
}
