use std::sync::Arc;

use kmath::{Vec2f, Vec2u};

use crate::{
    acc_buffer::AccBuffer,
    bindings,
    camera::Camera,
    config_buffer::ConfigBuffer,
    profile_buffer::ProfileBuffer,
    scene::{SceneBuffers, build_mock_scene},
};

use gtw::{
    Gpu, MemoryBarrier,
    resources::{Query, QueryTarget, Shader},
};

/// Radical-inverse Halton sequence, base `base`, 1-indexed (`halton(0, _)`
/// is degenerate — start at 1).
fn halton(mut i: u32, base: u32) -> f32 {
    let mut f = 1.0;
    let mut r = 0.0;
    while i > 0 {
        f /= base as f32;
        r += f * (i % base) as f32;
        i /= base;
    }
    r
}

/// Sub-pixel jitter offset in `[-0.5, 0.5]`, cycling through a 16-point
/// Halton(2, 3) sequence keyed off the accumulation frame index.
pub fn halton_jitter(frame_index: u32) -> Vec2f {
    let h = frame_index % 16 + 1;
    Vec2f::new(halton(h, 2) - 0.5, halton(h, 3) - 0.5)
}

pub struct Renderer {
    gpu: Arc<Gpu>,
    resolution: Vec2f,
    shader: Shader,

    acc_buffer: AccBuffer,
    profile_buffer: ProfileBuffer,
    config_buffer: ConfigBuffer,

    scene_buffers: SceneBuffers,

    frame_index: u32,
    delta_time: f32,
    last_frame_time: std::time::Instant,
    fps: f32,
    time_acc: f32,
    frame_count: u32,

    elapsed_time_query: Query,
}

impl Renderer {
    pub fn new(gpu: Arc<Gpu>, resolution: Vec2f) -> Result<Self, String> {
        let shader = unsafe {
            Shader::from_file(gpu.clone(), "assets/shaders/tier0.comp")
                .expect("Failed loading raytracer")
        };

        Ok(Self {
            gpu: gpu.clone(),
            resolution,
            shader,

            acc_buffer: AccBuffer::new(gpu.clone(), resolution.convert_to::<u32>()).unwrap(),
            profile_buffer: ProfileBuffer::new(gpu.clone(), bindings::PROFILE)?,
            config_buffer: ConfigBuffer::new(gpu.clone(), bindings::CONFIG)?,

            scene_buffers: SceneBuffers::upload(gpu.clone(), &build_mock_scene())?,

            frame_index: 0,
            delta_time: 0.0,
            last_frame_time: std::time::Instant::now(),
            fps: 0.0,
            time_acc: 0.0,
            frame_count: 0,

            elapsed_time_query: Query::new(gpu.clone(), QueryTarget::TimeElapsed)?,
        })
    }

    pub fn resize(&mut self, new_resolution: Vec2f) -> Result<(), String> {
        self.resolution = new_resolution;
        self.acc_buffer = AccBuffer::new(self.gpu.clone(), new_resolution.convert_to::<u32>())?;

        Ok(())
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn config_buffer(&self) -> &ConfigBuffer {
        &self.config_buffer
    }
    pub fn config_buffer_mut(&mut self) -> &mut ConfigBuffer {
        &mut self.config_buffer
    }

    pub fn render(&mut self, camera: &mut Camera) {
        tracy_client::frame_mark();
        let _span = tracy_client::span!("Renderer::render");

        let now = std::time::Instant::now();
        self.delta_time = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        self.time_acc += self.delta_time;
        self.frame_count += 1;

        if self.time_acc >= 0.5 {
            // update the displayed number twice a second
            self.fps = self.frame_count as f32 / self.time_acc;
            self.time_acc = 0.0;
            self.frame_count = 0;
        }

        if camera.dirty || self.config_buffer.dirty {
            self.frame_index = 0;
            camera.dirty = false;
        }
        let jitter = halton_jitter(self.frame_index);

        self.elapsed_time_query.begin();

        let _span = tracy_client::span!("Render Pass");

        self.acc_buffer.bind_write_tex();
        self.gpu
            .viewport(Vec2u::new(0, 0), self.resolution.convert_to::<u32>());

        self.shader.use_program();

        {
            let _span = tracy_client::span!("Buffer Binding");

            self.acc_buffer.bind_read_tex(0);

            self.shader.set_uniform_i32("historyTex", 0);
            self.shader.set_uniform_u32("frameIndex", self.frame_index);

            self.shader.set_uniform_vec2f("jitter", jitter);
            self.shader
                .set_uniform_u32("volumeCount", self.scene_buffers.volumes_count());

            let inv_view = camera.get_view_matrix().inverse().unwrap();
            let inv_proj = camera.get_proj_matrix().inverse().unwrap();
            self.shader.set_uniform_mat4f("invView", inv_view);
            self.shader.set_uniform_mat4f("invProj", inv_proj);
            self.shader
                .set_uniform_vec3f("cameraPos", camera.transform().position);
            self.shader.set_uniform_vec2f("resolution", self.resolution);

            self.profile_buffer.begin_frame();
            self.config_buffer.upload_if_dirty();
        }

        self.gpu.dispatch_compute(
            (self.resolution.x() as u32 + 15) / 16,
            (self.resolution.y() as u32 + 15) / 16,
            1,
        );
        self.gpu.memory_barrier(
            MemoryBarrier::SHADER_IMAGE_ACCESS_BARRIER | MemoryBarrier::FRAMEBUFFER_BARRIER,
        );

        self.profile_buffer.end_frame().expect("Failed ending profile frame");

        let _span = tracy_client::span!("Blit");
        self.acc_buffer.blit();

        self.elapsed_time_query.end();

        self.frame_index += 1;
        self.acc_buffer.swap();

        self.profile_buffer.poll_and_report();

        if let Some(ns) = self.elapsed_time_query.get_u64() {
            let ms = ns as f64 / 1_000_000.0;
            tracy_client::plot!("GPU Time (ms)", ms);
        }
    }
}
