use std::sync::Arc;

use kmath::Vec2u;

use gtw::{
    BlitRegion, BlitTarget, Gpu, present::Framebuffer, resources::{
        FilterMode, ImageAccess, Sampler, SamplerDesc, Texture, TextureDesc, TextureFormat,
        TextureKind, WrapMode,
    },
};

pub struct AccBuffer {
    gpu: Arc<Gpu>,

    write_fbo: Framebuffer,
    read_fbo: Framebuffer,

    write_tex: Arc<Texture>,
    read_tex: Arc<Texture>,
    sampler: Sampler,

    size: Vec2u,
}

impl AccBuffer {
    pub fn new(gpu: Arc<Gpu>, size: Vec2u) -> Result<Self, String> {
        let write_tex = Arc::new(Texture::new(
            gpu.clone(),
            TextureDesc {
                kind: TextureKind::Texture2D,
                format: TextureFormat::RGBA32F,
                size: size.extend(1),
                mip_levels: 1,
            },
        )?);

        let read_tex = Arc::new(Texture::new(
            gpu.clone(),
            TextureDesc {
                kind: TextureKind::Texture2D,
                format: TextureFormat::RGBA32F,
                size: size.extend(1),
                mip_levels: 1,
            },
        )?);

        let sampler = Sampler::new(
            gpu.clone(),
            &SamplerDesc {
                min_filter: FilterMode::Nearest,
                mag_filter: FilterMode::Nearest,
                wrap_u: WrapMode::ClampToEdge,
                wrap_v: WrapMode::ClampToEdge,
                ..Default::default()
            },
        )?;

        let mut write_fbo = Framebuffer::new(gpu.clone())?;
        write_fbo.attach_texture(
            gtw::present::Attachment::Color(0),
            Arc::clone(&write_tex),
            0,
        )?;

        let mut read_fbo = Framebuffer::new(gpu.clone())?;
        read_fbo.attach_texture(gtw::present::Attachment::Color(0), Arc::clone(&read_tex), 0)?;

        Ok(Self {
            gpu,
            write_fbo,
            read_fbo,
            write_tex,
            read_tex,
            sampler,
            size,
        })
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.read_tex, &mut self.write_tex);
        std::mem::swap(&mut self.read_fbo, &mut self.write_fbo);
    }

    pub fn bind_write_tex(&self) {
        self.write_tex.bind_image(0, ImageAccess::WriteOnly);
    }

    pub fn bind_read_tex(&self, unit: u32) {
        self.read_tex.bind(unit);
        self.sampler.bind(unit);
    }

    pub fn blit(&self) {
        self.gpu.blit(
            &self.read_fbo,
            BlitTarget::Screen,
            BlitRegion {
                src_pos: Vec2u::new(0, 0),
                src_size: self.size,
                dst_pos: Vec2u::new(0, 0),
                dst_size: self.size,
            },
            gtw::BlitMask::Color,
            gtw::BlitFilter::Nearest,
        );
    }
}
