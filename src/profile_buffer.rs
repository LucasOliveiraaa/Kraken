#![allow(unsafe_op_in_unsafe_fn)]

use gl::types::GLsync;

/// Counter indices, mirrored in `bicubic.frag`'s `Profile` SSBO.
pub mod counters {
    pub const NEWTON_ITERATIONS: usize = 0;
    pub const PLANE_TESTS: usize = 1;
    pub const QUAD_NODE_VISITS: usize = 2;
    pub const RAY_BOUNCES: usize = 3;
    pub const LIGHT_HITS: usize = 4;
    pub const COUNT: usize = 5;
}

const BYTES: isize = (counters::COUNT * std::mem::size_of::<u32>()) as isize;

pub struct ProfileBuffer {
    ssbo: [u32; 2],
    fence: [Option<GLsync>; 2],
    write_idx: usize,
    binding: u32,
}

impl ProfileBuffer {
    pub unsafe fn new(binding: u32) -> Self {
        let mut ssbo = [0u32; 2];
        gl::GenBuffers(2, ssbo.as_mut_ptr());
        for &buf in &ssbo {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buf);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                BYTES,
                std::ptr::null(),
                gl::DYNAMIC_COPY, // GPU writes (atomics), CPU reads back
            );
        }

        Self {
            ssbo,
            fence: [None, None],
            write_idx: 0,
            binding,
        }
    }

    /// Zeroes this frame's counters and binds the buffer at its SSBO
    /// binding point, ready for the shader to atomically add into. Call
    /// once per frame, before the draw call.
    pub unsafe fn begin_frame(&mut self) {
        let idx = self.write_idx;
        let zeros = [0u32; counters::COUNT];
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.ssbo[idx]);
        gl::BufferSubData(gl::SHADER_STORAGE_BUFFER, 0, BYTES, zeros.as_ptr() as *const _);
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, self.binding, self.ssbo[idx]);
    }

    /// Fences the buffer just written to and swaps which buffer is "next
    /// to write". Call once per frame, right after the draw call.
    pub unsafe fn end_frame(&mut self) {
        let idx = self.write_idx;
        if let Some(f) = self.fence[idx].take() {
            gl::DeleteSync(f);
        }
        self.fence[idx] = Some(gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0));
        self.write_idx = 1 - idx;
    }

    /// Non-blocking: checks both buffers' fences, and for any that have
    /// signaled, reads the counters back and plots them in Tracy. Call
    /// once per frame, anywhere after `end_frame`.
    pub unsafe fn poll_and_report(&mut self) {
        for idx in 0..2 {
            let Some(f) = self.fence[idx] else { continue };

            let status = gl::ClientWaitSync(f, 0, 0);
            let signaled = status == gl::ALREADY_SIGNALED || status == gl::CONDITION_SATISFIED;
            if !signaled {
                continue;
            }

            let mut data = [0u32; counters::COUNT];
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.ssbo[idx]);
            gl::GetBufferSubData(gl::SHADER_STORAGE_BUFFER, 0, BYTES, data.as_mut_ptr() as *mut _);

            tracy_client::plot!("Newton Iterations", data[counters::NEWTON_ITERATIONS] as f64);
            tracy_client::plot!("Plane Tests", data[counters::PLANE_TESTS] as f64);
            tracy_client::plot!("Quad Node Visits", data[counters::QUAD_NODE_VISITS] as f64);
            tracy_client::plot!("Ray Bounces", data[counters::RAY_BOUNCES] as f64);
            tracy_client::plot!("Light Hits", data[counters::LIGHT_HITS] as f64);

            gl::DeleteSync(f);
            self.fence[idx] = None;
        }
    }

    pub unsafe fn destroy(&mut self) {
        for f in self.fence.iter_mut().flatten() {
            gl::DeleteSync(*f);
        }
        gl::DeleteBuffers(2, self.ssbo.as_ptr());
    }
}