use std::{collections::HashMap, sync::Arc};

use glow::HasContext;

use crate::{
    Gpu,
    resources::{Texture, TextureKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Attachment {
    Color(u32),
    Depth,
    Stencil,
    DepthStencil,
}

impl From<Attachment> for u32 {
    fn from(attachment: Attachment) -> Self {
        match attachment {
            Attachment::Color(i) => glow::COLOR_ATTACHMENT0 + i,
            Attachment::Depth => glow::DEPTH_ATTACHMENT,
            Attachment::Stencil => glow::STENCIL_ATTACHMENT,
            Attachment::DepthStencil => glow::DEPTH_STENCIL_ATTACHMENT,
        }
    }
}

pub struct Framebuffer {
    gpu: Arc<Gpu>,

    handle: glow::NativeFramebuffer,

    attachments: HashMap<Attachment, Arc<Texture>>,
}

impl Framebuffer {
    pub fn new(gpu: Arc<Gpu>) -> Result<Self, String> {
        unsafe {
            let gl = gpu.context();
            let handle = gl.create_framebuffer()?;

            Ok(Self {
                gpu,
                handle,
                attachments: HashMap::new(),
            })
        }
    }

    pub fn handle(&self) -> glow::NativeFramebuffer {
        self.handle
    }

    pub fn attach_texture(
        &mut self,
        attachment: Attachment,
        texture: Arc<Texture>,
        level: i32,
    ) -> Result<(), String> {
        unsafe {
            let gl = self.gpu.context();

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.handle));

            match texture.kind() {
                TextureKind::Texture2D => {
                    gl.framebuffer_texture_2d(
                        glow::FRAMEBUFFER,
                        attachment.into(),
                        glow::TEXTURE_2D,
                        Some(texture.handle()),
                        level,
                    );
                }

                TextureKind::Texture3D => {
                    gl.framebuffer_texture_3d(
                        glow::FRAMEBUFFER,
                        attachment.into(),
                        glow::TEXTURE_3D,
                        Some(texture.handle()),
                        level,
                        0,
                    );
                }

                _ => return Err("Unsupported texture kind".into()),
            }

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        self.attachments.insert(attachment, texture);

        Ok(())
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            let gl = self.gpu.context();
            gl.delete_framebuffer(self.handle);
        }
    }
}
