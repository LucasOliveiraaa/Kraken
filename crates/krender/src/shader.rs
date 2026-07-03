use std::sync::Arc;

use glow::HasContext;
use kmath::{Mat4, Vec2f, Vec3f, Vec4f};

pub struct Shader {
    gl: Arc<glow::Context>,
    pub program: glow::Program,
}

unsafe fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    source: &str,
) -> Result<glow::NativeShader, String> {
    unsafe {
        let shader = gl.create_shader(shader_type).map_err(|e| e.to_string())?;

        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            return Err(gl.get_shader_info_log(shader));
        }

        Ok(shader)
    }
}

impl Shader {
    pub unsafe fn from_file(gl: Arc<glow::Context>, path: &str) -> Result<Self, String> {
        let source = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        unsafe { Self::new(gl, &source) }
    }

    pub unsafe fn new(gl: Arc<glow::Context>, source: &str) -> Result<Self, String> {
        unsafe {
            let program = gl.create_program().map_err(|e| e.to_string())?;

            let shader = compile_shader(&gl, glow::COMPUTE_SHADER, source)?;

            gl.attach_shader(program, shader);
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                return Err(format!("Failed to link shader program: {}", log));
            }

            gl.delete_shader(shader);

            Ok(Self { gl, program })
        }
    }

    pub fn use_program(&self) {
        unsafe {
            self.gl.use_program(Some(self.program));
        }
    }

    pub fn set_uniform_f32(&self, name: &str, value: f32) {
        unsafe {
            let loc = self.gl.get_uniform_location(self.program, name);
            if let Some(loc) = loc {
                self.gl.uniform_1_f32(Some(&loc), value);
            }
        }
    }

    pub fn set_uniform_vec2f(&self, name: &str, value: Vec2f) {
        unsafe {
            let loc = self.gl.get_uniform_location(self.program, name);
            if let Some(loc) = loc {
                self.gl.uniform_2_f32(Some(&loc), value.x(), value.y());
            }
        }
    }

    pub fn set_uniform_vec3f(&self, name: &str, value: Vec3f) {
        unsafe {
            let loc = self.gl.get_uniform_location(self.program, name);
            if let Some(loc) = loc {
                self.gl.uniform_3_f32(Some(&loc), value.x(), value.y(), value.z());
            }
        }
    }

    pub fn set_uniform_vec4f(&self, name: &str, value: Vec4f) {
        unsafe {
            let loc = self.gl.get_uniform_location(self.program, name);
            if let Some(loc) = loc {
                self.gl.uniform_4_f32(Some(&loc), value.x(), value.y(), value.z(), value.w());
            }
        }
    }

    pub fn set_uniform_i32(&self, name: &str, value: i32) {
        unsafe {
            let loc = self.gl.get_uniform_location(self.program, name);
            if let Some(loc) = loc {
                self.gl.uniform_1_i32(Some(&loc), value);
            }
        }
    }

    pub fn set_uniform_u32(&self, name: &str, value: u32) {
        unsafe {
            let loc = self.gl.get_uniform_location(self.program, name);
            if let Some(loc) = loc {
                self.gl.uniform_1_u32(Some(&loc), value);
            }
        }
    }

    pub fn set_uniform_mat4f(&self, name: &str, mat: Mat4<f32>) {
        unsafe {
            let loc = self.gl.get_uniform_location(self.program, name);
            if let Some(loc) = loc {
                self.gl.uniform_matrix_4_f32_slice(Some(&loc), false, mat.as_flat_data());
            }
        }
    }
}
