use ash::vk;
use contract::GpuError;

pub fn map_vk(err: vk::Result) -> GpuError {
    match err {
        vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => GpuError::OutOfDeviceMemory,
        vk::Result::ERROR_OUT_OF_HOST_MEMORY => GpuError::OutOfHostMemory,
        vk::Result::ERROR_DEVICE_LOST => GpuError::DeviceLost,
        other => GpuError::Backend {
            code: other.as_raw(),
            message: format!("{other:?}"),
        },
    }
}

pub fn map_ash_load(err: ash::LoadingError) -> GpuError {
    match err {
        ash::LoadingError::MissingEntryPoint(_) => GpuError::InitializationFailed {
            reason: format!("missing entry point"),
        },
        ash::LoadingError::LibraryLoadFailure(err) => GpuError::InitializationFailed {
            reason: format!("{err:?}"),
        },
    }
}

pub fn map_alloc(err: gpu_allocator::AllocationError) -> GpuError {
    match err {
        gpu_allocator::AllocationError::OutOfMemory => GpuError::OutOfDeviceMemory,
        other => GpuError::BufferCreationFailed {
            reason: other.to_string(),
        },
    }
}

pub fn map_spirq(err: spirq::error::Error) -> GpuError {
    GpuError::FailedSPIRVReflection {
        reason: format!("{:#}", err),
    }
}
