#![allow(unsafe_op_in_unsafe_fn)]

use gl::types::GLenum;
use std::ffi::CString;
use std::mem;

/// Compiles a single shader stage from source, panicking with the driver's
/// info log if compilation fails.
pub fn compile_shader(src: &str, kind: GLenum) -> u32 {
    unsafe {
        let shader = gl::CreateShader(kind);
        let c_src = CString::new(src).unwrap();
        gl::ShaderSource(shader, 1, &c_src.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut ok = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut ok);
        if ok == 0 {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0u8; len as usize];
            gl::GetShaderInfoLog(
                shader,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut i8,
            );
            panic!("Shader compile error: {}", String::from_utf8_lossy(&buf));
        }
        shader
    }
}

/// Links a vertex + fragment shader pair into a program, panicking with the
/// driver's info log if linking fails. Does not delete the input shaders.
pub fn link_program(vert: u32, frag: u32) -> u32 {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vert);
        gl::AttachShader(program, frag);
        gl::LinkProgram(program);

        let mut ok = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut ok);
        if ok == 0 {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0u8; len as usize];
            gl::GetProgramInfoLog(
                program,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut i8,
            );
            panic!("Program link error: {}", String::from_utf8_lossy(&buf));
        }
        program
    }
}

/// Looks up a uniform location by name. Cheap enough to call once a frame
/// for a handful of uniforms; if this ever shows up in a profile, switch to
/// caching the locations once after linking.
pub unsafe fn uniform_loc(program: u32, name: &str) -> i32 {
    let c = CString::new(name).unwrap();
    gl::GetUniformLocation(program, c.as_ptr())
}

/// Uploads `data` as a `GL_STATIC_DRAW` SSBO and binds it at `binding`.
/// Used for scene data that's set up once and never touched again.
pub unsafe fn upload_ssbo<T>(data: &[T], binding: u32) -> u32 {
    let mut ssbo = 0u32;
    gl::GenBuffers(1, &mut ssbo);
    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
    gl::BufferData(
        gl::SHADER_STORAGE_BUFFER,
        (data.len() * mem::size_of::<T>()) as isize,
        data.as_ptr() as *const _,
        gl::STATIC_DRAW,
    );
    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, ssbo);
    ssbo
}

/// Creates a zero-initialized SSBO of `size_bytes` and binds it at
/// `binding`. Used for buffers the CPU or GPU will write into later
/// (profiling counters, config data) rather than upload-once scene data.
pub unsafe fn create_empty_ssbo(size_bytes: isize, binding: u32, usage: GLenum) -> u32 {
    let mut ssbo = 0u32;
    gl::GenBuffers(1, &mut ssbo);
    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
    gl::BufferData(gl::SHADER_STORAGE_BUFFER, size_bytes, std::ptr::null(), usage);
    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, ssbo);
    ssbo
}