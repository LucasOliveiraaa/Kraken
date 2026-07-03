#![allow(unsafe_op_in_unsafe_fn)]

use std::sync::Arc;

use gl::types::GLsync;
use glow::{Context, HasContext};

/// Counter indices, mirrored in the shader.
pub mod counters {
    pub const NEWTON_ITERATIONS: usize = 0;
    pub const PLANE_TESTS: usize = 1;
    pub const QUAD_NODE_VISITS: usize = 2;
    pub const RAY_BOUNCES: usize = 3;
    pub const LIGHT_HITS: usize = 4;
    pub const COUNT: usize = 5;
}

const BYTES: i32 = (counters::COUNT * std::mem::size_of::<u32>()) as i32;

pub struct ProfileBuffer {
    gl: Arc<Context>,

    ssbo: [glow::NativeBuffer; 2],
    fence: [Option<GLsync>; 2],

    write_idx: usize,
    binding: u32,
}

impl ProfileBuffer {
    pub unsafe fn new(
        gl: Arc<Context>,
        binding: u32,
    ) -> Result<Self, String> {
        let ssbo = [
            gl.create_buffer()?,
            gl.create_buffer()?,
        ];

        for &buffer in &ssbo {
            gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(buffer));

            gl.buffer_data_size(
                glow::SHADER_STORAGE_BUFFER,
                BYTES,
                glow::DYNAMIC_COPY,
            );
        }

        gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, None);

        Ok(Self {
            gl,
            ssbo,
            fence: [None, None],
            write_idx: 0,
            binding,
        })
    }

    pub unsafe fn begin_frame(&mut self) {
        let idx = self.write_idx;

        let zeros = [0u32; counters::COUNT];

        self.gl.bind_buffer(
            glow::SHADER_STORAGE_BUFFER,
            Some(self.ssbo[idx]),
        );

        self.gl.buffer_sub_data_u8_slice(
            glow::SHADER_STORAGE_BUFFER,
            0,
            bytemuck::cast_slice(&zeros),
        );

        self.gl.bind_buffer_base(
            glow::SHADER_STORAGE_BUFFER,
            self.binding,
            Some(self.ssbo[idx]),
        );
    }

    pub unsafe fn end_frame(&mut self) {
        let idx = self.write_idx;

        if let Some(sync) = self.fence[idx].take() {
            gl::DeleteSync(sync);
        }

        self.fence[idx] =
            Some(gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0));

        self.write_idx ^= 1;
    }

    pub unsafe fn poll_and_report(&mut self) {
        for idx in 0..2 {
            let Some(sync) = self.fence[idx] else {
                continue;
            };

            let status = gl::ClientWaitSync(sync, 0, 0);

            if status != gl::ALREADY_SIGNALED
                && status != gl::CONDITION_SATISFIED
            {
                continue;
            }

            let mut data = [0u32; counters::COUNT];

            self.gl.bind_buffer(
                glow::SHADER_STORAGE_BUFFER,
                Some(self.ssbo[idx]),
            );

            self.gl.get_buffer_sub_data(
                glow::SHADER_STORAGE_BUFFER,
                0,
                bytemuck::cast_slice_mut(&mut data),
            );

            tracy_client::plot!(
                "Newton Iterations",
                data[counters::NEWTON_ITERATIONS] as f64
            );

            tracy_client::plot!(
                "Plane Tests",
                data[counters::PLANE_TESTS] as f64
            );

            tracy_client::plot!(
                "Quad Node Visits",
                data[counters::QUAD_NODE_VISITS] as f64
            );

            tracy_client::plot!(
                "Ray Bounces",
                data[counters::RAY_BOUNCES] as f64
            );

            tracy_client::plot!(
                "Light Hits",
                data[counters::LIGHT_HITS] as f64
            );

            gl::DeleteSync(sync);
            self.fence[idx] = None;
        }
    }
}

impl Drop for ProfileBuffer {
    fn drop(&mut self) {
        unsafe {
            for sync in self.fence.iter().flatten() {
                gl::DeleteSync(*sync);
            }

            for buffer in self.ssbo {
                self.gl.delete_buffer(buffer);
            }
        }
    }
}