use crate::{Mat4f, Vec3f};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec3f,
    pub rotation: Vec3f,
    pub scale: Vec3f,
}

impl Transform {
    pub fn new(position: Vec3f, rotation: Vec3f, scale: Vec3f) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn identity() -> Self {
        Self {
            position: Vec3f::new(0.0, 0.0, 0.0),
            rotation: Vec3f::new(0.0, 0.0, 0.0),
            scale: Vec3f::new(1.0, 1.0, 1.0),
        }
    }

    pub fn from_position(&self) -> Mat4f {
        let mut data = [[0.0; 4]; 4];
        data[0][0] = 1.0;
        data[1][1] = 1.0;
        data[2][2] = 1.0;
        data[3][3] = 1.0;
        data[0][3] = self.position.x();
        data[1][3] = self.position.y();
        data[2][3] = self.position.z();
        Mat4f::new(data)
    }

    pub fn from_scale(&self) -> Mat4f {
        let mut data = [[0.0; 4]; 4];
        data[0][0] = self.scale.x();
        data[1][1] = self.scale.y();
        data[2][2] = self.scale.z();
        data[3][3] = 1.0;
        Mat4f::new(data)
    }

    pub fn from_rotation(&self) -> Mat4f {
        let (sin_x, cos_x) = self.rotation.x().sin_cos();
        let (sin_y, cos_y) = self.rotation.y().sin_cos();
        let (sin_z, cos_z) = self.rotation.z().sin_cos();

        let mut data = [[0.0; 4]; 4];
        data[0][0] = cos_y * cos_z + sin_y * sin_x * sin_z;
        data[0][1] = -cos_y * sin_z + sin_y * sin_x * cos_z;
        data[0][2] = sin_y * cos_x;

        data[1][0] = cos_x * sin_z;
        data[1][1] = cos_x * cos_z;
        data[1][2] = -sin_x;

        data[2][0] = -sin_y * cos_z + cos_y * sin_x * sin_z;
        data[2][1] = sin_y * sin_z + cos_y * sin_x * cos_z;
        data[2][2] = cos_y * cos_x;

        data[3][3] = 1.0;
        Mat4f::new(data)
    }

    pub fn to_matrix(&self) -> Mat4f {
        let translation_matrix = self.from_position();
        let rotation_matrix = self.from_rotation();
        let scale_matrix = self.from_scale();

        translation_matrix * rotation_matrix * scale_matrix
    }
}
