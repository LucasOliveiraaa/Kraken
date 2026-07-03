use num_traits::{Float, Num, NumCast};
use std::{
    array,
    fmt::{Debug, Display},
    ops::Neg,
};

use crate::matrix::Matrix;

#[derive(Clone, Copy, PartialEq)]
pub struct Vector<T: Num + Copy, const N: usize> {
    data: [T; N],
}

impl<T: Num + Copy, const N: usize> Vector<T, N> {
    pub fn from_data(data: [T; N]) -> Self {
        Self { data }
    }

    pub fn splat(value: T) -> Self {
        Self { data: [value; N] }
    }

    pub fn from_slice(slice: &[T]) -> Self {
        assert_eq!(slice.len(), N);

        Self {
            data: array::from_fn(|i| slice[i]),
        }
    }

    pub fn data(&self) -> &[T; N] {
        &self.data
    }

    pub const fn len(&self) -> usize {
        N
    }

    pub fn dot(&self, other: &Self) -> T {
        self.data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| *a * *b)
            .fold(T::zero(), |acc, x| acc + x)
    }

    pub fn length_squared(&self) -> T {
        self.dot(self)
    }

    pub fn convert_to<U: NumCast + Num + Copy>(&self) -> Vector<U, N>
    where
        T: NumCast + Num + Copy,
    {
        Vector::from_data(std::array::from_fn(|i| {
            NumCast::from(self.data[i]).unwrap()
        }))
    }
}

impl<T: Float, const N: usize> Vector<T, N> {
    pub fn magnitude(&self) -> T {
        self.length_squared().sqrt()
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();

        if mag == T::zero() {
            return Self::splat(T::zero());
        }

        Self {
            data: array::from_fn(|i| self.data[i] / mag),
        }
    }
}

impl<T: Num + Copy, const N: usize> std::ops::Add for Vector<T, N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            data: array::from_fn(|i| self.data[i] + rhs.data[i]),
        }
    }
}

impl<T: Num + Copy, const N: usize> std::ops::AddAssign for Vector<T, N> {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.data[i] = self.data[i] + rhs.data[i];
        }
    }
}

impl<T: Num + Copy, const N: usize> std::ops::Sub for Vector<T, N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            data: array::from_fn(|i| self.data[i] - rhs.data[i]),
        }
    }
}

impl<T: Num + Copy, const N: usize> std::ops::SubAssign for Vector<T, N> {
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.data[i] = self.data[i] - rhs.data[i];
        }
    }
}

impl<T: Num + Copy, const N: usize> std::ops::Mul<T> for Vector<T, N> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        Self {
            data: array::from_fn(|i| self.data[i] * scalar),
        }
    }
}

impl<T: Num + Copy, const N: usize> std::ops::Div<T> for Vector<T, N> {
    type Output = Self;

    fn div(self, scalar: T) -> Self {
        Self {
            data: array::from_fn(|i| self.data[i] / scalar),
        }
    }
}

impl<T: Float, const N: usize> std::ops::Neg for Vector<T, N> {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            data: array::from_fn(|i| -self.data[i]),
        }
    }
}

impl<T: Num + Copy, const N: usize> std::ops::Index<usize> for Vector<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: Num + Copy, const N: usize> std::ops::IndexMut<usize> for Vector<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T: Num + Copy, const R: usize, const C: usize> std::ops::Mul<Matrix<T, R, C>>
    for Vector<T, R>
{
    type Output = Vector<T, C>;

    fn mul(self, matrix: Matrix<T, R, C>) -> Self::Output {
        Vector::from_data(std::array::from_fn(|col| {
            let mut sum = T::zero();

            for row in 0..R {
                sum = sum + self[row] * matrix[row][col];
            }

            sum
        }))
    }
}

impl<T: Num + Copy + Debug, const N: usize> Debug for Vector<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.data.iter()).finish()
    }
}

impl<T: Num + Copy> Vector<T, 2> {
    pub fn new(x: T, y: T) -> Self {
        Self { data: [x, y] }
    }

    pub fn x(&self) -> T {
        self.data[0]
    }

    pub fn y(&self) -> T {
        self.data[1]
    }

    pub fn extend(&self, z: T) -> Vector<T, 3> {
        Vector::<T, 3>::new(self.x(), self.y(), z)
    }
}

impl<T: Num + Copy> Vector<T, 3> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { data: [x, y, z] }
    }

    pub fn x(&self) -> T {
        self.data[0]
    }

    pub fn y(&self) -> T {
        self.data[1]
    }

    pub fn z(&self) -> T {
        self.data[2]
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            data: [
                self.y() * other.z() - self.z() * other.y(),
                self.z() * other.x() - self.x() * other.z(),
                self.x() * other.y() - self.y() * other.x(),
            ],
        }
    }

    pub fn extend(&self, w: T) -> Vector<T, 4> {
        Vector::<T, 4>::new(self.x(), self.y(), self.z(), w)
    }

    pub fn truncate(&self) -> Vector<T, 2> {
        Vector::<T, 2>::new(self.x(), self.y())
    }
}

impl<T: Num + Copy> Vector<T, 4> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { data: [x, y, z, w] }
    }

    pub fn x(&self) -> T {
        self.data[0]
    }

    pub fn y(&self) -> T {
        self.data[1]
    }

    pub fn z(&self) -> T {
        self.data[2]
    }

    pub fn w(&self) -> T {
        self.data[3]
    }

    pub fn truncate(&self) -> Vector<T, 3> {
        Vector::<T, 3>::new(self.x(), self.y(), self.z())
    }
}
