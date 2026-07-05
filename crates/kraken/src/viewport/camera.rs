use kmath::{Transform, Vec2f, Vec3f};

pub struct Camera {
    camera: krender::camera::Camera,

    sensitivity: f32,
    speed: f32,
}

impl Camera {
    pub fn new(transform: Transform, fov: f32, near_plane: f32, far_plane: f32) -> Self {
        let camera = krender::camera::Camera::new(transform, fov, near_plane, far_plane);

        Self {
            camera,
            sensitivity: 0.05,
            speed: 1.0,
        }
    }

    pub fn transform(&self) -> Transform {
        self.camera.transform()
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.camera.set_transform(transform);
    }

    pub fn sensitivity(&self) -> f32 {
        self.sensitivity
    }

    pub fn sensitivity_mut(&mut self) -> &mut f32 {
        &mut self.sensitivity
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn speed_mut(&mut self) -> &mut f32 {
        &mut self.speed
    }

    pub fn raw_camera_mut(&mut self) -> &mut krender::camera::Camera {
        &mut self.camera
    }

    pub fn move_towards(&mut self, direction: Vec3f) {
        let dir = self.camera.get_direction_vector(direction);

        let mut transform = self.camera.transform();
        transform.position += dir * self.speed;
        self.camera.set_transform(transform);
    }

    pub fn rotate(&mut self, delta_rotation: Vec2f) {
        let mut transform = self.camera.transform();
        transform.rotation += delta_rotation.extend(0.0).normalize() * self.sensitivity;
        self.camera.set_transform(transform);
    }
}
