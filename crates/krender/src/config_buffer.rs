use std::sync::Arc;

use bytemuck::{Pod, Zeroable};

use gtw::{
    Gpu,
    resources::{Buffer, BufferDesc, BufferTarget, BufferUsage},
};

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
    gpu: Arc<Gpu>,

    ssbo: Buffer,
    binding: u32,

    data: GlConfig,
    pub dirty: bool,
}

impl ConfigBuffer {
    pub fn new(gpu: Arc<Gpu>, binding: u32) -> Result<Self, String> {
        let data = GlConfig {
            view_mode: ViewMode::Normal as u32,
            exposure: 1.0,
            newton_max_steps: 5,
            max_bounces: 8,
        };

        let ssbo = Buffer::new_with_data(
            gpu.clone(),
            &[data],
            BufferDesc {
                size: 0,
                target: BufferTarget::ShaderStorageBuffer,
                usage: BufferUsage::DynamicDraw,
            },
        )?;

        ssbo.bind_base(binding);

        Ok(Self {
            gpu,
            ssbo,
            binding,
            data,
            dirty: false,
        })
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

    pub fn upload_if_dirty(&mut self) {
        if !self.dirty {
            return;
        }

        println!("Uploading config buffer: {:?}", self.data);

        self.ssbo.upload_data(0, &[self.data]);
        self.ssbo.bind_base(self.binding);

        self.dirty = false;
    }
}
