#![allow(unsafe_op_in_unsafe_fn)]

use std::sync::Arc;

use bytemuck::{bytes_of, Pod, Zeroable};
use glow::{Context, HasContext};

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
            0 => Self::Normal,
            1 => Self::NewtonIterations,
            2 => Self::RayBounces,
            3 => Self::QuadNodeVisits,
            4 => Self::Normals,
            5 => Self::Albedo,
            6 => Self::Roughness,
            _ => Self::Depth,
        }
    }

    pub fn next(self) -> Self {
        Self::from_index(self as u32 + 1)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::NewtonIterations => "Newton Iterations",
            Self::RayBounces => "Ray Bounces",
            Self::QuadNodeVisits => "Quad Node Visits",
            Self::Normals => "Normals",
            Self::Albedo => "Albedo",
            Self::Roughness => "Roughness",
            Self::Depth => "Depth",
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        (0..Self::COUNT).map(Self::from_index)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct GlConfig {
    view_mode: u32,
    exposure: f32,
    newton_max_steps: u32,
    max_bounces: u32,
}

pub struct ConfigBuffer {
    gl: Arc<Context>,

    ssbo: glow::NativeBuffer,
    binding: u32,

    data: GlConfig,
    pub dirty: bool,
}

impl ConfigBuffer {
    pub unsafe fn new(
        gl: Arc<Context>,
        binding: u32,
    ) -> Result<Self, String> {
        let data = GlConfig {
            view_mode: ViewMode::Normal as u32,
            exposure: 1.0,
            newton_max_steps: 5,
            max_bounces: 8,
        };

        let ssbo = gl.create_buffer()?;

        gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(ssbo));

        gl.buffer_data_size(
            glow::SHADER_STORAGE_BUFFER,
            std::mem::size_of::<GlConfig>() as i32,
            glow::DYNAMIC_DRAW,
        );

        gl.bind_buffer_base(
            glow::SHADER_STORAGE_BUFFER,
            binding,
            Some(ssbo),
        );

        gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, None);

        let mut this = Self {
            gl,
            ssbo,
            binding,
            data,
            dirty: true,
        };

        this.upload_if_dirty();

        Ok(this)
    }

    pub fn view_mode(&self) -> ViewMode {
        ViewMode::from_index(self.data.view_mode)
    }

    pub fn exposure(&self) -> f32 {
        self.data.exposure
    }

    pub fn max_newton_steps(&self) -> u32 {
        self.data.newton_max_steps
    }

    pub fn max_bounces(&self) -> u32 {
        self.data.max_bounces
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        if self.data.view_mode != mode as u32 {
            self.data.view_mode = mode as u32;
            self.dirty = true;
        }
    }

    pub fn set_exposure(&mut self, exposure: f32) {
        if self.data.exposure != exposure {
            self.data.exposure = exposure;
            self.dirty = true;
        }
    }

    pub fn set_max_newton_steps(&mut self, steps: u32) {
        if self.data.newton_max_steps != steps {
            self.data.newton_max_steps = steps;
            self.dirty = true;
        }
    }

    pub fn set_max_bounces(&mut self, bounces: u32) {
        if self.data.max_bounces != bounces {
            self.data.max_bounces = bounces;
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

        println!("Uploading config buffer: {:?}", self.data);

        self.gl.bind_buffer(
            glow::SHADER_STORAGE_BUFFER,
            Some(self.ssbo),
        );

        self.gl.buffer_sub_data_u8_slice(
            glow::SHADER_STORAGE_BUFFER,
            0,
            bytes_of(&self.data),
        );

        self.gl.bind_buffer_base(
            glow::SHADER_STORAGE_BUFFER,
            self.binding,
            Some(self.ssbo),
        );

        self.dirty = false;
    }
}

impl Drop for ConfigBuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.ssbo);
        }
    }
}