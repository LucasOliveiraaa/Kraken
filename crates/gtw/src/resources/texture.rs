use std::sync::Arc;

use glow::HasContext;
use kmath::Vec3u;

use crate::Gpu;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TextureKind {
    Texture1D,
    Texture2D,
    Texture3D,
    CubeMap,
    CubeArray,
    Texture2DArray,
}

impl From<TextureKind> for u32 {
    fn from(kind: TextureKind) -> Self {
        match kind {
            TextureKind::Texture1D => glow::TEXTURE_1D,
            TextureKind::Texture2D => glow::TEXTURE_2D,
            TextureKind::Texture3D => glow::TEXTURE_3D,
            TextureKind::CubeMap => glow::TEXTURE_CUBE_MAP,
            TextureKind::CubeArray => glow::TEXTURE_CUBE_MAP_ARRAY,
            TextureKind::Texture2DArray => glow::TEXTURE_2D_ARRAY,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TextureFormat {
    R8,
    RG8,
    RGB8,
    RGBA8,
    R16F,
    RG16F,
    RGB16F,
    RGBA16F,
    R32F,
    RG32F,
    RGB32F,
    RGBA32F,
}

impl TextureFormat {
    pub fn get_format(&self) -> u32 {
        match self {
            TextureFormat::R8 => glow::RED,
            TextureFormat::RG8 => glow::RG,
            TextureFormat::RGB8 => glow::RGB,
            TextureFormat::RGBA8 => glow::RGBA,
            TextureFormat::R16F => glow::RED,
            TextureFormat::RG16F => glow::RG,
            TextureFormat::RGB16F => glow::RGB,
            TextureFormat::RGBA16F => glow::RGBA,
            TextureFormat::R32F => glow::RED,
            TextureFormat::RG32F => glow::RG,
            TextureFormat::RGB32F => glow::RGB,
            TextureFormat::RGBA32F => glow::RGBA,
        }
    }

    pub fn get_type(&self) -> u32 {
        match self {
            TextureFormat::R8 => glow::UNSIGNED_BYTE,
            TextureFormat::RG8 => glow::UNSIGNED_BYTE,
            TextureFormat::RGB8 => glow::UNSIGNED_BYTE,
            TextureFormat::RGBA8 => glow::UNSIGNED_BYTE,
            TextureFormat::R16F => glow::HALF_FLOAT,
            TextureFormat::RG16F => glow::HALF_FLOAT,
            TextureFormat::RGB16F => glow::HALF_FLOAT,
            TextureFormat::RGBA16F => glow::HALF_FLOAT,
            TextureFormat::R32F => glow::FLOAT,
            TextureFormat::RG32F => glow::FLOAT,
            TextureFormat::RGB32F => glow::FLOAT,
            TextureFormat::RGBA32F => glow::FLOAT,
        }
    }
}

impl From<TextureFormat> for u32 {
    fn from(format: TextureFormat) -> Self {
        match format {
            TextureFormat::R8 => glow::R8,
            TextureFormat::RG8 => glow::RG8,
            TextureFormat::RGB8 => glow::RGB8,
            TextureFormat::RGBA8 => glow::RGBA8,
            TextureFormat::R16F => glow::R16F,
            TextureFormat::RG16F => glow::RG16F,
            TextureFormat::RGB16F => glow::RGB16F,
            TextureFormat::RGBA16F => glow::RGBA16F,
            TextureFormat::R32F => glow::R32F,
            TextureFormat::RG32F => glow::RG32F,
            TextureFormat::RGB32F => glow::RGB32F,
            TextureFormat::RGBA32F => glow::RGBA32F,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ImageAccess {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

impl From<ImageAccess> for u32 {
    fn from(access: ImageAccess) -> Self {
        match access {
            ImageAccess::ReadOnly => glow::READ_ONLY,
            ImageAccess::WriteOnly => glow::WRITE_ONLY,
            ImageAccess::ReadWrite => glow::READ_WRITE,
        }
    }
}

pub struct Texture {
    gpu: Arc<Gpu>,

    handle: glow::NativeTexture,
    kind: TextureKind,
    format: TextureFormat,
    size: Vec3u,
    mip_levels: u32,
}

pub struct TextureDesc {
    pub kind: TextureKind,
    pub format: TextureFormat,
    pub size: Vec3u,
    pub mip_levels: u32,
}

fn create_texture(gl: &glow::Context, desc: &TextureDesc) {
    unsafe {
        match desc.kind {
            TextureKind::Texture1D => {
                gl.tex_image_1d(
                    desc.kind.into(),
                    0,
                    Into::<u32>::into(desc.format) as i32,
                    desc.size.x() as i32,
                    0,
                    desc.format.get_format(),
                    desc.format.get_type(),
                    glow::PixelUnpackData::Slice(None),
                );
            }
            TextureKind::Texture2D => {
                gl.tex_image_2d(
                    desc.kind.into(),
                    0,
                    Into::<u32>::into(desc.format) as i32,
                    desc.size.x() as i32,
                    desc.size.y() as i32,
                    0,
                    desc.format.get_format(),
                    desc.format.get_type(),
                    glow::PixelUnpackData::Slice(None),
                );
            }
            TextureKind::Texture3D => {
                gl.tex_image_3d(
                    desc.kind.into(),
                    0,
                    Into::<u32>::into(desc.format) as i32,
                    desc.size.x() as i32,
                    desc.size.y() as i32,
                    desc.size.z() as i32,
                    0,
                    desc.format.get_format(),
                    desc.format.get_type(),
                    glow::PixelUnpackData::Slice(None),
                );
            }
            _ => unimplemented!("Texture kind {:?} not implemented yet", desc.kind),
        }
    }
}

impl Texture {
    pub fn new(gpu: Arc<Gpu>, desc: TextureDesc) -> Result<Self, String> {
        unsafe {
            let gl = gpu.context();
            let handle = gl.create_texture().map_err(|e| e.to_string())?;

            gl.bind_texture(desc.kind.into(), Some(handle));
            create_texture(&gl, &desc);
            gl.bind_texture(desc.kind.into(), None);

            Ok(Self {
                gpu,
                handle,
                kind: desc.kind,
                format: desc.format,
                size: desc.size,
                mip_levels: desc.mip_levels,
            })
        }
    }

    pub fn handle(&self) -> glow::NativeTexture {
        self.handle
    }

    pub fn kind(&self) -> TextureKind {
        self.kind
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn size(&self) -> Vec3u {
        self.size
    }

    pub fn mip_levels(&self) -> u32 {
        self.mip_levels
    }

    pub fn resize(&mut self, new_size: Vec3u) {
        unsafe {
            let gl = self.gpu.context();
            gl.bind_texture(self.kind.into(), Some(self.handle));
            create_texture(
                &gl,
                &TextureDesc {
                    kind: self.kind,
                    format: self.format,
                    size: new_size,
                    mip_levels: self.mip_levels,
                },
            );
            gl.bind_texture(self.kind.into(), None);
        }
        self.size = new_size;
    }

    pub fn bind(&self, unit: u32) {
        unsafe {
            let gl = self.gpu.context();
            gl.active_texture(glow::TEXTURE0 + unit);
            gl.bind_texture(self.kind.into(), Some(self.handle));
        }
    }

    pub fn bind_image(&self, binding: u32, access: ImageAccess) {
        unsafe {
            let gl = self.gpu.context();
            gl.bind_image_texture(
                binding,
                Some(self.handle),
                0,
                false,
                0,
                access.into(),
                self.format.into(),
            );
        }
    }

    pub fn unbind(&self, unit: u32) {
        unsafe {
            let gl = self.gpu.context();
            gl.active_texture(glow::TEXTURE0 + unit);
            gl.bind_texture(self.kind.into(), None);
        }
    }

    pub fn unbind_image(&self, binding: u32) {
        unsafe {
            let gl = self.gpu.context();
            gl.bind_image_texture(
                binding,
                None,
                0,
                false,
                0,
                glow::READ_ONLY,
                self.format.into(),
            );
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            let gl = self.gpu.context();
            gl.delete_texture(self.handle);
        }
    }
}
