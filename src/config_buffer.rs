#![allow(unsafe_op_in_unsafe_fn)]

use crate::gl_util::create_empty_ssbo;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewMode {
    Normal = 0,
    NewtonIterations = 1,
    RayBounces = 2,
    QuadNodeVisits = 3,
    Normals = 4,
    Albedo = 5,
    Roughness = 6,
    Depth = 7,
}

impl ViewMode {
    pub const COUNT: u32 = 8;

    pub fn from_index(i: u32) -> Self {
        match i % Self::COUNT {
            0 => ViewMode::Normal,
            1 => ViewMode::NewtonIterations,
            2 => ViewMode::RayBounces,
            3 => ViewMode::QuadNodeVisits,
            4 => ViewMode::Normals,
            5 => ViewMode::Albedo,
            6 => ViewMode::Roughness,
            _ => ViewMode::Depth,
        }
    }

    pub fn next(self) -> Self {
        Self::from_index(self as u32 + 1)
    }

    pub fn label(&self) -> &'static str {
        match self {
            ViewMode::Normal => "Normal",
            ViewMode::NewtonIterations => "Newton Iterations",
            ViewMode::RayBounces => "Ray Bounces",
            ViewMode::QuadNodeVisits => "Quad Node Visits",
            ViewMode::Normals => "Normals",
            ViewMode::Albedo => "Albedo",
            ViewMode::Roughness => "Roughness",
            ViewMode::Depth => "Depth",
        }
    }
}

/// Mirrors the `Config` SSBO's std430 layout exactly — 16 bytes, no
/// implicit padding surprises.
#[repr(C)]
struct GlConfig {
    view_mode: u32,
    exposure: f32,
    newton_max_steps: u32,
    max_bounces: u32,
}

pub struct ConfigBuffer {
    ssbo: u32,
    binding: u32,
    data: GlConfig,
    /// Set whenever `data` changes and hasn't been uploaded yet.
    pub dirty: bool,
}

impl ConfigBuffer {
    pub unsafe fn new(binding: u32) -> Self {
        let data = GlConfig {
            view_mode: ViewMode::Normal as u32,
            exposure: 1.0,
            newton_max_steps: 5,
            max_bounces: 8,
        };
        let ssbo = create_empty_ssbo(
            std::mem::size_of::<GlConfig>() as isize,
            binding,
            gl::DYNAMIC_DRAW,
        );
        let mut this = Self {
            ssbo,
            binding,
            data,
            dirty: true,
        };
        this.upload_if_dirty();
        this
    }

    pub fn view_mode(&self) -> ViewMode {
        ViewMode::from_index(self.data.view_mode)
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        if self.data.view_mode != mode as u32 {
            self.data.view_mode = mode as u32;
            self.dirty = true;
        }
    }

    pub fn cycle_view_mode(&mut self) {
        self.set_view_mode(self.view_mode().next());
    }

    pub unsafe fn upload_if_dirty(&mut self) {
        if !self.dirty {
            return;
        }
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.ssbo);
        gl::BufferSubData(
            gl::SHADER_STORAGE_BUFFER,
            0,
            std::mem::size_of::<GlConfig>() as isize,
            &self.data as *const GlConfig as *const _,
        );
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, self.binding, self.ssbo);
        self.dirty = false;
    }

    pub unsafe fn destroy(&self) {
        gl::DeleteBuffers(1, &self.ssbo);
    }
}