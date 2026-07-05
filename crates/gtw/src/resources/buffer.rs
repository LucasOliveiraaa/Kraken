use std::sync::Arc;

use glow::HasContext;

use crate::Gpu;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BufferTarget {
    ArrayBuffer,
    ElementArrayBuffer,
    UniformBuffer,
    ShaderStorageBuffer,
}

impl From<BufferTarget> for u32 {
    fn from(target: BufferTarget) -> Self {
        match target {
            BufferTarget::ArrayBuffer => glow::ARRAY_BUFFER,
            BufferTarget::ElementArrayBuffer => glow::ELEMENT_ARRAY_BUFFER,
            BufferTarget::UniformBuffer => glow::UNIFORM_BUFFER,
            BufferTarget::ShaderStorageBuffer => glow::SHADER_STORAGE_BUFFER,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BufferUsage {
    StaticDraw,
    DynamicDraw,
    StreamDraw,
    DynamicCopy,
}

impl From<BufferUsage> for u32 {
    fn from(usage: BufferUsage) -> Self {
        match usage {
            BufferUsage::StaticDraw => glow::STATIC_DRAW,
            BufferUsage::DynamicDraw => glow::DYNAMIC_DRAW,
            BufferUsage::StreamDraw => glow::STREAM_DRAW,
            BufferUsage::DynamicCopy => glow::DYNAMIC_COPY,
        }
    }
}

pub struct Buffer {
    gpu: Arc<Gpu>,

    handle: glow::NativeBuffer,
    size: usize,
    target: BufferTarget,
    usage: BufferUsage,
}

pub struct BufferDesc {
    pub size: usize,
    pub target: BufferTarget,
    pub usage: BufferUsage,
}

impl Buffer {
    pub fn new(gpu: Arc<Gpu>, desc: BufferDesc) -> Result<Self, String> {
        unsafe {
            let gl = gpu.context();
            let handle = gl.create_buffer()?;

            gl.bind_buffer(desc.target.into(), Some(handle));
            gl.buffer_data_size(desc.target.into(), desc.size as i32, desc.usage.into());
            gl.bind_buffer(desc.target.into(), None);

            Ok(Self {
                gpu,
                handle,
                size: desc.size,
                target: desc.target,
                usage: desc.usage,
            })
        }
    }

    pub fn new_with_data<T: bytemuck::Pod>(
        gpu: Arc<Gpu>,
        data: &[T],
        desc: BufferDesc,
    ) -> Result<Self, String> {
        let size = std::mem::size_of_val(data);
        unsafe {
            let gl = gpu.context();
            let handle = gl.create_buffer()?;

            gl.bind_buffer(desc.target.into(), Some(handle));
            gl.buffer_data_u8_slice(
                desc.target.into(),
                bytemuck::cast_slice(data),
                desc.usage.into(),
            );
            gl.bind_buffer(desc.target.into(), None);

            Ok(Self {
                gpu,
                handle,
                size,
                target: desc.target,
                usage: desc.usage,
            })
        }
    }

    pub fn handle(&self) -> glow::NativeBuffer {
        self.handle
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn target(&self) -> BufferTarget {
        self.target
    }

    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    pub fn bind_base(&self, index: u32) {
        if !matches!(
            self.target,
            BufferTarget::UniformBuffer | BufferTarget::ShaderStorageBuffer
        ) {
            panic!("bind_base can only be used with UniformBuffer or ShaderStorageBuffer targets");
        }

        unsafe {
            let gl = self.gpu.context();
            gl.bind_buffer(self.target.into(), Some(self.handle));
            gl.bind_buffer_base(self.target.into(), index, Some(self.handle));
            gl.bind_buffer(self.target.into(), None);
        }
    }

    pub fn copy_from(&self, src: &Buffer, read_offset: usize, write_offset: usize, size: usize) {
        if self.target != src.target {
            panic!("copy_from can only be used with buffers of the same target");
        }

        if read_offset + size > src.size || write_offset + size > self.size {
            panic!("copy_from out of bounds");
        }

        unsafe {
            let gl = self.gpu.context();
            gl.bind_buffer(glow::COPY_READ_BUFFER, Some(src.handle));
            gl.bind_buffer(glow::COPY_WRITE_BUFFER, Some(self.handle));
            gl.copy_buffer_sub_data(
                glow::COPY_READ_BUFFER,
                glow::COPY_WRITE_BUFFER,
                read_offset as i32,
                write_offset as i32,
                size as i32,
            );
            gl.bind_buffer(glow::COPY_READ_BUFFER, None);
            gl.bind_buffer(glow::COPY_WRITE_BUFFER, None);
        }
    }

    pub fn upload_data<T: bytemuck::Pod>(&self, offset: usize, data: &[T]) {
        let bytes = std::mem::size_of_val(data);
        assert!(offset + bytes <= self.size);

        unsafe {
            let gl = self.gpu.context();
            gl.bind_buffer(self.target.into(), Some(self.handle));
            gl.buffer_sub_data_u8_slice(
                self.target.into(),
                offset as i32,
                bytemuck::cast_slice(data),
            );
            gl.bind_buffer(self.target.into(), None);
        }
    }

    pub fn download_data<T: bytemuck::Pod>(&self, offset: usize, data: &mut [T]) {
        let bytes = std::mem::size_of_val(data);
        assert!(offset + bytes <= self.size);

        unsafe {
            let gl = self.gpu.context();
            gl.bind_buffer(self.target.into(), Some(self.handle));
            gl.get_buffer_sub_data(
                self.target.into(),
                offset as i32,
                bytemuck::cast_slice_mut(data),
            );
            gl.bind_buffer(self.target.into(), None);
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            let gl = self.gpu.context();
            gl.delete_buffer(self.handle);
        }
    }
}
