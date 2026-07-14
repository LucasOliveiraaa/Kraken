use std::sync::Arc;

use ash::vk;
use contract::{GpuError, GpuResult, pipeline::PipelineStage, presenting::SwapchainImageId};

use crate::{
    FromContract, IntoVk, VkDevice, command::VkCommandBuffer, map_vk, presenting::VkSwapchain, sync::{VkFence, VkSemaphore},
};

fn score_family(props: &vk::QueueFamilyProperties, target_flags: vk::QueueFlags) -> u32 {
    let mut score = 0i32;
    if props.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
        score += if target_flags == vk::QueueFlags::GRAPHICS {
            10
        } else {
            -1
        };
    }
    if props.queue_flags.contains(vk::QueueFlags::COMPUTE) {
        score += if target_flags == vk::QueueFlags::COMPUTE {
            10
        } else {
            -1
        };
    }
    if props.queue_flags.contains(vk::QueueFlags::TRANSFER) {
        score += if target_flags == vk::QueueFlags::TRANSFER {
            10
        } else {
            -1
        };
    }
    score.max(0) as u32
}

fn find_best_family(
    props: &[vk::QueueFamilyProperties],
    target_flags: vk::QueueFlags,
) -> Option<u32> {
    props
        .iter()
        .enumerate()
        .map(|(i, p)| (i as u32, score_family(p, target_flags)))
        .filter(|(_, score)| *score > 0)
        .max_by_key(|(_, score)| *score)
        .map(|(i, _)| i)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueueFamilies {
    pub graphics: u32,
    pub compute: u32,
    pub transfer: u32,
}

impl QueueFamilies {
    pub fn find(instance: &ash::Instance, physical: vk::PhysicalDevice) -> GpuResult<Self> {
        let props = unsafe { instance.get_physical_device_queue_family_properties(physical) };

        let graphics = find_best_family(&props, vk::QueueFlags::GRAPHICS).ok_or(
            GpuError::MissingQueueFamily {
                reason: "no graphics queue found".to_string(),
            },
        )?;

        let compute = find_best_family(&props, vk::QueueFlags::COMPUTE).ok_or(
            GpuError::MissingQueueFamily {
                reason: "no compute queue found".to_string(),
            },
        )?;

        let transfer = find_best_family(&props, vk::QueueFlags::TRANSFER).ok_or(
            GpuError::MissingQueueFamily {
                reason: "no transfer queue found".to_string(),
            },
        )?;

        Ok(Self {
            graphics,
            compute,
            transfer,
        })
    }
}

pub struct Queues {
    pub graphics: Arc<VkQueue>,
    pub compute: Arc<VkQueue>,
    pub transfer: Arc<VkQueue>,
}

impl Queues {
    pub fn retrieve(device: &VkDevice, families: &QueueFamilies) -> Queues {
        unsafe {
            Self {
                graphics: Arc::new(VkQueue::from_raw(
                    device.handle().get_device_queue(families.graphics, 0),
                    families.graphics,
                )),
                compute: Arc::new(VkQueue::from_raw(
                    device.handle().get_device_queue(families.compute, 0),
                    families.compute,
                )),
                transfer: Arc::new(VkQueue::from_raw(
                    device.handle().get_device_queue(families.transfer, 0),
                    families.transfer,
                )),
            }
        }
    }
}

pub struct VkSemaphoreWait {
    pub semaphore: Arc<VkSemaphore>,
    pub stage_mask: PipelineStage,
}

pub struct VkSubmitDesc {
    pub buffers: Vec<Arc<VkCommandBuffer>>,
    pub waits: Vec<VkSemaphoreWait>,
    pub signals: Vec<Arc<VkSemaphore>>,
}

impl FromContract<PipelineStage> for vk::PipelineStageFlags2 {
    fn from_contract(stage: PipelineStage) -> Self {
        let mut flags = vk::PipelineStageFlags2::empty();
        if stage.contains(PipelineStage::TOP_OF_PIPE) {
            flags |= vk::PipelineStageFlags2::TOP_OF_PIPE;
        }
        if stage.contains(PipelineStage::DRAW_INDIRECT) {
            flags |= vk::PipelineStageFlags2::DRAW_INDIRECT;
        }
        if stage.contains(PipelineStage::VERTEX_INPUT) {
            flags |= vk::PipelineStageFlags2::VERTEX_INPUT;
        }
        if stage.contains(PipelineStage::VERTEX_SHADER) {
            flags |= vk::PipelineStageFlags2::VERTEX_SHADER;
        }
        if stage.contains(PipelineStage::FRAGMENT_SHADER) {
            flags |= vk::PipelineStageFlags2::FRAGMENT_SHADER;
        }
        if stage.contains(PipelineStage::EARLY_FRAGMENT_TESTS) {
            flags |= vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS;
        }
        if stage.contains(PipelineStage::LATE_FRAGMENT_TESTS) {
            flags |= vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS;
        }
        if stage.contains(PipelineStage::COLOR_ATTACHMENT_OUTPUT) {
            flags |= vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT;
        }
        if stage.contains(PipelineStage::COMPUTE_SHADER) {
            flags |= vk::PipelineStageFlags2::COMPUTE_SHADER;
        }
        if stage.contains(PipelineStage::TRANSFER) {
            flags |= vk::PipelineStageFlags2::TRANSFER;
        }
        if stage.contains(PipelineStage::BOTTOM_OF_PIPE) {
            flags |= vk::PipelineStageFlags2::BOTTOM_OF_PIPE;
        }
        flags
    }
}

pub struct VkQueue {
    handle: vk::Queue,
    family: u32,
    submit_lock: parking_lot::Mutex<()>,
}

impl VkQueue {
    pub fn from_raw(handle: vk::Queue, family: u32) -> Self {
        Self {
            handle,
            family,
            submit_lock: parking_lot::Mutex::new(()),
        }
    }

    pub fn handle(&self) -> vk::Queue {
        self.handle
    }

    pub fn family(&self) -> u32 {
        self.family
    }

    pub fn submit(
        &self,
        device: &VkDevice,
        submits: &[VkSubmitDesc],
        fence: Option<Arc<VkFence>>,
    ) -> GpuResult<()> {
        let _lock = self.submit_lock.lock();

        let submits = submits.iter().map(|sub| {
            let buffer_infos = sub.buffers.iter().map(|cb| {
                vk::CommandBufferSubmitInfo {
                    s_type: vk::StructureType::COMMAND_BUFFER_SUBMIT_INFO,
                    p_next: std::ptr::null(),
                    command_buffer: cb.handle(),
                    device_mask: 0,
                    ..Default::default()
                }
            }).collect::<Vec<_>>();

            let wait_infos = sub.waits.iter().map(|sem| {
                vk::SemaphoreSubmitInfo {
                    s_type: vk::StructureType::SEMAPHORE_SUBMIT_INFO,
                    p_next: std::ptr::null(),
                    semaphore: sem.semaphore.handle(),
                    stage_mask: sem.stage_mask.into_vk(),
                    device_index: 0,
                    ..Default::default()
                }
            }).collect::<Vec<_>>();

            let signal_infos = sub.signals.iter().map(|sem| {
                vk::SemaphoreSubmitInfo {
                    s_type: vk::StructureType::SEMAPHORE_SUBMIT_INFO,
                    p_next: std::ptr::null(),
                    semaphore: sem.handle(),
                    stage_mask: vk::PipelineStageFlags2::ALL_COMMANDS,
                    device_index: 0,
                    ..Default::default()
                }
            }).collect::<Vec<_>>();

            vk::SubmitInfo2 {
                s_type: vk::StructureType::SUBMIT_INFO_2,
                p_next: std::ptr::null(),
                flags: vk::SubmitFlags::empty(),
                command_buffer_info_count: buffer_infos.len() as u32,
                p_command_buffer_infos: buffer_infos.as_ptr(),
                wait_semaphore_info_count: wait_infos.len() as u32,
                p_wait_semaphore_infos: wait_infos.as_ptr(),
                signal_semaphore_info_count: signal_infos.len() as u32,
                p_signal_semaphore_infos: signal_infos.as_ptr(),
                ..Default::default()
            }
        }).collect::<Vec<_>>();

        unsafe {
            device
                .handle()
                .queue_submit2(
                    self.handle,
                    &submits,
                    fence
                        .as_ref()
                        .map(|f| f.handle())
                        .unwrap_or(vk::Fence::null()),
                )
                .map_err(map_vk)?;
        }
        Ok(())
    }

    pub fn present(
        &self,
        device: &ash::khr::swapchain::Device,
        swapchains: &[Arc<VkSwapchain>],
        image_indices: &[SwapchainImageId],
        wait_semaphores: &[Arc<VkSemaphore>],
    ) -> GpuResult<bool> {
        let swapchains = swapchains
            .iter()
            .map(|s| s.handle())
            .collect::<Vec<vk::SwapchainKHR>>();

        let image_indices = image_indices.iter().map(|i| i.0).collect::<Vec<u32>>();

        let wait_semaphores = wait_semaphores
            .iter()
            .map(|s| s.handle())
            .collect::<Vec<vk::Semaphore>>();

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: std::ptr::null(),
            p_results: std::ptr::null_mut(),
            swapchain_count: swapchains.len() as u32,
            p_image_indices: image_indices.as_ptr(),
            p_swapchains: swapchains.as_ptr(),
            p_wait_semaphores: wait_semaphores.as_ptr(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            ..Default::default()
        };

        let suboptimal = unsafe {
            device
                .queue_present(self.handle, &present_info)
                .map_err(map_vk)?
        };

        Ok(suboptimal)
    }
}
