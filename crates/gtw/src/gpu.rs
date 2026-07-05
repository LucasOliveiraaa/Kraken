use std::sync::Arc;

use glow::HasContext;
use kmath::Vec2u;

use crate::present::Framebuffer;

pub type GpuContext = glow::Context;

bitflags::bitflags! {
    pub struct MemoryBarrier: u32 {
        const VERTEX_ATTRIB_ARRAY_BARRIER = glow::VERTEX_ATTRIB_ARRAY_BARRIER_BIT;
        const ELEMENT_ARRAY_BARRIER = glow::ELEMENT_ARRAY_BARRIER_BIT;
        const UNIFORM_BARRIER = glow::UNIFORM_BARRIER_BIT;
        const TEXTURE_FETCH_BARRIER = glow::TEXTURE_FETCH_BARRIER_BIT;
        const SHADER_IMAGE_ACCESS_BARRIER = glow::SHADER_IMAGE_ACCESS_BARRIER_BIT;
        const COMMAND_BARRIER = glow::COMMAND_BARRIER_BIT;
        const PIXEL_BUFFER_BARRIER = glow::PIXEL_BUFFER_BARRIER_BIT;
        const TEXTURE_UPDATE_BARRIER = glow::TEXTURE_UPDATE_BARRIER_BIT;
        const BUFFER_UPDATE_BARRIER = glow::BUFFER_UPDATE_BARRIER_BIT;
        const FRAMEBUFFER_BARRIER = glow::FRAMEBUFFER_BARRIER_BIT;
        const TRANSFORM_FEEDBACK_BARRIER = glow::TRANSFORM_FEEDBACK_BARRIER_BIT;
        const ATOMIC_COUNTER_BARRIER = glow::ATOMIC_COUNTER_BARRIER_BIT;
        const SHADER_STORAGE_BARRIER = glow::SHADER_STORAGE_BARRIER_BIT;
        const ALL_BARRIER_BITS = glow::ALL_BARRIER_BITS;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlitRegion {
    pub src_pos: Vec2u,
    pub src_size: Vec2u,
    pub dst_pos: Vec2u,
    pub dst_size: Vec2u,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BlitMask {
    Color,
    Depth,
    Stencil,
    ColorDepth,
    All,
}

impl From<BlitMask> for u32 {
    fn from(mask: BlitMask) -> Self {
        match mask {
            BlitMask::Color => glow::COLOR_BUFFER_BIT,
            BlitMask::Depth => glow::DEPTH_BUFFER_BIT,
            BlitMask::Stencil => glow::STENCIL_BUFFER_BIT,
            BlitMask::ColorDepth => glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT,
            BlitMask::All => {
                glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BlitFilter {
    Nearest,
    Linear,
}

impl From<BlitFilter> for u32 {
    fn from(filter: BlitFilter) -> Self {
        match filter {
            BlitFilter::Nearest => glow::NEAREST,
            BlitFilter::Linear => glow::LINEAR,
        }
    }
}

pub enum BlitTarget<'a> {
    Framebuffer(&'a Framebuffer),
    Screen,
}

pub struct Gpu {
    context: Arc<GpuContext>,
}

impl Gpu {
    pub fn new(context: GpuContext) -> Self {
        Self {
            context: Arc::new(context),
        }
    }

    pub fn context(&self) -> Arc<GpuContext> {
        Arc::clone(&self.context)
    }

    pub fn viewport(&self, pos: Vec2u, size: Vec2u) {
        unsafe {
            let gl = self.context();
            gl.viewport(
                pos.x() as i32,
                pos.y() as i32,
                size.x() as i32,
                size.y() as i32,
            );
        }
    }

    pub fn dispatch_compute(&self, x: u32, y: u32, z: u32) {
        unsafe {
            let gl = self.context();
            gl.dispatch_compute(x, y, z);
        }
    }

    pub fn memory_barrier(&self, barrier: MemoryBarrier) {
        unsafe {
            let gl = self.context();
            gl.memory_barrier(barrier.bits());
        }
    }

    pub fn blit(
        &self,
        src: &Framebuffer,
        dst: BlitTarget,
        region: BlitRegion,
        mask: BlitMask,
        filter: BlitFilter,
    ) {
        unsafe {
            let gl = self.context();

            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(src.handle()));
            match dst {
                BlitTarget::Framebuffer(fb) => {
                    gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(fb.handle()))
                }
                BlitTarget::Screen => gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None),
            }

            gl.blit_framebuffer(
                region.src_pos.x() as i32,
                region.src_pos.y() as i32,
                (region.src_pos.x() + region.src_size.x()) as i32,
                (region.src_pos.y() + region.src_size.y()) as i32,
                region.dst_pos.x() as i32,
                region.dst_pos.y() as i32,
                (region.dst_pos.x() + region.dst_size.x()) as i32,
                (region.dst_pos.y() + region.dst_size.y()) as i32,
                mask.into(),
                filter.into(),
            );

            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
        }
    }
}
