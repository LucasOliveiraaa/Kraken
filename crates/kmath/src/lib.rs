mod matrix;
mod vector;

use std::ops::{Deref, DerefMut};

pub use matrix::Matrix;
use num_traits::{Float, Num};
pub use vector::Vector;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2<T: Num + Copy>(pub Vector<T, 2>);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3<T: Num + Copy>(pub Vector<T, 3>);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec4<T: Num + Copy>(pub Vector<T, 4>);

impl<T: Num + Copy> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self(Vector::new([x, y]))
    }

    pub fn x(&self) -> T {
        self.0.data()[0]
    }

    pub fn y(&self) -> T {
        self.0.data()[1]
    }

    pub fn extend(&self, z: T) -> Vec3<T> {
        Vec3::new(self.x(), self.y(), z)
    }
}

impl<T: Num + Copy> Deref for Vec2<T> {
    type Target = Vector<T, 2>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Num + Copy> DerefMut for Vec2<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Num + Copy> Vec3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self(Vector::new([x, y, z]))
    }

    pub fn x(&self) -> T {
        self.0.data()[0]
    }

    pub fn y(&self) -> T {
        self.0.data()[1]
    }

    pub fn z(&self) -> T {
        self.0.data()[2]
    }

    pub fn extend(&self, w: T) -> Vec4<T> {
        Vec4::new(self.x(), self.y(), self.z(), w)
    }

    pub fn truncate(&self) -> Vec2<T> {
        Vec2::new(self.x(), self.y())
    }
}

impl<T: Num + Copy> Deref for Vec3<T> {
    type Target = Vector<T, 3>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Num + Copy> DerefMut for Vec3<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Num + Copy> Vec4<T> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self(Vector::new([x, y, z, w]))
    }

    pub fn x(&self) -> T {
        self.0.data()[0]
    }

    pub fn y(&self) -> T {
        self.0.data()[1]
    }

    pub fn z(&self) -> T {
        self.0.data()[2]
    }

    pub fn w(&self) -> T {
        self.0.data()[3]
    }

    pub fn truncate(&self) -> Vec3<T> {
        Vec3::new(self.x(), self.y(), self.z())
    }
}

impl<T: Num + Copy> Deref for Vec4<T> {
    type Target = Vector<T, 4>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Num + Copy> DerefMut for Vec4<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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

    pub fn from_translation(translation: Vec3<T>) -> Self {
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
