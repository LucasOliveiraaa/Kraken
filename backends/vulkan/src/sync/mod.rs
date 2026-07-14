use std::sync::{Arc, OnceLock};

use contract::{GpuError, GpuResult, sync::{FenceId, FenceState, SemaphoreId, SyncGateway}};

use crate::{alloc::Slab, core::{VkContext, WeakGateways}};

mod semaphore;
mod fence;

pub use semaphore::*;
pub use fence::*;

pub struct VkSyncGateway {
    context: Arc<VkContext>,
    gateways: OnceLock<WeakGateways>,

    semaphores: parking_lot::Mutex<Slab<Arc<VkSemaphore>>>,
    fences: parking_lot::Mutex<Slab<Arc<VkFence>>>,
}

impl VkSyncGateway {
    pub fn new(context: Arc<VkContext>) -> Self {
        Self {
            context: context.clone(),
            gateways: OnceLock::new(),

            semaphores: parking_lot::Mutex::new(Slab::new()),
            fences: parking_lot::Mutex::new(Slab::new()),
        }
    }

    pub fn set_gateways(&self, gateways: WeakGateways) {
        self.gateways.set(gateways).unwrap();
    }

    pub fn get_semaphore(&self, semaphore_id: SemaphoreId) -> GpuResult<Arc<VkSemaphore>> {
        let semaphores = self.semaphores.lock();
        let semaphore = semaphores
            .get(semaphore_id.0 as usize)
            .ok_or(GpuError::InvalidSemaphoreId(semaphore_id))?;

        Ok(semaphore.clone())
    }

    pub fn get_fence(&self, fence_id: FenceId) -> GpuResult<Arc<VkFence>> {
        let fences = self.fences.lock();
        let fence = fences
            .get(fence_id.0 as usize)
            .ok_or(GpuError::InvalidFenceId(fence_id))?;

        Ok(fence.clone())
    }
}

impl SyncGateway for VkSyncGateway {
    fn create_semaphore(&self) -> GpuResult<SemaphoreId> {
        let semaphore = VkSemaphore::new(self.context.clone())?;

        let mut semaphores = self.semaphores.lock();
        Ok(SemaphoreId(semaphores.insert(Arc::new(semaphore)) as u32))
    }

    fn destroy_semaphore(&self, semaphore_id: SemaphoreId) -> GpuResult<()> {
        let mut semaphores = self.semaphores.lock();
        if semaphores.remove(semaphore_id.0 as usize).is_none() {
            return Err(GpuError::InvalidSemaphoreId(semaphore_id));
        }

        Ok(())
    }

    fn create_fence(&self) -> GpuResult<FenceId> {
        let fence = VkFence::new(self.context.clone())?;

        let mut fences = self.fences.lock();
        Ok(FenceId(fences.insert(Arc::new(fence)) as u32))
    }

    fn destroy_fence(&self, fence_id: FenceId) -> GpuResult<()> {
        let mut fences = self.fences.lock();
        if fences.remove(fence_id.0 as usize).is_none() {
            return Err(GpuError::InvalidFenceId(fence_id));
        }

        Ok(())
    }

    fn wait_for_fences(
        &self,
        fence_ids: &[FenceId],
        wait_for_all: bool,
        timeout: Option<u64>,
    ) -> GpuResult<()>
    {
        let fences: Vec<Arc<VkFence>> = fence_ids
            .iter()
            .map(|id| self.get_fence(*id))
            .collect::<GpuResult<Vec<_>>>()?;

        VkFence::wait(self.context.clone(), &fences, wait_for_all, timeout)
    }

    fn reset_fences(&self, fence_ids: &[FenceId]) -> GpuResult<()> {
        let fences: Vec<Arc<VkFence>> = fence_ids
            .iter()
            .map(|id| self.get_fence(*id))
            .collect::<GpuResult<Vec<_>>>()?;

        VkFence::reset(self.context.clone(), &fences)
    }

    fn get_fence_state(&self, fence_id: FenceId) -> GpuResult<FenceState> {
        let fence = self.get_fence(fence_id)?;
        fence.get_state()
    }
}