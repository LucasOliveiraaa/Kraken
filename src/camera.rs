use glfw::{Action, Key, Window};

use crate::kmath::{Mat4, Vec2, Vec3};

pub struct Camera {
    pub pos: Vec3,
    pub yaw: f32,   // rotation around Y (left/right)
    pub pitch: f32, // rotation around X (up/down)
}

impl Camera {
    pub fn new(pos: Vec3) -> Self {
        Self {
            pos,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    /// Returns (forward, right, up), all normalized.
    pub fn basis(&self) -> (Vec3, Vec3, Vec3) {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();

        let forward = Vec3::new(-sy * cp, sp, -cy * cp).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward).normalize();

        (forward, right, up)
    }

    /// Applies WASDQE movement and arrow-key look based on currently held
    /// keys. Returns `true` if the camera moved or rotated this frame,
    /// which callers use to decide whether to reset temporal accumulation.
    pub fn handle_input(&mut self, window: &Window, dt: f32) -> bool {
        const MOVE_SPEED: f32 = 4.0; // units/sec
        const LOOK_SPEED: f32 = 0.25; // rad/sec

        let (forward, right, up) = self.basis();
        let mut changed = false;

        let mut apply_move = |delta: Vec3| {
            self.pos += delta;
            changed = true;
        };

        if window.get_key(Key::W) == Action::Press {
            apply_move(forward * MOVE_SPEED * dt);
        }
        if window.get_key(Key::S) == Action::Press {
            apply_move(-forward * MOVE_SPEED * dt);
        }
        if window.get_key(Key::A) == Action::Press {
            apply_move(-right * MOVE_SPEED * dt);
        }
        if window.get_key(Key::D) == Action::Press {
            apply_move(right * MOVE_SPEED * dt);
        }
        if window.get_key(Key::Q) == Action::Press {
            apply_move(-up * MOVE_SPEED * dt);
        }
        if window.get_key(Key::E) == Action::Press {
            apply_move(up * MOVE_SPEED * dt);
        }

        if window.get_key(Key::Left) == Action::Press {
            self.yaw += LOOK_SPEED * dt;
            changed = true;
        }
        if window.get_key(Key::Right) == Action::Press {
            self.yaw -= LOOK_SPEED * dt;
            changed = true;
        }
        if window.get_key(Key::Up) == Action::Press {
            self.pitch += LOOK_SPEED * dt;
            changed = true;
        }
        if window.get_key(Key::Down) == Action::Press {
            self.pitch -= LOOK_SPEED * dt;
            changed = true;
        }

        // Prevent gimbal flip.
        self.pitch = self.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );

        changed
    }

    /// Builds the inverse-view matrix expected by the shader (camera basis
    /// as columns, translation in the last column).
    pub fn inv_view(&self) -> Mat4 {
        let (forward, right, up) = self.basis();
        Mat4::from_cols(
            right.extend(0.0),
            up.extend(0.0),
            (-forward).extend(0.0),
            self.pos.extend(1.0),
        )
    }
}

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
pub fn halton_jitter(frame_index: u32) -> Vec2 {
    let h = frame_index % 16 + 1;
    Vec2::new(halton(h, 2) - 0.5, halton(h, 3) - 0.5)
}