//! Builds `AllocationRequest`s and `vk::WriteDescriptorSet`s from resolved
//! `SetDefinition`s (see `resolve.rs`).

use std::collections::HashMap;

use ash::vk;
use contract::{GpuError, GpuResult};

use crate::{
    command::descriptor::{AllocationRequest, VkDescriptorAllocator, VkDescriptorSet},
    pipeline::DescriptorType as PDescriptorType,
};

use super::resolve::{Element, SetDefinition};

pub(super) fn build_writes<'a>(
    allocator: &mut VkDescriptorAllocator,
    sets: &[SetDefinition<'a>],
) -> GpuResult<(
    Vec<vk::WriteDescriptorSet<'a>>,
    Vec<Vec<vk::DescriptorImageInfo>>,
    Vec<Vec<vk::DescriptorBufferInfo>>,
    Vec<VkDescriptorSet>,
)> {
    let requests = build_allocation_requests(sets);

    // Allocator is interiorly synchronized by the caller.
    let sets_handles = allocator.allocate_unchecked(&requests)?;

    let mut writes = Vec::new();
    // Storage for image/buffer infos so the pointers `writes` holds stay
    // valid. Ownership is returned to the caller, who must keep these alive
    // at least until `vkUpdateDescriptorSets` is called.
    let mut image_infos: Vec<Vec<vk::DescriptorImageInfo>> = Vec::new();
    let mut buffer_infos: Vec<Vec<vk::DescriptorBufferInfo>> = Vec::new();

    for (set_index, set_def) in sets.iter().enumerate() {
        let vk_set = sets_handles[set_index].handle();

        for (binding_num, elems) in set_def.bindings.iter() {
            if elems.is_empty() {
                return Err(GpuError::FailedSPIRVReflection {
                    reason: format!(
                        "empty descriptor array at set {} binding {}",
                        set_index, binding_num
                    ),
                });
            }

            let mut local_image_infos: Vec<vk::DescriptorImageInfo> = Vec::new();
            let mut local_buffer_infos: Vec<vk::DescriptorBufferInfo> = Vec::new();

            for elem in elems.iter() {
                match elem {
                    Element::Sampler(s) => local_image_infos.push(vk::DescriptorImageInfo {
                        sampler: s.handle(),
                        image_view: vk::ImageView::null(),
                        image_layout: vk::ImageLayout::UNDEFINED,
                    }),
                    Element::SampledTexture(view, samp) => {
                        local_image_infos.push(vk::DescriptorImageInfo {
                            sampler: samp.handle(),
                            image_view: view.handle(),
                            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                        })
                    }
                    Element::CombinedImageSampler(view, samp) => {
                        local_image_infos.push(vk::DescriptorImageInfo {
                            sampler: samp.handle(),
                            image_view: view.handle(),
                            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                        })
                    }
                    Element::StorageImage(view) => local_image_infos.push(vk::DescriptorImageInfo {
                        sampler: vk::Sampler::null(),
                        image_view: view.handle(),
                        image_layout: vk::ImageLayout::GENERAL,
                    }),
                    Element::StorageBuffer(buf) => {
                        local_buffer_infos.push(vk::DescriptorBufferInfo {
                            buffer: buf.handle(),
                            offset: 0,
                            range: vk::WHOLE_SIZE,
                        })
                    }
                    Element::UniformBuffer(buf) => {
                        local_buffer_infos.push(vk::DescriptorBufferInfo {
                            buffer: buf.handle(),
                            offset: 0,
                            range: vk::WHOLE_SIZE,
                        })
                    }
                }
            }

            let descriptor_count = elems.len() as u32;

            // Determine descriptor type from the first element (all
            // elements in a binding are guaranteed the same type by the
            // per-element validation in `resolve.rs`).
            let descriptor_type = match &elems[0] {
                Element::Sampler(_) => vk::DescriptorType::SAMPLER,
                Element::SampledTexture(_, _) => vk::DescriptorType::SAMPLED_IMAGE,
                Element::CombinedImageSampler(_, _) => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                Element::StorageImage(_) => vk::DescriptorType::STORAGE_IMAGE,
                Element::StorageBuffer(_) => vk::DescriptorType::STORAGE_BUFFER,
                Element::UniformBuffer(_) => vk::DescriptorType::UNIFORM_BUFFER,
            };

            let p_image_info = if !local_image_infos.is_empty() {
                image_infos.push(local_image_infos);
                image_infos.last().unwrap().as_ptr()
            } else {
                std::ptr::null()
            };

            let p_buffer_info = if !local_buffer_infos.is_empty() {
                buffer_infos.push(local_buffer_infos);
                buffer_infos.last().unwrap().as_ptr()
            } else {
                std::ptr::null()
            };

            writes.push(vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: std::ptr::null(),
                dst_set: vk_set,
                dst_binding: *binding_num,
                dst_array_element: 0,
                descriptor_count,
                descriptor_type,
                p_image_info,
                p_buffer_info,
                p_texel_buffer_view: std::ptr::null(),
                ..Default::default()
            });
        }
    }

    Ok((writes, image_infos, buffer_infos, sets_handles))
}

fn build_allocation_requests(sets: &[SetDefinition<'_>]) -> Vec<AllocationRequest> {
    sets.iter()
        .map(|set_def| {
            let mut counts: HashMap<PDescriptorType, u32> = HashMap::new();

            for (_binding_idx, elems) in &set_def.bindings {
                for elem in elems.iter() {
                    let ty = match elem {
                        Element::Sampler(_) => PDescriptorType::Sampler,
                        Element::SampledTexture(_, _) => PDescriptorType::SampledImage,
                        Element::CombinedImageSampler(_, _) => {
                            PDescriptorType::CombinedImageSampler
                        }
                        Element::StorageImage(_) => PDescriptorType::StorageImage,
                        Element::StorageBuffer(_) => PDescriptorType::StorageBuffer,
                        Element::UniformBuffer(_) => PDescriptorType::UniformBuffer,
                    };
                    *counts.entry(ty).or_insert(0) += 1;
                }
            }

            AllocationRequest {
                set_layout: std::sync::Arc::new(set_def.layout.clone()),
                count: counts,
            }
        })
        .collect()
}