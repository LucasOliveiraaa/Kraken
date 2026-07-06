mod camera;

pub use camera::*;

use std::{collections::HashSet, sync::Arc};

use gtw::Gpu;
use kmath::{Transform, Vec2f, Vec3f};
use krender::{Renderer, config_buffer::ConfigBuffer};
use winit::keyboard::KeyCode;

use crate::editor::MouseState;

pub struct Viewport {
    renderer: Renderer,
    resolution: Vec2f,

    camera: Camera,
}

impl Viewport {
    pub fn new(gpu: Arc<Gpu>, resolution: Vec2f) -> Result<Self, String> {
        let renderer = Renderer::new(gpu, resolution)?;

        Ok(Self {
            renderer,
            resolution,

            camera: Camera::new(
                Transform::new(
                    Vec3f::new(0.0, 0.0, 0.0),
                    Vec3f::new(0.0, 0.0, 0.0),
                    resolution.extend(1.0),
                ),
                90f32.to_radians(), // FOV
                0.1,                // Near plane
                100.0,              // Far plane
            ),
        })
    }

    pub fn switch_tier(&mut self, tier: krender::RenderTier) -> Result<(), String> {
        self.renderer.switch_tier(tier)
    }

    pub fn config_buffer(&self) -> &ConfigBuffer {
        self.renderer.config_buffer()
    }
    pub fn config_buffer_mut(&mut self) -> &mut ConfigBuffer {
        self.renderer.config_buffer_mut()
    }

    pub fn resize(&mut self, resolution: Vec2f) -> Result<(), String> {
        self.resolution = resolution;
        self.camera.set_transform(Transform::new(
            self.camera.transform().position,
            self.camera.transform().rotation,
            resolution.extend(1.0),
        ));

        self.renderer.resize(resolution)
    }

    pub fn fps(&self) -> f32 {
        self.renderer.fps()
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn update(&mut self, pressed_keys: &HashSet<KeyCode>, _mouse_state: &MouseState) {
        let dt = self.renderer.delta_time();

        if pressed_keys.contains(&KeyCode::KeyW) {
            self.camera.move_towards(Vec3f::new(0.0, 0.0, -1.0) * dt);
        }
        if pressed_keys.contains(&KeyCode::KeyS) {
            self.camera.move_towards(Vec3f::new(0.0, 0.0, 1.0) * dt);
        }
        if pressed_keys.contains(&KeyCode::KeyA) {
            self.camera.move_towards(Vec3f::new(1.0, 0.0, 0.0) * dt);
        }
        if pressed_keys.contains(&KeyCode::KeyD) {
            self.camera.move_towards(Vec3f::new(-1.0, 0.0, 0.0) * dt);
        }

        if pressed_keys.contains(&KeyCode::KeyQ) {
            self.camera.move_towards(Vec3f::new(0.0, -1.0, 0.0) * dt);
        }
        if pressed_keys.contains(&KeyCode::KeyE) {
            self.camera.move_towards(Vec3f::new(0.0, 1.0, 0.0) * dt);
        }

        if pressed_keys.contains(&KeyCode::ArrowUp) {
            self.camera.rotate(Vec2f::new(1.0, 0.0) * dt); // pitch
        }
        if pressed_keys.contains(&KeyCode::ArrowDown) {
            self.camera.rotate(Vec2f::new(-1.0, 0.0) * dt); // pitch
        }
        if pressed_keys.contains(&KeyCode::ArrowLeft) {
            self.camera.rotate(Vec2f::new(0.0, 1.0) * dt); // yaw
        }
        if pressed_keys.contains(&KeyCode::ArrowRight) {
            self.camera.rotate(Vec2f::new(0.0, -1.0) * dt); // yaw
        }

        self.renderer.render(self.camera.raw_camera_mut());
    }
}
