use std::sync::Arc;

use ash::vk;
use contract::{GpuError, GpuResult, resources::{AddressMode, BorderColor, CompareOp, Filter, MipmapMode, SamplerDesc}};

use crate::{FromContract, IntoVk, TryFromVk, VkContext, map_vk};

impl FromContract<Filter> for vk::Filter {
    fn from_contract(value: Filter) -> Self {
        match value {
            Filter::Nearest => vk::Filter::NEAREST,
            Filter::Linear => vk::Filter::LINEAR,
        }
    }
}
impl TryFromVk<vk::Filter> for Filter {
    fn try_from_vk(value: vk::Filter) -> GpuResult<Self> {
        match value {
            vk::Filter::NEAREST => Ok(Filter::Nearest),
            vk::Filter::LINEAR => Ok(Filter::Linear),
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("Filter {:?}", value),
            }),
        }
    }
}

impl FromContract<MipmapMode> for vk::SamplerMipmapMode {
    fn from_contract(value: MipmapMode) -> Self {
        match value {
            MipmapMode::Nearest => vk::SamplerMipmapMode::NEAREST,
            MipmapMode::Linear => vk::SamplerMipmapMode::LINEAR,
        }
    }
}
impl TryFromVk<vk::SamplerMipmapMode> for MipmapMode {
    fn try_from_vk(value: vk::SamplerMipmapMode) -> GpuResult<Self> {
        match value {
            vk::SamplerMipmapMode::NEAREST => Ok(MipmapMode::Nearest),
            vk::SamplerMipmapMode::LINEAR => Ok(MipmapMode::Linear),
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("MipmapMode {:?}", value),
            }),
        }
    }
}

impl FromContract<AddressMode> for vk::SamplerAddressMode {
    fn from_contract(value: AddressMode) -> Self {
        match value {
            AddressMode::Repeat => vk::SamplerAddressMode::REPEAT,
            AddressMode::MirroredRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
            AddressMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            AddressMode::ClampToBorder => vk::SamplerAddressMode::CLAMP_TO_BORDER,
        }
    }
}
impl TryFromVk<vk::SamplerAddressMode> for AddressMode {
    fn try_from_vk(value: vk::SamplerAddressMode) -> GpuResult<Self> {
        match value {
            vk::SamplerAddressMode::REPEAT => Ok(AddressMode::Repeat),
            vk::SamplerAddressMode::MIRRORED_REPEAT => Ok(AddressMode::MirroredRepeat),
            vk::SamplerAddressMode::CLAMP_TO_EDGE => Ok(AddressMode::ClampToEdge),
            vk::SamplerAddressMode::CLAMP_TO_BORDER => Ok(AddressMode::ClampToBorder),
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("AddressMode {:?}", value),
            }),
        }
    }
}

impl FromContract<CompareOp> for vk::CompareOp {
    fn from_contract(value: CompareOp) -> Self {
        match value {
            CompareOp::Never => vk::CompareOp::NEVER,
            CompareOp::Less => vk::CompareOp::LESS,
            CompareOp::Equal => vk::CompareOp::EQUAL,
            CompareOp::LessOrEqual => vk::CompareOp::LESS_OR_EQUAL,
            CompareOp::Greater => vk::CompareOp::GREATER,
            CompareOp::NotEqual => vk::CompareOp::NOT_EQUAL,
            CompareOp::GreaterOrEqual => vk::CompareOp::GREATER_OR_EQUAL,
            CompareOp::Always => vk::CompareOp::ALWAYS,
        }
    }
}
impl TryFromVk<vk::CompareOp> for CompareOp {
    fn try_from_vk(value: vk::CompareOp) -> GpuResult<Self> {
        match value {
            vk::CompareOp::NEVER => Ok(CompareOp::Never),
            vk::CompareOp::LESS => Ok(CompareOp::Less),
            vk::CompareOp::EQUAL => Ok(CompareOp::Equal),
            vk::CompareOp::LESS_OR_EQUAL => Ok(CompareOp::LessOrEqual),
            vk::CompareOp::GREATER => Ok(CompareOp::Greater),
            vk::CompareOp::NOT_EQUAL => Ok(CompareOp::NotEqual),
            vk::CompareOp::GREATER_OR_EQUAL => Ok(CompareOp::GreaterOrEqual),
            vk::CompareOp::ALWAYS => Ok(CompareOp::Always),
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("CompareOp {:?}", value),
            }),
        }
    }
}

impl FromContract<BorderColor> for vk::BorderColor {
    fn from_contract(value: BorderColor) -> Self {
        match value {
            BorderColor::TransparentBlack => vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
            BorderColor::OpaqueBlack => vk::BorderColor::FLOAT_OPAQUE_BLACK,
            BorderColor::OpaqueWhite => vk::BorderColor::FLOAT_OPAQUE_WHITE,
        }
    }
}
impl TryFromVk<vk::BorderColor> for BorderColor {
    fn try_from_vk(value: vk::BorderColor) -> GpuResult<Self> {
        match value {
            vk::BorderColor::FLOAT_TRANSPARENT_BLACK => Ok(BorderColor::TransparentBlack),
            vk::BorderColor::FLOAT_OPAQUE_BLACK => Ok(BorderColor::OpaqueBlack),
            vk::BorderColor::FLOAT_OPAQUE_WHITE => Ok(BorderColor::OpaqueWhite),
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("BorderColor {:?}", value),
            }),
        }
    }
}

pub struct VkSampler {
    context: Arc<VkContext>,
    handle: vk::Sampler,
}

impl VkSampler {
    pub fn new(context: Arc<VkContext>, desc: &SamplerDesc) -> GpuResult<Self> {
        let sampler_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: desc.mag_filter.into_vk(),
            min_filter: desc.min_filter.into_vk(),
            mipmap_mode: desc.mipmap_mode.into_vk(),
            address_mode_u: desc.address_mode.u.into_vk(),
            address_mode_v: desc.address_mode.v.into_vk(),
            address_mode_w: desc.address_mode.w.into_vk(),
            mip_lod_bias: desc.mip_lod_bias,
            anisotropy_enable: desc.anisotropy.is_some().into_vk(),
            max_anisotropy: desc.anisotropy.unwrap_or(1.0),
            compare_enable: desc.compare.is_some().into_vk(),
            compare_op: desc.compare.map(|cp| cp.into_vk()).unwrap_or(vk::CompareOp::ALWAYS),
            min_lod: desc.min_lod,
            max_lod: desc.max_lod,
            border_color: desc.border_color.into_vk(),
            unnormalized_coordinates: desc.unnormalized_coordinates.into_vk(),
            ..Default::default()
        };

        let handle = unsafe {
            context
                .device
                .handle()
                .create_sampler(&sampler_info, None)
                .map_err(map_vk)?
        };

        Ok(Self { context, handle })
    }

    pub fn handle(&self) -> vk::Sampler {
        self.handle
    }
}

impl Drop for VkSampler {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .handle()
                .destroy_sampler(self.handle, None);
        }
    }
}