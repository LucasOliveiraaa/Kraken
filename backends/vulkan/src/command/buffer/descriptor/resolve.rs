//! Resolves a `ResourceStage` (the resources the caller staged for the
//! currently-bound pipeline) against that pipeline's reflected
//! `VkPipelineLayout`, turning `StagingResource` ids into live resource
//! handles (`Element`), while validating that the stage actually matches
//! what the shader expects.
//!
//! TODO (unverified against files I couldn't see in this pass):
//! - The following `GpuError` variants are assumed; I'm proposing shapes for
//!   them based on the existing `NoPipelineBound` / `FailedSPIRVReflection`
//!   style, but haven't seen the real enum definition:
//!     - `DescriptorSetCountMismatch { expected: usize, found: usize }`
//!     - `DescriptorBindingCountMismatch { set: usize, expected: usize, found: usize }`
//!     - `DescriptorArrayCountMismatch { set: usize, binding: u32, expected: usize, found: usize }`
//!     - `DescriptorTypeMismatch { expected: DescriptorType, found: StagingResource }`
//!       — note `expected` is now `DescriptorType`, not a fabricated
//!       `StagingResource` (see bugs #3/#6 below).
//! - `layout_binding.count` is assumed to be a `DescriptorCount` enum with
//!   `Fixed(u32)` / `Variable { max: u32 }` variants, matching what the
//!   design doc described as "the reflected count". Adjust
//!   `validate_count` below if the real shape differs (e.g. different
//!   variant/field names, or this data living somewhere else on the
//!   binding/layout type).

use std::sync::Arc;

use contract::{
    GpuError, GpuResult,
    command::{ResourceStage, StagingResource},
};

use crate::{
    pipeline::{DescriptorBinding, DescriptorType, VkDescriptorSetLayout, VkPipelineLayout},
    resources::{VkBuffer, VkImageView, VkResourcesGateway, VkSampler},
};

/// One resolved descriptor-array element: a staged resource id turned into
/// a live handle, already checked against the layout's expected type.
pub(super) enum Element {
    Sampler(Arc<VkSampler>),
    SampledTexture(Arc<VkImageView>, Arc<VkSampler>),
    CombinedImageSampler(Arc<VkImageView>, Arc<VkSampler>),
    StorageImage(Arc<VkImageView>),
    StorageBuffer(Arc<VkBuffer>),
    UniformBuffer(Arc<VkBuffer>),
}

pub(super) struct SetDefinition<'a> {
    pub layout: &'a VkDescriptorSetLayout,
    /// (binding_index, elements...)
    pub bindings: Vec<(u32, Vec<Element>)>,
}

pub(super) fn resolve_sets<'a>(
    layout: &'a Arc<VkPipelineLayout>,
    stage: &ResourceStage,
    resources_gateway: &VkResourcesGateway,
) -> GpuResult<Vec<SetDefinition<'a>>> {
    let set_layouts = layout.descriptor_set_layouts();

    // Bug #1 (review): `zip` silently drops whichever side is longer, so a
    // stage missing a trailing set was previously accepted without
    // complaint — those descriptor sets would just never get materialized.
    // Validate the lengths match up front instead of relying on `zip`.
    if set_layouts.len() != stage.sets.len() {
        return Err(GpuError::DescriptorSetCountMismatch {
            expected: set_layouts.len(),
            found: stage.sets.len(),
        });
    }

    let mut sets = Vec::with_capacity(set_layouts.len());

    for (set_index, (set_layout, staged_set)) in
        set_layouts.iter().zip(stage.sets.iter()).enumerate()
    {
        let layout_bindings = set_layout.bindings();

        // Same truncation hazard one level down, for bindings within a set.
        if layout_bindings.len() != staged_set.len() {
            return Err(GpuError::DescriptorBindingCountMismatch {
                set: set_index,
                expected: layout_bindings.len(),
                found: staged_set.len(),
            });
        }

        let mut binding_defs: Vec<(u32, Vec<Element>)> = Vec::with_capacity(layout_bindings.len());

        for (layout_binding, staged_binding_array) in layout_bindings.iter().zip(staged_set.iter())
        {
            // Bug #2 (review): nothing previously compared the staged
            // array length against the binding's reflected descriptor
            // count. A `Fixed(4)` binding staged with 1 or 9 resources
            // sailed through unvalidated — writing more descriptors than a
            // binding's allocated slot count is out-of-bounds/undefined
            // behavior at the Vulkan level, not something that should ever
            // reach `vkUpdateDescriptorSets` un-checked.
            validate_count(set_index, layout_binding, staged_binding_array.len())?;

            let mut elements = Vec::with_capacity(staged_binding_array.len());

            for staging in staged_binding_array.iter() {
                elements.push(resolve_element(layout_binding, staging, resources_gateway)?);
            }

            binding_defs.push((layout_binding.binding, elements));
        }

        sets.push(SetDefinition {
            layout: set_layout,
            bindings: binding_defs,
        });
    }

    Ok(sets)
}

fn validate_count(
    set_index: usize,
    layout_binding: &DescriptorBinding,
    found: usize,
) -> GpuResult<()> {
    match layout_binding.count {
        crate::pipeline::DescriptorCount::Fixed(n) => {
            let expected = n as usize;
            if found != expected {
                return Err(GpuError::DescriptorArrayCountMismatch {
                    set: set_index,
                    binding: layout_binding.binding,
                    expected,
                    found,
                });
            }
        }
        _ => {}
    }

    Ok(())
}

fn resolve_element(
    layout_binding: &DescriptorBinding,
    staging: &StagingResource,
    resources_gateway: &VkResourcesGateway,
) -> GpuResult<Element> {
    Ok(match (staging, layout_binding.ty) {
        (StagingResource::Sampler(id), DescriptorType::Sampler) => {
            Element::Sampler(resources_gateway.get_sampler(*id)?)
        }
        (StagingResource::SampledTexture(view, samp), DescriptorType::SampledImage) => {
            Element::SampledTexture(
                resources_gateway.get_image_view(*view)?,
                resources_gateway.get_sampler(*samp)?,
            )
        }
        (StagingResource::SampledTexture(view, samp), DescriptorType::CombinedImageSampler) => {
            Element::CombinedImageSampler(
                resources_gateway.get_image_view(*view)?,
                resources_gateway.get_sampler(*samp)?,
            )
        }
        (StagingResource::StorageImage(view), DescriptorType::StorageImage) => {
            Element::StorageImage(resources_gateway.get_image_view(*view)?)
        }
        (StagingResource::StorageBuffer(buf), DescriptorType::StorageBuffer) => {
            Element::StorageBuffer(resources_gateway.get_buffer(*buf)?)
        }
        (StagingResource::UniformBuffer(buf), DescriptorType::UniformBuffer) => {
            Element::UniformBuffer(resources_gateway.get_buffer(*buf)?)
        }
        _ => {
            // Bugs #3 and #6 (review): this branch previously fabricated a
            // fake `StagingResource` (e.g. `SamplerId(0)`) purely to
            // describe "what type was expected" — which risked being
            // confused with a real id-0 resource later in logs/debuggers,
            // and outright panicked via `todo!()` for
            // `DescriptorType::AccelerationStructure`, since
            // `StagingResource` has no variant for it at all. That made
            // any shader reflecting an acceleration-structure binding
            // guaranteed to panic the first time it was exercised.
            //
            // Reporting the expected `DescriptorType` directly (rather than
            // a manufactured value of the wrong kind) fixes both: there's
            // nothing to fabricate, and no `StagingResource` variant is
            // needed for types it can't represent.
            return Err(GpuError::DescriptorTypeMismatch {
                expected: layout_binding.ty.to_string(),
                found: staging.clone(),
            });
        }
    })
}
