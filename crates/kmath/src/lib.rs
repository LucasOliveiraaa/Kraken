mod vector;
mod matrix;

use std::ops::{Deref, DerefMut};

use num_traits::Float;
pub use vector::Vector;
pub use matrix::Matrix;

pub struct Vec2<T: Float>(pub Vector<T, 2>);
pub struct Vec3<T: Float>(pub Vector<T, 3>);
pub struct Vec4<T: Float>(pub Vector<T, 4>);

impl<T: Float> Vec2<T> {
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

impl<T: Float> Deref for Vec2<T> {
    type Target = Vector<T, 2>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Float> DerefMut for Vec2<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Float> Vec3<T> {
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

impl<T: Float> Deref for Vec3<T> {
    type Target = Vector<T, 3>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Float> DerefMut for Vec3<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Float> Vec4<T> {
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

impl<T: Float> Deref for Vec4<T> {
    type Target = Vector<T, 4>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Float> DerefMut for Vec4<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub type Vec2f = Vec2<f32>;
pub type Vec3f = Vec3<f32>;
pub type Vec4f = Vec4<f32>;


pub struct Mat4<T: Float>(pub Matrix<T, 4, 4>);
impl<T: Float> Mat4<T> {
    pub fn new(data: [[T; 4]; 4]) -> Self {
        Self(Matrix::new(data))
    }
}

impl<T: Float> Deref for Mat4<T> {
    type Target = Matrix<T, 4, 4>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Float> DerefMut for Mat4<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}