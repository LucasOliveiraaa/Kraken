use std::sync::Arc;

use kmath::{Vec2f, Vec2i, Vec3f};

use crate::{acc_buffer::AccBuffer, camera::Camera};

pub struct Renderer {
    gl: Arc<glow::Context>,
    resolution: Vec2f,
    acc_buffer: AccBuffer,
    camera: Camera,
}

impl Renderer {
    pub fn new(gl: Arc<glow::Context>, resolution: Vec2f) -> Self {
        Self {
            gl: gl.clone(),
            resolution,
            acc_buffer: AccBuffer::new(
                gl.clone(),
                Vec2i::new(resolution.x() as i32, resolution.y() as i32),
            )
            .unwrap(),
            camera: Camera::new(
                Vec3f::new(0.0, 0.0, 0.0),
                Vec3f::new(0.0, 0.0, 0.0),
                resolution,
                90.0,
                0.1,
                100.0,
            ),
        }
    }

    pub fn resize(&mut self, new_resolution: Vec2f) -> Result<(), String> {
        self.resolution = new_resolution;
        self.acc_buffer.resize(Vec2i::new(
            new_resolution.x() as i32,
            new_resolution.y() as i32,
        ))?;
        self.camera.update_size(new_resolution);

        Ok(())
    }
}
