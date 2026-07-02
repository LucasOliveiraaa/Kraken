use std::array;
use num_traits::{Float, Num};

use crate::matrix::Matrix;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector<T: Num + Copy, const N: usize> {
    data: [T; N],
}

impl<T: Num + Copy, const N: usize> Vector<T, N> {
    pub fn new(data: [T; N]) -> Self {
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

impl<T: Num + Copy, const N: usize> std::ops::Sub for Vector<T, N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            data: array::from_fn(|i| self.data[i] - rhs.data[i]),
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
        Vector::new(std::array::from_fn(|col| {
            let mut sum = T::zero();

            for row in 0..R {
                sum = sum + self[row] * matrix[row][col];
            }

            sum
        }))
    }
}