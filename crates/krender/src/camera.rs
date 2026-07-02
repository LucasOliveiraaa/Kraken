use kmath::{Mat4, Vec2f, Vec3f};

pub struct Camera {
    pub position: Vec3f,
    pub rotation: Vec3f,
    pub size: Vec2f,

    pub fov: f32,
    pub aspect_ratio: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Camera {
    pub fn new(
        position: Vec3f,
        rotation: Vec3f,
        size: Vec2f,
        fov: f32,
        near_plane: f32,
        far_plane: f32,
    ) -> Self {
        let aspect_ratio = size.x() / size.y();
        Self {
            position,
            rotation,
            size,
            fov,
            aspect_ratio,
            near_plane,
            far_plane,
        }
    }

    pub fn update_size(&mut self, new_size: Vec2f) {
        self.aspect_ratio = new_size.x() / new_size.y();
        self.size = new_size;
    }

    pub fn update_position(&mut self, new_position: Vec3f) {
        self.position = new_position;
    }

    pub fn update_rotation(&mut self, new_rotation: Vec3f) {
        self.rotation = new_rotation;
    }

    pub fn update_fov(&mut self, new_fov: f32) {
        self.fov = new_fov;
    }

    pub fn update_near_plane(&mut self, new_near_plane: f32) {
        self.near_plane = new_near_plane;
    }

    pub fn update_far_plane(&mut self, new_far_plane: f32) {
        self.far_plane = new_far_plane;
    }

    pub fn position(&self) -> Vec3f {
        self.position
    }

    pub fn rotation(&self) -> Vec3f {
        self.rotation
    }

    pub fn size(&self) -> Vec2f {
        self.size
    }

    pub fn fov(&self) -> f32 {
        self.fov
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn near_plane(&self) -> f32 {
        self.near_plane
    }

    pub fn far_plane(&self) -> f32 {
        self.far_plane
    }

    pub fn get_view_matrix(&self) -> Mat4<f32> {
        Mat4::from_transform(self.position, self.rotation, Vec3f::new(1.0, 1.0, 1.0))
            .inverse()
            .unwrap()
            .into()
    }
}
