mod fence;
mod semaphore;

pub use fence::*;
pub use semaphore::*;

use crate::GpuResult;

pub trait SyncGateway: Sync + Send {
    /// Create a new semaphore.
    fn create_semaphore(&self) -> GpuResult<SemaphoreId>;

    /// Destroy a semaphore given its ID.
    fn destroy_semaphore(&self, semaphore_id: SemaphoreId) -> GpuResult<()>;

    /// Create a new fence.
    fn create_fence(&self) -> GpuResult<FenceId>;

    /// Destroy a fence given its ID.
    fn destroy_fence(&self, fence_id: FenceId) -> GpuResult<()>;

    /// Wait for a fence to be signaled, optionally with a timeout in nanoseconds.
    /// If `wait_for_all` is true, the function will wait for all fences to be signaled; otherwise, it will return when any fence is signaled.
    fn wait_for_fences(
        &self,
        fence_ids: &[FenceId],
        wait_for_all: bool,
        timeout: Option<u64>,
    ) -> GpuResult<()>;

    /// Reset a fence to the unsignaled state.
    fn reset_fences(&self, fence_ids: &[FenceId]) -> GpuResult<()>;

    /// Get the current state of a fence.
    fn get_fence_state(&self, fence_id: FenceId) -> GpuResult<FenceState>;
}
