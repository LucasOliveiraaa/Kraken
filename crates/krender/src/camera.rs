use kmath::{Mat4, Vec2f, Vec3f};

pub struct Camera {
    position: Vec3f,
    rotation: Vec3f,
    size: Vec2f,

    fov: f32,
    aspect_ratio: f32,
    near_plane: f32,
    far_plane: f32,

    pub dirty: bool,
    pub speed: f32,
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
            dirty: true,
            speed: 1.0,
        }
    }

    pub fn update_size(&mut self, new_size: Vec2f) {
        self.aspect_ratio = new_size.x() / new_size.y();
        self.size = new_size;
        self.dirty = true;
    }

    pub fn update_position(&mut self, new_position: Vec3f) {
        self.position = new_position;
        self.dirty = true;
    }

    pub fn update_rotation(&mut self, new_rotation: Vec3f) {
        self.rotation = new_rotation;
        self.dirty = true;
    }

    pub fn update_fov(&mut self, new_fov: f32) {
        self.fov = new_fov;
        self.dirty = true;
    }

    pub fn update_near_plane(&mut self, new_near_plane: f32) {
        self.near_plane = new_near_plane;
        self.dirty = true;
    }

    pub fn update_far_plane(&mut self, new_far_plane: f32) {
        self.far_plane = new_far_plane;
        self.dirty = true;
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

    pub fn get_proj_matrix(&self) -> Mat4<f32> {
        Mat4::from_perspective(self.fov, self.aspect_ratio, self.near_plane, self.far_plane)
            .inverse()
            .unwrap()
            .into()
    }

    pub fn set_speed(&mut self, new_speed: f32) {
        self.speed = new_speed;
    }

    pub fn move_towards(&mut self, direction: Vec3f) {
        let forward: Vec3f = Vec3f::new(
            self.rotation.y().sin() * self.rotation.x().cos(),
            -self.rotation.x().sin(),
            self.rotation.y().cos() * self.rotation.x().cos(),
        )
        .normalize();

        let world_up = Vec3f::new(0.0, 1.0, 0.0);

        let right = forward.cross(&world_up).normalize();
        let up = right.cross(&forward).normalize();

        self.position += right * direction.x() + up * direction.y() + forward * direction.z() * self.speed;

        self.dirty = true;
    }

    pub fn rotate(&mut self, delta_rotation: Vec2f) {
        self.rotation += Vec3f::new(delta_rotation.y(), delta_rotation.x(), 0.0);
        self.dirty = true;
    }
}
