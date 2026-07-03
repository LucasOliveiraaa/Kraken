mod matrix;
mod vector;

use std::ops::{Deref, DerefMut};

pub use matrix::Matrix;
use num_traits::{Float, Num};
pub use vector::Vector;

pub type Vec2<T> = Vector<T, 2>;
pub type Vec3<T> = Vector<T, 3>;
pub type Vec4<T> = Vector<T, 4>;

pub type Vec2f = Vec2<f32>;
pub type Vec3f = Vec3<f32>;
pub type Vec4f = Vec4<f32>;

pub type Vec2i = Vec2<i32>;
pub type Vec3i = Vec3<i32>;
pub type Vec4i = Vec4<i32>;

pub type Vec2u = Vec2<u32>;
pub type Vec3u = Vec3<u32>;
pub type Vec4u = Vec4<u32>;

pub struct Mat4<T: Num + Copy>(pub Matrix<T, 4, 4>);
impl<T: Float> Mat4<T> {
    pub fn new(data: [[T; 4]; 4]) -> Self {
        Self(Matrix::new(data))
    }

    pub fn from_translation(translation: Vector<T, 3>) -> Self {
        let mut data = [[T::zero(); 4]; 4];
        data[0][0] = T::one();
        data[1][1] = T::one();
        data[2][2] = T::one();
        data[3][3] = T::one();
        data[0][3] = translation.x();
        data[1][3] = translation.y();
        data[2][3] = translation.z();
        Self(Matrix::new(data))
    }

    pub fn from_scale(scale: Vec3<T>) -> Self {
        let mut data = [[T::zero(); 4]; 4];
        data[0][0] = scale.x();
        data[1][1] = scale.y();
        data[2][2] = scale.z();
        data[3][3] = T::one();
        Self(Matrix::new(data))
    }

    pub fn from_rotation(rotation: Vec3<T>) -> Self {
        let (sin_x, cos_x) = rotation.x().sin_cos();
        let (sin_y, cos_y) = rotation.y().sin_cos();
        let (sin_z, cos_z) = rotation.z().sin_cos();

        let mut data = [[T::zero(); 4]; 4];
        data[0][0] = cos_y * cos_z;
        data[0][1] = -cos_y * sin_z;
        data[0][2] = sin_y;
        data[1][0] = sin_x * sin_y * cos_z + cos_x * sin_z;
        data[1][1] = -sin_x * sin_y * sin_z + cos_x * cos_z;
        data[1][2] = -sin_x * cos_y;
        data[2][0] = -cos_x * sin_y * cos_z + sin_x * sin_z;
        data[2][1] = cos_x * sin_y * sin_z + sin_x * cos_z;
        data[2][2] = cos_x * cos_y;
        data[3][3] = T::one();
        Self(Matrix::new(data))
    }

    pub fn from_transform(position: Vec3<T>, rotation: Vec3<T>, scale: Vec3<T>) -> Self {
        let translation_matrix = Mat4::from_translation(position);
        let rotation_matrix = Mat4::from_rotation(rotation);
        let scale_matrix = Mat4::from_scale(scale);

        translation_matrix * rotation_matrix * scale_matrix
    }

    pub fn from_perspective(fov_y: T, aspect: T, near: T, far: T) -> Self {
        let f = T::one() / (fov_y * T::one() / (T::one() + T::one())).tan();

        Mat4::new([
            [f / aspect, T::zero(), T::zero(), T::zero()],
            [T::zero(), f, T::zero(), T::zero()],
            [
                T::zero(),
                T::zero(),
                far / (near - far),
                (far * near) / (near - far),
            ],
            [T::zero(), T::zero(), -T::one(), T::zero()],
        ])
    }
}

impl<T: Num + Copy> std::ops::Mul for Mat4<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl<T: Num + Copy> Into<Matrix<T, 4, 4>> for Mat4<T> {
    fn into(self) -> Matrix<T, 4, 4> {
        self.0
    }
}

impl<T: Num + Copy> Into<Mat4<T>> for Matrix<T, 4, 4> {
    fn into(self) -> Mat4<T> {
        Mat4(self)
    }
}

impl<T: Num + Copy> Deref for Mat4<T> {
    type Target = Matrix<T, 4, 4>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Num + Copy> DerefMut for Mat4<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
