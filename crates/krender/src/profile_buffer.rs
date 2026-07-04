use std::sync::Arc;

use gtw::{
    Gpu,
    resources::{Buffer, BufferDesc, BufferTarget, BufferUsage},
    sync::Fence,
};

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
    gpu: Arc<Gpu>,

    ssbo: [Buffer; 2],
    fence: [Option<Fence>; 2],

    write_idx: usize,
    binding: u32,
}

impl ProfileBuffer {
    pub fn new(gpu: Arc<Gpu>, binding: u32) -> Result<Self, String> {
        let ssbo1 = Buffer::new(
            gpu.clone(),
            BufferDesc {
                size: BYTES as usize,
                target: BufferTarget::ShaderStorageBuffer,
                usage: BufferUsage::DynamicCopy,
            },
        )?;
        let ssbo2 = Buffer::new(
            gpu.clone(),
            BufferDesc {
                size: BYTES as usize,
                target: BufferTarget::ShaderStorageBuffer,
                usage: BufferUsage::DynamicCopy,
            },
        )?;

        Ok(Self {
            gpu,
            ssbo: [ssbo1, ssbo2],
            fence: [None, None],
            write_idx: 0,
            binding,
        })
    }

    pub fn begin_frame(&mut self) {
        let idx = self.write_idx;

        let zeros = [0u32; counters::COUNT];

        self.ssbo[idx].upload_data(0, &zeros);
        self.ssbo[idx].bind_base(self.binding);
    }

    pub fn end_frame(&mut self) -> Result<(), String> {
        let idx = self.write_idx;

        if let Some(sync) = self.fence[idx].take() {
            drop(sync);
        }

        self.fence[idx] = Some(Fence::new(self.gpu.clone())?);
        self.write_idx ^= 1;

        Ok(())
    }

    pub fn poll_and_report(&mut self) {
        for idx in 0..2 {
            let Some(sync) = self.fence[idx].take() else {
                continue;
            };

            if sync.wait_client(false, Some(0)).is_err() {
                continue;
            }

            let mut data = [0u32; counters::COUNT];
            self.ssbo[idx].download_data(0, &mut data);

            tracy_client::plot!(
                "Newton Iterations",
                data[counters::NEWTON_ITERATIONS] as f64
            );
            tracy_client::plot!("Plane Tests", data[counters::PLANE_TESTS] as f64);
            tracy_client::plot!("Quad Node Visits", data[counters::QUAD_NODE_VISITS] as f64);
            tracy_client::plot!("Ray Bounces", data[counters::RAY_BOUNCES] as f64);
            tracy_client::plot!("Light Hits", data[counters::LIGHT_HITS] as f64);
        }
    }
}
