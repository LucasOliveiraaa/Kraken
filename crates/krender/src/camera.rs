use kmath::{Mat4f, Transform, Vec3f};

pub struct Camera {
    transform: Transform,

    fov: f32,
    near_plane: f32,
    far_plane: f32,

    pub dirty: bool,
}

impl Camera {
    pub fn new(transform: Transform, fov: f32, near_plane: f32, far_plane: f32) -> Self {
        Self {
            transform,
            fov,
            near_plane,
            far_plane,
            dirty: true,
        }
    }

    pub fn set_transform(&mut self, new_transform: kmath::Transform) {
        self.transform = new_transform;
        self.dirty = true;
    }

    pub fn set_fov(&mut self, new_fov: f32) {
        self.fov = new_fov;
        self.dirty = true;
    }

    pub fn set_near_plane(&mut self, new_near_plane: f32) {
        self.near_plane = new_near_plane;
        self.dirty = true;
    }

    pub fn set_far_plane(&mut self, new_far_plane: f32) {
        self.far_plane = new_far_plane;
        self.dirty = true;
    }

    pub fn transform(&self) -> Transform {
        self.transform
    }

    pub fn fov(&self) -> f32 {
        self.fov
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.transform.scale.x() / self.transform.scale.y()
    }

    pub fn near_plane(&self) -> f32 {
        self.near_plane
    }

    pub fn far_plane(&self) -> f32 {
        self.far_plane
    }

    pub fn get_view_matrix(&self) -> Mat4f {
        let mut transform = self.transform;
        transform.scale = Vec3f::new(1.0, 1.0, 1.0);
        transform.to_matrix()
            .inverse()
            .unwrap()
    }

    pub fn get_proj_matrix(&self) -> Mat4f {
        Mat4f::from_perspective(self.fov, self.aspect_ratio(), self.near_plane, self.far_plane)
    }

    pub fn get_forward(&self) -> Vec3f {
        Vec3f::new(
            self.transform.rotation.y().sin() * self.transform.rotation.x().cos(),
            -self.transform.rotation.x().sin(),
            self.transform.rotation.y().cos() * self.transform.rotation.x().cos(),
        )
        .normalize()
    }

    pub fn get_direction_vector(&self, direction: Vec3f) -> Vec3f {
        let forward: Vec3f = self.get_forward();

        let world_up = Vec3f::new(0.0, 1.0, 0.0);

        let right = forward.cross(&world_up).normalize();
        let up = right.cross(&forward).normalize();

        right * direction.x() + up * direction.y() + forward * direction.z()
    }
}
