use std::sync::Arc;

use glow::HasContext;

use crate::Gpu;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FilterMode {
    Nearest,
    Linear,
}

impl From<FilterMode> for u32 {
    fn from(mode: FilterMode) -> Self {
        match mode {
            FilterMode::Nearest => glow::NEAREST,
            FilterMode::Linear => glow::LINEAR,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MipmapFilter {
    None,
    Nearest,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WrapMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}

impl From<WrapMode> for u32 {
    fn from(mode: WrapMode) -> Self {
        match mode {
            WrapMode::Repeat => glow::REPEAT,
            WrapMode::MirroredRepeat => glow::MIRRORED_REPEAT,
            WrapMode::ClampToEdge => glow::CLAMP_TO_EDGE,
            WrapMode::ClampToBorder => glow::CLAMP_TO_BORDER,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CompareOp {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}

impl From<CompareOp> for u32 {
    fn from(op: CompareOp) -> Self {
        match op {
            CompareOp::Never => glow::NEVER,
            CompareOp::Less => glow::LESS,
            CompareOp::Equal => glow::EQUAL,
            CompareOp::LessOrEqual => glow::LEQUAL,
            CompareOp::Greater => glow::GREATER,
            CompareOp::NotEqual => glow::NOTEQUAL,
            CompareOp::GreaterOrEqual => glow::GEQUAL,
            CompareOp::Always => glow::ALWAYS,
        }
    }
}

pub struct Sampler {
    gpu: Arc<Gpu>,

    handle: glow::NativeSampler,
}

pub struct SamplerDesc {
    pub min_filter: FilterMode,
    pub mag_filter: FilterMode,
    pub mipmap_filter: MipmapFilter,

    pub wrap_u: WrapMode,
    pub wrap_v: WrapMode,
    pub wrap_w: WrapMode,

    pub anisotropy: Option<f32>,
    pub border_color: [f32; 4],

    pub min_lod: f32,
    pub max_lod: f32,
    pub lod_bias: f32,

    pub compare: Option<CompareOp>,
}

impl Default for SamplerDesc {
    fn default() -> Self {
        Self {
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            mipmap_filter: MipmapFilter::None,

            wrap_u: WrapMode::Repeat,
            wrap_v: WrapMode::Repeat,
            wrap_w: WrapMode::Repeat,

            anisotropy: None,
            border_color: [0.0, 0.0, 0.0, 0.0],

            min_lod: -1000.0,
            max_lod: 1000.0,
            lod_bias: 0.0,

            compare: None,
        }
    }
}

impl SamplerDesc {
    fn gl_min_filter(&self) -> i32 {
        match (self.min_filter, self.mipmap_filter) {
            (FilterMode::Nearest, MipmapFilter::None) => glow::NEAREST as i32,
            (FilterMode::Linear, MipmapFilter::None) => glow::LINEAR as i32,

            (FilterMode::Nearest, MipmapFilter::Nearest) => glow::NEAREST_MIPMAP_NEAREST as i32,
            (FilterMode::Linear, MipmapFilter::Nearest) => glow::LINEAR_MIPMAP_NEAREST as i32,

            (FilterMode::Nearest, MipmapFilter::Linear) => glow::NEAREST_MIPMAP_LINEAR as i32,
            (FilterMode::Linear, MipmapFilter::Linear) => glow::LINEAR_MIPMAP_LINEAR as i32,
        }
    }

    fn gl_mag_filter(&self) -> i32 {
        match self.mag_filter {
            FilterMode::Nearest => glow::NEAREST as i32,
            FilterMode::Linear => glow::LINEAR as i32,
        }
    }
}

impl Sampler {
    pub fn new(gpu: Arc<Gpu>, desc: &SamplerDesc) -> Result<Self, String> {
        unsafe {
            let gl = gpu.context();

            let handle = gl.create_sampler()?;

            gl.sampler_parameter_i32(handle, glow::TEXTURE_MIN_FILTER, desc.gl_min_filter());
            gl.sampler_parameter_i32(handle, glow::TEXTURE_MAG_FILTER, desc.gl_mag_filter());

            gl.sampler_parameter_i32(
                handle,
                glow::TEXTURE_WRAP_S,
                Into::<u32>::into(desc.wrap_u) as i32,
            );
            gl.sampler_parameter_i32(
                handle,
                glow::TEXTURE_WRAP_T,
                Into::<u32>::into(desc.wrap_v) as i32,
            );
            gl.sampler_parameter_i32(
                handle,
                glow::TEXTURE_WRAP_R,
                Into::<u32>::into(desc.wrap_w) as i32,
            );

            if let Some(anisotropy) = desc.anisotropy {
                gl.sampler_parameter_i32(
                    handle,
                    glow::TEXTURE_MAX_ANISOTROPY_EXT,
                    anisotropy as i32,
                );
            }

            gl.sampler_parameter_f32(handle, glow::TEXTURE_MIN_LOD, desc.min_lod);
            gl.sampler_parameter_f32(handle, glow::TEXTURE_MAX_LOD, desc.max_lod);
            gl.sampler_parameter_f32(handle, glow::TEXTURE_LOD_BIAS, desc.lod_bias);

            gl.sampler_parameter_f32_slice(handle, glow::TEXTURE_BORDER_COLOR, &desc.border_color);

            if let Some(compare) = desc.compare {
                gl.sampler_parameter_i32(
                    handle,
                    glow::TEXTURE_COMPARE_MODE,
                    glow::COMPARE_REF_TO_TEXTURE as i32,
                );
                gl.sampler_parameter_i32(
                    handle,
                    glow::TEXTURE_COMPARE_FUNC,
                    Into::<u32>::into(compare) as i32,
                );
            } else {
                gl.sampler_parameter_i32(handle, glow::TEXTURE_COMPARE_MODE, glow::NONE as i32);
            }

            Ok(Self { gpu, handle })
        }
    }

    pub fn handle(&self) -> glow::NativeSampler {
        self.handle
    }

    pub fn bind(&self, unit: u32) {
        unsafe {
            let gl = self.gpu.context();
            gl.bind_sampler(unit, Some(self.handle));
        }
    }

    pub fn unbind(&self, unit: u32) {
        unsafe {
            let gl = self.gpu.context();
            gl.bind_sampler(unit, None);
        }
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            let gl = self.gpu.context();
            gl.delete_sampler(self.handle);
        }
    }
}
