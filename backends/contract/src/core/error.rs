use thiserror::Error;

use crate::{
    DeviceId, DevicePropsId,
    command::{CommandBufferId, CommandPoolId, StagingResource},
    pipeline::ComputePipelineId,
    presenting::{SurfaceId, SwapchainId, SwapchainImageId},
    resources::{BufferId, ImageId, ImageViewId, SamplerId, ShaderId},
    sync::{FenceId, SemaphoreId},
};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GpuError {
    #[error("Library load error: {reason}")]
    LibraryLoadError { reason: String },

    #[error("Initialization failed: {reason}")]
    InitializationFailed { reason: String },

    #[error("Missing queue family: {reason}")]
    MissingQueueFamily { reason: String },

    #[error("Out of device memory")]
    OutOfDeviceMemory,
    #[error("Out of host memory")]
    OutOfHostMemory,

    #[error("Device lost")]
    DeviceLost,
    #[error("Gateways not set")]
    GatewaysNotSet,

    #[error("Invalid device props ID: {0}")]
    InvalidDevicePropsId(DevicePropsId),
    #[error("Invalid device ID: {0}")]
    InvalidDeviceId(DeviceId),

    #[error("Invalid buffer ID: {0}")]
    InvalidBufferId(BufferId),
    #[error("Invalid image ID: {0}")]
    InvalidImageId(ImageId),
    #[error("Invalid image view ID: {0}")]
    InvalidImageViewId(ImageViewId),
    #[error("Invalid shader ID: {0}")]
    InvalidShaderId(ShaderId),
    #[error("Invalid sampler ID: {0}")]
    InvalidSamplerId(SamplerId),

    #[error("Invalid surface ID: {0}")]
    InvalidSurfaceId(SurfaceId),
    #[error("Invalid swapchain ID: {0}")]
    InvalidSwapchainId(SwapchainId),
    #[error("Invalid swapchain image ID: {0}")]
    InvalidSwapchainImageId(SwapchainImageId),

    #[error("Invalid compute pipeline ID: {0}")]
    InvalidComputePipelineId(ComputePipelineId),

    #[error("Invalid command pool ID: {0}")]
    InvalidCommandPoolId(CommandPoolId),
    #[error("Invalid command buffer ID: {0}")]
    InvalidCommandBufferId(CommandBufferId),

    #[error("Invalid semaphore ID: {0}")]
    InvalidSemaphoreId(SemaphoreId),
    #[error("Invalid fence ID: {0}")]
    InvalidFenceId(FenceId),

    #[error("Buffer creation failed: {reason}")]
    BufferCreationFailed { reason: String },

    #[error("Invalid SPIR-V code: {reason}")]
    InvalidSPIRV { reason: String },
    #[error("Failed SPIR-V reflection: {reason}")]
    FailedSPIRVReflection { reason: String },

    #[error("Descriptor type mismatch: expected {expected}, found {found:?}")]
    DescriptorTypeMismatch {
        expected: String,
        found: StagingResource,
    },

    #[error("Descriptor set count mismatch: expected {expected}, found {found}")]
    DescriptorSetCountMismatch { expected: usize, found: usize },
    #[error("Descriptor binding count mismatch: set {set}, expected {expected}, found {found}")]
    DescriptorBindingCountMismatch {
        set: usize,
        expected: usize,
        found: usize,
    },
    #[error(
        "Descriptor array count mismatch: set {set}, binding {binding}, expected {expected}, found {found}"
    )]
    DescriptorArrayCountMismatch {
        set: usize,
        binding: u32,
        expected: usize,
        found: usize,
    },

    #[error("Invalid shader stage: {reason}")]
    InvalidShaderStage { reason: String },
    #[error("Invalid shader passes: {reason}")]
    InvalidShaderPasses { reason: String },

    #[error("No pipeline bound")]
    NoPipelineBound,

    #[error("Unsupported feature: {feature}")]
    UnsupportedFeature { feature: String },
    #[error("Invalid operation: {reason}")]
    InvalidOperation { reason: String },

    #[error("Backend error: code {code}, message: {message}")]
    Backend { code: i32, message: String },

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

pub type GpuResult<T> = Result<T, GpuError>;
