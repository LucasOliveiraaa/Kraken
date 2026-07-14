use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

use ash::vk;
use contract::{GpuError, GpuResult, resources::ShaderStage};

use crate::{FromContract, IntoVk, TryFromVk, VkContext, map_vk};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DescriptorType {
    Sampler,
    CombinedImageSampler,
    SampledImage,
    StorageImage,
    UniformBuffer,
    StorageBuffer,
    AccelerationStructure,
}
impl DescriptorType {
    pub fn to_string(&self) -> String {
        match self {
            DescriptorType::Sampler => "Sampler".to_string(),
            DescriptorType::CombinedImageSampler => "CombinedImageSampler".to_string(),
            DescriptorType::SampledImage => "SampledImage".to_string(),
            DescriptorType::StorageImage => "StorageImage".to_string(),
            DescriptorType::UniformBuffer => "UniformBuffer".to_string(),
            DescriptorType::StorageBuffer => "StorageBuffer".to_string(),
            DescriptorType::AccelerationStructure => "AccelerationStructure".to_string(),
        }
    }
}

impl FromContract<DescriptorType> for vk::DescriptorType {
    fn from_contract(value: DescriptorType) -> Self {
        match value {
            DescriptorType::Sampler => vk::DescriptorType::SAMPLER,
            DescriptorType::CombinedImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            DescriptorType::SampledImage => vk::DescriptorType::SAMPLED_IMAGE,
            DescriptorType::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
            DescriptorType::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
            DescriptorType::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
            DescriptorType::AccelerationStructure => vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
        }
    }
}
impl TryFromVk<vk::DescriptorType> for DescriptorType {
    fn try_from_vk(value: vk::DescriptorType) -> GpuResult<Self> {
        match value {
            vk::DescriptorType::SAMPLER => Ok(DescriptorType::Sampler),
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER => Ok(DescriptorType::CombinedImageSampler),
            vk::DescriptorType::SAMPLED_IMAGE => Ok(DescriptorType::SampledImage),
            vk::DescriptorType::STORAGE_IMAGE => Ok(DescriptorType::StorageImage),
            vk::DescriptorType::UNIFORM_BUFFER => Ok(DescriptorType::UniformBuffer),
            vk::DescriptorType::STORAGE_BUFFER => Ok(DescriptorType::StorageBuffer),
            vk::DescriptorType::ACCELERATION_STRUCTURE_KHR => {
                Ok(DescriptorType::AccelerationStructure)
            }
            _ => Err(GpuError::UnsupportedFeature {
                feature: format!("DescriptorType {:?}", value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DescriptorCount {
    Fixed(u32),
    Variable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DescriptorBinding {
    pub binding: u32,
    pub ty: DescriptorType,
    pub count: DescriptorCount,
    pub used_stages: Vec<ShaderStage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DescriptorSetLayout {
    pub bindings: BTreeMap<u32, DescriptorBinding>,
}
impl Hash for DescriptorSetLayout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bindings.len().hash(state);

        for (binding, desc) in &self.bindings {
            binding.hash(state);
            desc.hash(state);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PushConstant {
    pub size: u32,
    pub offset: u32,
    pub used_stages: Vec<ShaderStage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct LayoutInfo {
    pub descriptor_set_layouts: Vec<DescriptorSetLayout>,
    pub push_constants: Vec<PushConstant>,
}

fn to_stage_flags(stages: &Vec<ShaderStage>) -> vk::ShaderStageFlags {
    let mut flags = vk::ShaderStageFlags::empty();

    for stage in stages {
        flags |= match stage {
            ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
            ShaderStage::Compute => vk::ShaderStageFlags::COMPUTE,
        };
    }

    flags
}

pub struct VkDescriptorSetLayout {
    context: Arc<VkContext>,
    handle: vk::DescriptorSetLayout,
    bindings: Vec<DescriptorBinding>,
}

impl Clone for VkDescriptorSetLayout {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            handle: self.handle,
            bindings: self.bindings.clone(),
        }
    }
}

impl VkDescriptorSetLayout {
    pub fn from_handle(
        context: Arc<VkContext>,
        handle: vk::DescriptorSetLayout,
        bindings: Vec<DescriptorBinding>,
    ) -> Self {
        Self {
            context,
            handle,
            bindings,
        }
    }

    pub fn handle(&self) -> vk::DescriptorSetLayout {
        self.handle
    }

    pub fn bindings(&self) -> &[DescriptorBinding] {
        &self.bindings
    }
}

pub struct VkPipelineLayout {
    context: Arc<VkContext>,
    handle: vk::PipelineLayout,
    descriptor_set_layouts: Vec<VkDescriptorSetLayout>,
}

impl VkPipelineLayout {
    pub fn new(context: Arc<VkContext>, info: &LayoutInfo) -> GpuResult<Self> {
        let mut descriptor_set_layouts = vec![];

        let set_layouts = info
            .descriptor_set_layouts
            .iter()
            .map(|set| {
                let bindings = set
                    .bindings
                    .values()
                    .cloned()
                    .map(|binding| vk::DescriptorSetLayoutBinding {
                        binding: binding.binding,
                        descriptor_type: binding.ty.into_vk(),
                        descriptor_count: match binding.count {
                            DescriptorCount::Fixed(count) => count,
                            DescriptorCount::Variable => 1024,
                        },
                        p_immutable_samplers: std::ptr::null(),
                        stage_flags: to_stage_flags(&binding.used_stages),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>();

                let create_info = vk::DescriptorSetLayoutCreateInfo {
                    s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
                    p_next: std::ptr::null(),
                    flags: vk::DescriptorSetLayoutCreateFlags::empty(),
                    p_bindings: bindings.as_ptr(),
                    binding_count: bindings.len() as u32,
                    ..Default::default()
                };

                let handle = unsafe {
                    context
                        .device
                        .handle()
                        .create_descriptor_set_layout(&create_info, None)
                        .map_err(map_vk)?
                };

                descriptor_set_layouts.push(VkDescriptorSetLayout::from_handle(
                    context.clone(),
                    handle,
                    set.bindings.values().cloned().collect(),
                ));

                Ok(handle)
            })
            .collect::<GpuResult<Vec<_>>>()?;

        let push_constants = info
            .push_constants
            .iter()
            .map(|pc| vk::PushConstantRange {
                offset: pc.offset,
                size: pc.size,
                stage_flags: to_stage_flags(&pc.used_stages),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let layout_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: set_layouts.len() as u32,
            p_set_layouts: set_layouts.as_ptr(),
            push_constant_range_count: push_constants.len() as u32,
            p_push_constant_ranges: push_constants.as_ptr(),
            ..Default::default()
        };

        let handle = unsafe {
            context
                .device
                .handle()
                .create_pipeline_layout(&layout_info, None)
                .map_err(map_vk)?
        };
        Ok(Self {
            context: context.clone(),
            handle,
            descriptor_set_layouts,
        })
    }

    pub fn handle(&self) -> vk::PipelineLayout {
        self.handle
    }

    pub fn descriptor_set_layouts(&self) -> &[VkDescriptorSetLayout] {
        &self.descriptor_set_layouts
    }
}

impl Drop for VkPipelineLayout {
    fn drop(&mut self) {
        unsafe {
            for layout in &self.descriptor_set_layouts {
                self.context
                    .device
                    .handle()
                    .destroy_descriptor_set_layout(layout.handle, None);
            }
            self.context
                .device
                .handle()
                .destroy_pipeline_layout(self.handle, None);
        }
    }
}
