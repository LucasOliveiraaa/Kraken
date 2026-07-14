use crate::{
    pipeline::{
        DescriptorBinding, DescriptorCount, DescriptorSetLayout, DescriptorType, LayoutInfo,
        PushConstant,
    },
    resources::VkShader,
};
use contract::{
    GpuError, GpuResult,
    resources::{ShaderPass, ShaderStage},
};
use spirq::{entry_point::EntryPoint, ty::Type, var::Variable};
use std::collections::BTreeMap;
use std::sync::Arc;

fn from_spirq_stage(stage: spirq::spirv::ExecutionModel) -> GpuResult<ShaderStage> {
    match stage {
        spirq::spirv::ExecutionModel::Vertex => Ok(ShaderStage::Vertex),
        spirq::spirv::ExecutionModel::Fragment => Ok(ShaderStage::Fragment),
        spirq::spirv::ExecutionModel::GLCompute => Ok(ShaderStage::Compute),
        e => Err(GpuError::UnsupportedFeature {
            feature: format!("Shader Stage {:?}", e),
        }),
    }
}

impl TryFrom<spirq::ty::DescriptorType> for DescriptorType {
    type Error = GpuError;

    fn try_from(value: spirq::ty::DescriptorType) -> GpuResult<Self> {
        match value {
            spirq::ty::DescriptorType::Sampler() => Ok(DescriptorType::Sampler),
            spirq::ty::DescriptorType::CombinedImageSampler() => {
                Ok(DescriptorType::CombinedImageSampler)
            }
            spirq::ty::DescriptorType::SampledImage() => Ok(DescriptorType::SampledImage),
            spirq::ty::DescriptorType::StorageImage(_) => Ok(DescriptorType::StorageImage),
            spirq::ty::DescriptorType::UniformBuffer() => Ok(DescriptorType::UniformBuffer),
            spirq::ty::DescriptorType::StorageBuffer(_) => Ok(DescriptorType::StorageBuffer),
            spirq::ty::DescriptorType::AccelStruct() => Ok(DescriptorType::AccelerationStructure),
            _ => Err(GpuError::FailedSPIRVReflection {
                reason: format!("unsupported descriptor type {:?}", value),
            }),
        }
    }
}

/// Merges a newly-reflected push constant range into the accumulated list.
///
/// Per the Vulkan spec, if two stages declare push constant ranges whose
/// byte ranges overlap, they must be expressed as a single
/// `VkPushConstantRange` covering the union of both ranges, with the
/// union of both stages' flags — not as two separate, possibly-overlapping
/// ranges. Ranges that don't overlap at all are kept as distinct entries.
fn merge_push_constant(push_constants: &mut Vec<PushConstant>, new_pc: PushConstant) {
    let new_end = new_pc.offset + new_pc.size;

    for existing in push_constants.iter_mut() {
        let existing_end = existing.offset + existing.size;

        // Half-open interval overlap test: [offset, offset+size).
        let overlaps = new_pc.offset < existing_end && existing.offset < new_end;

        if overlaps {
            let merged_offset = existing.offset.min(new_pc.offset);
            let merged_end = existing_end.max(new_end);

            existing.offset = merged_offset;
            existing.size = merged_end - merged_offset;
            existing.used_stages.extend(new_pc.used_stages);
            return;
        }
    }

    push_constants.push(new_pc);
}

pub fn parse_entrypoint(
    reflection: EntryPoint,
    last_layout: Option<LayoutInfo>,
) -> GpuResult<LayoutInfo> {
    let last_layout = last_layout.unwrap_or_default();

    let mut descriptor_set_layouts: BTreeMap<u32, DescriptorSetLayout> = BTreeMap::new();

    // Carry over descriptor set layouts accumulated from previous stages.
    // `last_layout.descriptor_set_layouts` was itself built from a
    // contiguous-keys BTreeMap (validated below), so its Vec position
    // *is* the original set index — `enumerate` recovers it directly,
    // rather than reconstructing it from the map's growing length as a
    // side effect of insertion order.
    for (set_idx, set) in last_layout.descriptor_set_layouts.into_iter().enumerate() {
        descriptor_set_layouts.insert(set_idx as u32, set);
    }

    // Carry over push constants accumulated from previous stages. These
    // still need to go through the same overlap-merge logic as
    // this stage's own push constants, in case this stage's range
    // overlaps one carried over from an earlier stage.
    let mut push_constants: Vec<PushConstant> = Vec::new();
    for pc in last_layout.push_constants {
        merge_push_constant(&mut push_constants, pc);
    }

    let stage = from_spirq_stage(reflection.exec_model)?;

    for var in reflection.vars {
        match var {
            Variable::Descriptor {
                desc_bind,
                desc_ty,
                nbind,
                ..
            } => {
                let binding = DescriptorBinding {
                    binding: desc_bind.bind(),
                    count: if nbind == 0 {
                        DescriptorCount::Variable
                    } else {
                        DescriptorCount::Fixed(nbind)
                    },
                    ty: desc_ty.try_into()?,
                    used_stages: vec![stage],
                };

                let layout = descriptor_set_layouts
                    .entry(desc_bind.set())
                    .or_insert_with(DescriptorSetLayout::default);

                match layout.bindings.get_mut(&binding.binding) {
                    // Same binding index reflected again (e.g. a UBO
                    // shared between vertex and fragment stages) — merge
                    // stage flags instead of treating it as a conflict.
                    Some(existing) => {
                        if existing.ty != binding.ty || existing.count != binding.count {
                            return Err(GpuError::FailedSPIRVReflection {
                                reason: format!(
                                    "binding {} in descriptor set {} is declared with \
                                     incompatible type/count across stages ({:?}/{:?} vs {:?}/{:?})",
                                    binding.binding,
                                    desc_bind.set(),
                                    existing.ty,
                                    existing.count,
                                    binding.ty,
                                    binding.count,
                                ),
                            });
                        }

                        existing.used_stages.extend(binding.used_stages);
                    }
                    None => {
                        layout.bindings.insert(binding.binding, binding);
                    }
                }
            }

            Variable::PushConstant { name, ty } => {
                let pc_name = name.clone().unwrap_or_else(|| "<unnamed>".to_string());

                let Type::Struct(struct_ty) = &ty else {
                    return Err(GpuError::FailedSPIRVReflection {
                        reason: format!("push constant '{}' is not a struct type", pc_name),
                    });
                };

                if struct_ty.members.is_empty() {
                    return Err(GpuError::FailedSPIRVReflection {
                        reason: format!("push constant '{}' has no members", pc_name),
                    });
                }

                let mut min_offset = u32::MAX;
                let mut max_end = 0u32;

                for member in &struct_ty.members {
                    let member_name = member
                        .name
                        .clone()
                        .unwrap_or_else(|| "<unnamed>".to_string());

                    let member_offset =
                        member
                            .offset
                            .ok_or_else(|| GpuError::FailedSPIRVReflection {
                                reason: format!(
                                    "push constant '{}' member '{}' has no Offset decoration",
                                    pc_name, member_name
                                ),
                            })? as u32;

                    let member_size =
                        member
                            .ty
                            .nbyte()
                            .ok_or_else(|| GpuError::FailedSPIRVReflection {
                                reason: format!(
                                    "failed to get size of push constant '{}' member '{}'",
                                    pc_name, member_name
                                ),
                            })? as u32;

                    min_offset = min_offset.min(member_offset);
                    max_end = max_end.max(member_offset + member_size);
                }

                merge_push_constant(
                    &mut push_constants,
                    PushConstant {
                        offset: min_offset,
                        size: max_end - min_offset,
                        used_stages: vec![stage],
                    },
                );
            }

            _ => {}
        }
    }

    // This check is only meaningful *after* merging: two stages
    // declaring overlapping ranges must have already collapsed into one
    // entry by this point. What remains here is a check against a real
    // business rule (this pipeline layout allows only one push constant
    // range total), not an artifact of how many `PushConstant` variables
    // were seen across however many reflection calls contributed to it.
    if push_constants.len() > 1 {
        return Err(GpuError::FailedSPIRVReflection {
            reason: format!(
                "only one push constant range per pipeline layout is allowed, found {} \
                 non-overlapping ranges: {:?}",
                push_constants.len(),
                push_constants
                    .iter()
                    .map(|pc| (pc.offset, pc.size))
                    .collect::<Vec<_>>()
            ),
        });
    }

    let keys = descriptor_set_layouts.keys().cloned().collect::<Vec<_>>();
    if !keys.is_empty() {
        let first = keys[0];
        let last = keys[keys.len() - 1];

        if first != 0 || last != (keys.len() as u32 - 1) {
            return Err(GpuError::FailedSPIRVReflection {
                reason: format!(
                    "descriptor set indices are not contiguous, found sets: {:?}",
                    keys
                ),
            });
        }
    }

    Ok(LayoutInfo {
        descriptor_set_layouts: descriptor_set_layouts.values().cloned().collect(),
        push_constants,
    })
}

pub fn get_layout_info(shader: Arc<VkShader>, passes: &[ShaderPass]) -> GpuResult<LayoutInfo> {
    let entry_points = shader.reflect(passes)?;

    let mut layout_info: Option<LayoutInfo> = None;

    for entry in entry_points {
        layout_info = Some(parse_entrypoint(entry, layout_info)?);
    }

    Ok(layout_info.unwrap_or_default())
}
