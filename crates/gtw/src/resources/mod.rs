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

pub use contract::resources::{
    AddressMode, BorderColor, BufferDesc, BufferUsage, ColorSpace, CompareOp, Filter, Format,
    ImageAspect, ImageDesc, ImageSubresourceRange, ImageLayout, ImageType, ImageUsage,
    ImageViewType, MemoryLocation, MipmapMode, SampleCount, SamplerAddressing, SamplerDesc,
    ShaderDesc, ShaderPass, ShaderStage, SurfaceFormat,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageRef {
    Image(Image),
    SwapchainImage(SwapchainImage),
}

impl ToContract for ImageRef {
    type Contract = contract::resources::ImageIdRef;

    fn to_contract(&self) -> Self::Contract {
        match self {
            ImageRef::Image(image) => Self::Contract::Image(image.handle()),
            ImageRef::SwapchainImage(swapchain_image) => {
                Self::Contract::SwapchainImage(swapchain_image.handle())
            }
        }
    }
}

use crate::{macros::ToContract, presenting::SwapchainImage};

create_gateway! {
    pub struct ResourcesGateway {
        contract::resources::ResourcesGateway
    }
}

pub struct ImageViewDesc {
    pub image: ImageRef,

    pub format: Format,
    pub view_type: ImageViewType,

    pub aspect: ImageAspect,

    pub base_mip: u32,
    pub level_count: u32,
    pub base_layer: u32,
    pub layer_count: u32,
}

impl ToContract for ImageViewDesc {
    type Contract = contract::resources::ImageViewDesc;

    fn to_contract(&self) -> Self::Contract {
        Self::Contract {
            image_id: self.image.to_contract(),
            format: self.format,
            view_type: self.view_type,
            aspect: self.aspect,
            base_mip: self.base_mip,
            level_count: self.level_count,
            base_layer: self.base_layer,
            layer_count: self.layer_count,
        }
    }
}

impl ResourcesGateway {
    create_handle_methods! {
        fn create_buffer(desc: &BufferDesc) -> Buffer;
        fn create_image(desc: &ImageDesc) -> Image;
        fn create_shader(desc: &ShaderDesc) -> Shader;
        fn create_sampler(desc: &SamplerDesc) -> Sampler;
        fn create_image_view(#[desc] desc: &ImageViewDesc) -> ImageView;
    }
}
