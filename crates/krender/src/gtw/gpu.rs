use std::sync::Arc;

pub type GpuContext = glow::Context;

pub struct Gpu {
    context: Arc<GpuContext>,
}

impl Gpu {
    pub fn new(context: GpuContext) -> Self {
        Self { context: Arc::new(context) }
    }

    pub unsafe fn context(&self) -> Arc<GpuContext> {
        Arc::clone(&self.context)
    }
}