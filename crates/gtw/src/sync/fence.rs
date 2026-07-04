use std::sync::Arc;

use glow::HasContext;

use crate::Gpu;

pub struct Fence {
    gpu: Arc<Gpu>,

    handle: glow::NativeFence,
}

impl Fence {
    pub fn new(gpu: Arc<Gpu>) -> Result<Self, String> {
        unsafe {
            let gl = gpu.context();
            let handle = gl.fence_sync(glow::SYNC_GPU_COMMANDS_COMPLETE, 0)?;

            Ok(Self { gpu, handle })
        }
    }

    pub fn handle(&self) -> glow::NativeFence {
        self.handle
    }

    pub fn wait_client(&self, flush: bool, timeout_ns: Option<u64>) -> Result<(), String> {
        unsafe {
            let gl = self.gpu.context();

            let timeout = timeout_ns.unwrap_or(glow::TIMEOUT_IGNORED);

            let result = gl.client_wait_sync(
                self.handle,
                if flush {
                    glow::SYNC_FLUSH_COMMANDS_BIT
                } else {
                    0
                },
                timeout as i32,
            );

            match result {
                glow::ALREADY_SIGNALED | glow::CONDITION_SATISFIED => Ok(()),
                glow::TIMEOUT_EXPIRED => Err("Fence wait timed out".to_string()),
                glow::WAIT_FAILED => Err("Fence wait failed".to_string()),
                _ => Err("Unknown fence wait result".to_string()),
            }
        }
    }

    pub fn wait_for_gpu(&self) {
        unsafe {
            let gl = self.gpu.context();
            gl.wait_sync(self.handle, 0, glow::TIMEOUT_IGNORED);
        }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            let gl = self.gpu.context();
            gl.delete_sync(self.handle);
        }
    }
}