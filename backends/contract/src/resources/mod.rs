mod buffer;
mod image;
mod image_view;
mod sampler;
mod shader;

pub use buffer::*;
pub use image::*;
pub use image_view::*;
pub use sampler::*;
pub use shader::*;

use crate::GpuResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryLocation {
    GpuOnly,
    CpuToGpu,
    GpuToCpu,
}

pub trait ResourcesGateway: Sync + Send {
    /// Create a new buffer given its description.
    fn create_buffer(&self, desc: &BufferDesc) -> GpuResult<BufferId>;

    /// Destroy a buffer given its ID.
    fn destroy_buffer(&self, buffer_id: BufferId) -> GpuResult<()>;

    // Create a new image given its description.
    fn create_image(&self, desc: &ImageDesc) -> GpuResult<ImageId>;

    fn write_buffer(&self, buffer_id: BufferId, offset: u64, data: &[u8]) -> GpuResult<()>;
    fn read_buffer(&self, buffer_id: BufferId, offset: u64, data: &mut [u8]) -> GpuResult<()>;

    /// Destroy a image given its ID.
    fn destroy_image(&self, image_id: ImageId) -> GpuResult<()>;

    /// Create a new image view given its description.
    fn create_image_view(&self, desc: &ImageViewDesc) -> GpuResult<ImageViewId>;

    /// Destroy a image view given its ID.
    fn destroy_image_view(&self, image_view_id: ImageViewId) -> GpuResult<()>;

    /// Create a new shader given its description.
    fn create_shader(&self, desc: &ShaderDesc) -> GpuResult<ShaderId>;

    /// Destroy a shader given its ID.
    fn destroy_shader(&self, shader_id: ShaderId) -> GpuResult<()>;

    /// Create a new sampler given its description.
    fn create_sampler(&self, desc: &SamplerDesc) -> GpuResult<SamplerId>;

    /// Destroy a sampler given its ID.
    fn destroy_sampler(&self, sampler_id: SamplerId) -> GpuResult<()>;
}
