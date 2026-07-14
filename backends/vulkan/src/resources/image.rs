use std::{mem, sync::Arc};

use ash::vk::{self, ImageCreateInfo};
use contract::{
    GpuError, GpuResult,
    resources::{
        ColorSpace, Format, ImageDesc, ImageLayout, ImageType, ImageUsage, MemoryLocation,
        SampleCount,
    },
};

use crate::{
    TryFromVk,
    alloc::{AllocationDesc, VkAllocation, VkAllocator},
    core::{FromContract, FromVk, IntoVk, VkContext},
    map_vk,
};

impl FromContract<Format> for vk::Format {
    fn from_contract(value: Format) -> Self {
        match value {
            Format::Rgba8Unorm => vk::Format::R8G8B8A8_UNORM,
            Format::Rgba8Srgb => vk::Format::R8G8B8A8_SRGB,
            Format::Bgra8Unorm => vk::Format::B8G8R8A8_UNORM,
            Format::Bgra8Srgb => vk::Format::B8G8R8A8_SRGB,
            Format::R32Float => vk::Format::R32_SFLOAT,
            Format::R32Uint => vk::Format::R32_UINT,
            Format::R32Sint => vk::Format::R32_SINT,
            Format::R16Float => vk::Format::R16_SFLOAT,
            Format::R16Uint => vk::Format::R16_UINT,
            Format::R16Sint => vk::Format::R16_SINT,
            Format::R8Unorm => vk::Format::R8_UNORM,
            Format::R8Uint => vk::Format::R8_UINT,
            Format::R8Sint => vk::Format::R8_SINT,
            _ => unreachable!("Unsupported image format: {:?}", value),
        }
    }
}
impl TryFromVk<vk::Format> for Format {
    fn try_from_vk(value: vk::Format) -> GpuResult<Self> {
        match value {
            vk::Format::R8G8B8A8_UNORM => Ok(Format::Rgba8Unorm),
            vk::Format::R8G8B8A8_SRGB => Ok(Format::Rgba8Srgb),
            vk::Format::B8G8R8A8_UNORM => Ok(Format::Bgra8Unorm),
            vk::Format::B8G8R8A8_SRGB => Ok(Format::Bgra8Srgb),
            vk::Format::R32_SFLOAT => Ok(Format::R32Float),
            vk::Format::R32_UINT => Ok(Format::R32Uint),
            vk::Format::R32_SINT => Ok(Format::R32Sint),
            vk::Format::R16_SFLOAT => Ok(Format::R16Float),
            vk::Format::R16_UINT => Ok(Format::R16Uint),
            vk::Format::R16_SINT => Ok(Format::R16Sint),
            vk::Format::R8_UNORM => Ok(Format::R8Unorm),
            vk::Format::R8_UINT => Ok(Format::R8Uint),
            vk::Format::R8_SINT => Ok(Format::R8Sint),
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("Format {:?}", value),
            }),
        }
    }
}

impl FromContract<ColorSpace> for vk::ColorSpaceKHR {
    fn from_contract(value: ColorSpace) -> Self {
        match value {
            ColorSpace::SrgbNonlinear => vk::ColorSpaceKHR::SRGB_NONLINEAR,
            ColorSpace::DisplayP3Nonlinear => vk::ColorSpaceKHR::DISPLAY_P3_NONLINEAR_EXT,
            ColorSpace::ExtendedSrgbLinear => vk::ColorSpaceKHR::EXTENDED_SRGB_LINEAR_EXT,
            ColorSpace::Hdr10St2084 => vk::ColorSpaceKHR::HDR10_ST2084_EXT,
            ColorSpace::Hdr10Hlg => vk::ColorSpaceKHR::HDR10_HLG_EXT,
            _ => unreachable!("Unsupported color space: {:?}", value),
        }
    }
}
impl TryFromVk<vk::ColorSpaceKHR> for ColorSpace {
    fn try_from_vk(value: vk::ColorSpaceKHR) -> GpuResult<Self> {
        match value {
            vk::ColorSpaceKHR::SRGB_NONLINEAR => Ok(ColorSpace::SrgbNonlinear),
            vk::ColorSpaceKHR::DISPLAY_P3_NONLINEAR_EXT => Ok(ColorSpace::DisplayP3Nonlinear),
            vk::ColorSpaceKHR::EXTENDED_SRGB_LINEAR_EXT => Ok(ColorSpace::ExtendedSrgbLinear),
            vk::ColorSpaceKHR::HDR10_ST2084_EXT => Ok(ColorSpace::Hdr10St2084),
            vk::ColorSpaceKHR::HDR10_HLG_EXT => Ok(ColorSpace::Hdr10Hlg),
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("ColorSpace {:?}", value),
            }),
        }
    }
}

impl FromContract<ImageUsage> for vk::ImageUsageFlags {
    fn from_contract(value: ImageUsage) -> Self {
        let mut flags = vk::ImageUsageFlags::empty();

        if value.contains(ImageUsage::TRANSFER_SRC) {
            flags |= vk::ImageUsageFlags::TRANSFER_SRC;
        }
        if value.contains(ImageUsage::TRANSFER_DST) {
            flags |= vk::ImageUsageFlags::TRANSFER_DST;
        }
        if value.contains(ImageUsage::SAMPLED) {
            flags |= vk::ImageUsageFlags::SAMPLED;
        }
        if value.contains(ImageUsage::STORAGE) {
            flags |= vk::ImageUsageFlags::STORAGE;
        }
        if value.contains(ImageUsage::COLOR_ATTACHMENT) {
            flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
        }
        if value.contains(ImageUsage::DEPTH_STENCIL_ATTACHMENT) {
            flags |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
        }

        flags
    }
}
impl FromVk<vk::ImageUsageFlags> for ImageUsage {
    fn from_vk(value: vk::ImageUsageFlags) -> Self {
        let mut flags = ImageUsage::empty();

        if value.contains(vk::ImageUsageFlags::TRANSFER_SRC) {
            flags |= ImageUsage::TRANSFER_SRC;
        }
        if value.contains(vk::ImageUsageFlags::TRANSFER_DST) {
            flags |= ImageUsage::TRANSFER_DST;
        }
        if value.contains(vk::ImageUsageFlags::SAMPLED) {
            flags |= ImageUsage::SAMPLED;
        }
        if value.contains(vk::ImageUsageFlags::STORAGE) {
            flags |= ImageUsage::STORAGE;
        }
        if value.contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
            flags |= ImageUsage::COLOR_ATTACHMENT;
        }
        if value.contains(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
            flags |= ImageUsage::DEPTH_STENCIL_ATTACHMENT;
        }

        flags
    }
}

impl FromContract<ImageType> for vk::ImageType {
    fn from_contract(value: ImageType) -> Self {
        match value {
            ImageType::Image1D => vk::ImageType::TYPE_1D,
            ImageType::Image2D => vk::ImageType::TYPE_2D,
            ImageType::Image3D => vk::ImageType::TYPE_3D,
        }
    }
}
impl TryFromVk<vk::ImageType> for ImageType {
    fn try_from_vk(value: vk::ImageType) -> GpuResult<Self> {
        match value {
            vk::ImageType::TYPE_1D => Ok(ImageType::Image1D),
            vk::ImageType::TYPE_2D => Ok(ImageType::Image2D),
            vk::ImageType::TYPE_3D => Ok(ImageType::Image3D),
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("ImageType {:?}", value),
            }),
        }
    }
}

impl FromContract<SampleCount> for vk::SampleCountFlags {
    fn from_contract(value: SampleCount) -> Self {
        match value {
            SampleCount::One => vk::SampleCountFlags::TYPE_1,
            SampleCount::Two => vk::SampleCountFlags::TYPE_2,
            SampleCount::Four => vk::SampleCountFlags::TYPE_4,
            SampleCount::Eight => vk::SampleCountFlags::TYPE_8,
            SampleCount::Sixteen => vk::SampleCountFlags::TYPE_16,
        }
    }
}
impl TryFromVk<vk::SampleCountFlags> for SampleCount {
    fn try_from_vk(value: vk::SampleCountFlags) -> GpuResult<Self> {
        match value {
            vk::SampleCountFlags::TYPE_1 => Ok(SampleCount::One),
            vk::SampleCountFlags::TYPE_2 => Ok(SampleCount::Two),
            vk::SampleCountFlags::TYPE_4 => Ok(SampleCount::Four),
            vk::SampleCountFlags::TYPE_8 => Ok(SampleCount::Eight),
            vk::SampleCountFlags::TYPE_16 => Ok(SampleCount::Sixteen),
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("SampleCount {:?}", value),
            }),
        }
    }
}

impl FromContract<ImageLayout> for vk::ImageLayout {
    fn from_contract(value: ImageLayout) -> Self {
        match value {
            ImageLayout::Undefined => vk::ImageLayout::UNDEFINED,
            ImageLayout::General => vk::ImageLayout::GENERAL,
            ImageLayout::ColorAttachmentOptimal => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::DepthStencilAttachmentOptimal => {
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            }
            ImageLayout::DepthStencilReadOnlyOptimal => {
                vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL
            }
            ImageLayout::ShaderReadOnlyOptimal => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ImageLayout::TransferSrcOptimal => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            ImageLayout::TransferDstOptimal => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            ImageLayout::PresentSrc => vk::ImageLayout::PRESENT_SRC_KHR,
        }
    }
}
impl TryFromVk<vk::ImageLayout> for ImageLayout {
    fn try_from_vk(value: vk::ImageLayout) -> GpuResult<Self> {
        match value {
            vk::ImageLayout::UNDEFINED => Ok(ImageLayout::Undefined),
            vk::ImageLayout::GENERAL => Ok(ImageLayout::General),
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => Ok(ImageLayout::ColorAttachmentOptimal),
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL => {
                Ok(ImageLayout::DepthStencilAttachmentOptimal)
            }
            vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL => {
                Ok(ImageLayout::DepthStencilReadOnlyOptimal)
            }
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => Ok(ImageLayout::ShaderReadOnlyOptimal),
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL => Ok(ImageLayout::TransferSrcOptimal),
            vk::ImageLayout::TRANSFER_DST_OPTIMAL => Ok(ImageLayout::TransferDstOptimal),
            vk::ImageLayout::PRESENT_SRC_KHR => Ok(ImageLayout::PresentSrc),
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("ImageLayout {:?}", value),
            }),
        }
    }
}
pub struct VkImage {
    context: Arc<VkContext>,
    handle: vk::Image,
    allocator: Arc<VkAllocator>,
    allocation: VkAllocation,
    desc: ImageDesc,
}

impl VkImage {
    pub fn new(
        context: Arc<VkContext>,
        allocator: Arc<VkAllocator>,
        desc: &ImageDesc,
    ) -> GpuResult<Self> {
        let image_info = ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::ImageCreateFlags::empty(),
            format: desc.format.into_vk(),
            usage: desc.usage.into_vk(),
            extent: desc.size.into_vk(),
            mip_levels: desc.mip_levels,
            array_layers: desc.array_layers,
            samples: desc.sample_count.into_vk(),
            tiling: match desc.location {
                MemoryLocation::GpuOnly => vk::ImageTiling::OPTIMAL,
                MemoryLocation::CpuToGpu | MemoryLocation::GpuToCpu => vk::ImageTiling::LINEAR,
            },
            image_type: desc.image_type.into_vk(),
            initial_layout: vk::ImageLayout::UNDEFINED,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
            ..Default::default()
        };

        let handle = unsafe {
            context
                .device
                .handle()
                .create_image(&image_info, None)
                .map_err(map_vk)?
        };

        let mut allocation = allocator
            .allocate(AllocationDesc {
                name: &desc.name,
                linear: image_info.tiling == vk::ImageTiling::LINEAR,
                location: desc.location,
                requirements: unsafe {
                    context
                        .device
                        .handle()
                        .get_image_memory_requirements(handle)
                },
            })
            .map_err(|e| {
                unsafe {
                    context.device.handle().destroy_image(handle, None);
                }
                e
            })?;

        unsafe {
            context
                .device
                .handle()
                .bind_image_memory(handle, allocation.memory(), allocation.offset())
                .map_err(|e| {
                    allocator.deallocate(mem::take(&mut allocation)).ok();
                    context.device.handle().destroy_image(handle, None);
                    map_vk(e)
                })?;
        }

        Ok(Self {
            context,
            handle,
            allocator,
            allocation,
            desc: desc.clone(),
        })
    }

    pub fn handle(&self) -> vk::Image {
        self.handle
    }

    pub fn desc(&self) -> &ImageDesc {
        &self.desc
    }
}

impl Drop for VkImage {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .handle()
                .destroy_image(self.handle, None);
        }
        self.allocator
            .deallocate(mem::take(&mut self.allocation))
            .ok();
    }
}
