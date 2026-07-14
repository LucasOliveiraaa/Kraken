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

use std::sync::{Arc, OnceLock};

use contract::{
    GpuError, GpuResult,
    resources::{
        BufferDesc, BufferId, ImageDesc, ImageId, ImageViewDesc, ImageViewId, ResourcesGateway,
        SamplerDesc, SamplerId, ShaderDesc, ShaderId,
    },
};

use crate::{
    alloc::{Slab, VkAllocator},
    command::ImageRef,
    core::{VkContext, WeakGateways},
};

pub struct VkResourcesGateway {
    context: Arc<VkContext>,
    gateways: OnceLock<WeakGateways>,

    allocator: Arc<VkAllocator>,
    buffers: parking_lot::Mutex<Slab<Arc<VkBuffer>>>,
    images: parking_lot::Mutex<Slab<Arc<VkImage>>>,
    image_views: parking_lot::Mutex<Slab<Arc<VkImageView>>>,
    shaders: parking_lot::Mutex<Slab<Arc<VkShader>>>,
    samplers: parking_lot::Mutex<Slab<Arc<VkSampler>>>,
}

impl VkResourcesGateway {
    pub fn new(context: Arc<VkContext>) -> Self {
        Self {
            context: context.clone(),
            gateways: OnceLock::new(),

            allocator: Arc::new(VkAllocator::new(
                context.clone(),
                crate::alloc::AllocatorDesc {},
            )),
            buffers: parking_lot::Mutex::new(Slab::new()),
            images: parking_lot::Mutex::new(Slab::new()),
            image_views: parking_lot::Mutex::new(Slab::new()),
            shaders: parking_lot::Mutex::new(Slab::new()),
            samplers: parking_lot::Mutex::new(Slab::new()),
        }
    }

    pub fn set_gateways(&self, gateways: WeakGateways) {
        self.gateways.set(gateways).unwrap();
    }

    pub fn get_buffer(&self, buffer_id: BufferId) -> GpuResult<Arc<VkBuffer>> {
        let buffers = self.buffers.lock();
        let buffer = buffers
            .get(buffer_id.0 as usize)
            .ok_or(GpuError::InvalidBufferId(buffer_id))?;

        Ok(buffer.clone())
    }

    pub fn get_image(&self, image_id: ImageId) -> GpuResult<Arc<VkImage>> {
        let images = self.images.lock();
        let image = images
            .get(image_id.0 as usize)
            .ok_or(GpuError::InvalidImageId(image_id))?;

        Ok(image.clone())
    }

    pub fn get_image_view(&self, image_view_id: ImageViewId) -> GpuResult<Arc<VkImageView>> {
        let image_views = self.image_views.lock();
        let image_view = image_views
            .get(image_view_id.0 as usize)
            .ok_or(GpuError::InvalidImageViewId(image_view_id))?;

        Ok(image_view.clone())
    }

    pub fn get_shader(&self, shader_id: ShaderId) -> GpuResult<Arc<VkShader>> {
        let shaders = self.shaders.lock();
        let shader = shaders
            .get(shader_id.0 as usize)
            .ok_or(GpuError::InvalidShaderId(shader_id))?;

        Ok(shader.clone())
    }

    pub fn get_sampler(&self, sampler_id: SamplerId) -> GpuResult<Arc<VkSampler>> {
        let samplers = self.samplers.lock();
        let sampler = samplers
            .get(sampler_id.0 as usize)
            .ok_or(GpuError::InvalidSamplerId(sampler_id))?;

        Ok(sampler.clone())
    }
}

impl ResourcesGateway for VkResourcesGateway {
    fn create_buffer(&self, desc: &BufferDesc) -> GpuResult<BufferId> {
        let buffer = VkBuffer::new(self.context.clone(), self.allocator.clone(), desc)?;

        let mut buffers = self.buffers.lock();
        let buffer_id = BufferId(buffers.insert(Arc::new(buffer)) as u32);
        Ok(buffer_id)
    }

    fn destroy_buffer(&self, buffer_id: BufferId) -> GpuResult<()> {
        let mut buffers = self.buffers.lock();
        buffers.remove(buffer_id.0 as usize);
        Ok(())
    }

    fn write_buffer(&self, buffer_id: BufferId, offset: u64, data: &[u8]) -> GpuResult<()> {
        let buffer = self.get_buffer(buffer_id)?;
        buffer.write(offset, data)
    }

    fn read_buffer(&self, buffer_id: BufferId, offset: u64, data: &mut [u8]) -> GpuResult<()> {
        let buffer = self.get_buffer(buffer_id)?;
        buffer.read(offset, data)
    }

    fn create_image(&self, desc: &ImageDesc) -> GpuResult<ImageId> {
        let image = VkImage::new(self.context.clone(), self.allocator.clone(), desc)?;

        let mut images = self.images.lock();
        let image_id = ImageId(images.insert(Arc::new(image)) as u32);
        Ok(image_id)
    }

    fn destroy_image(&self, image_id: ImageId) -> GpuResult<()> {
        let mut images = self.images.lock();
        images.remove(image_id.0 as usize);
        Ok(())
    }

    fn create_image_view(&self, desc: &ImageViewDesc) -> GpuResult<ImageViewId> {
        let gateways = self.gateways.get().ok_or(GpuError::GatewaysNotSet)?;

        let resources_gateway = gateways
            .resources
            .upgrade()
            .ok_or(GpuError::GatewaysNotSet)?;
        let presenting_gateway = gateways
            .presenting
            .upgrade()
            .ok_or(GpuError::GatewaysNotSet)?;

        let image = ImageRef::from_id_ref(desc.image_id, &resources_gateway, &presenting_gateway)?;

        let image_view = VkImageView::new(self.context.clone(), image, desc)?;

        let mut image_views = self.image_views.lock();
        let image_view_id = ImageViewId(image_views.insert(Arc::new(image_view)) as u32);
        Ok(image_view_id)
    }

    fn destroy_image_view(&self, image_view_id: ImageViewId) -> GpuResult<()> {
        let mut image_views = self.image_views.lock();
        image_views.remove(image_view_id.0 as usize);
        Ok(())
    }

    fn create_shader(&self, desc: &ShaderDesc) -> GpuResult<ShaderId> {
        let shader = VkShader::new(self.context.clone(), desc)?;

        let mut shaders = self.shaders.lock();
        let shader_id = ShaderId(shaders.insert(Arc::new(shader)) as u32);
        Ok(shader_id)
    }

    fn destroy_shader(&self, shader_id: ShaderId) -> GpuResult<()> {
        let mut shaders = self.shaders.lock();
        shaders.remove(shader_id.0 as usize);
        Ok(())
    }

    fn create_sampler(&self, desc: &SamplerDesc) -> GpuResult<SamplerId> {
        let sampler = VkSampler::new(self.context.clone(), desc)?;

        let mut samplers = self.samplers.lock();
        let sampler_id = SamplerId(samplers.insert(Arc::new(sampler)) as u32);
        Ok(sampler_id)
    }

    fn destroy_sampler(&self, sampler_id: SamplerId) -> GpuResult<()> {
        let mut samplers = self.samplers.lock();
        samplers.remove(sampler_id.0 as usize);
        Ok(())
    }
}
