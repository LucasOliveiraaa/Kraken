use num_traits::{Float, Num};

use crate::vector::Vector;

pub struct Matrix<T: Num + Copy, const R: usize, const C: usize> {
    data: [[T; C]; R],
}

impl<T: Num + Copy, const R: usize, const C: usize> Matrix<T, R, C> {
    pub fn new(data: [[T; C]; R]) -> Self {
        Self { data }
    }

    pub fn identity(size: usize) -> Self {
        assert_eq!(R, C, "Identity matrix must be square");

        let mut data = [[T::zero(); C]; R];
        for i in 0..size {
            data[i][i] = T::one();
        }
        Self { data }
    }

    pub fn transpose(&self) -> Matrix<T, C, R> {
        let mut transposed_data = [[T::zero(); R]; C];
        for i in 0..R {
            for j in 0..C {
                transposed_data[j][i] = self.data[i][j];
            }
        }
        Matrix {
            data: transposed_data,
        }
    }

    pub fn det(&self) -> T {
        assert_eq!(
            R, C,
            "Determinant can only be calculated for square matrices"
        );

        if R == 2 {
            self.data[0][0] * self.data[1][1] - self.data[0][1] * self.data[1][0]
        } else if R == 3 {
            let a = self.data[0][0];
            let b = self.data[0][1];
            let c = self.data[0][2];
            let d = self.data[1][0];
            let e = self.data[1][1];
            let f = self.data[1][2];
            let g = self.data[2][0];
            let h = self.data[2][1];
            let i = self.data[2][2];

            a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
        } else if R == 4 {
            let m = self.as_flat_data();

            let a0 = m[0] * m[5] - m[1] * m[4];
            let a1 = m[0] * m[6] - m[2] * m[4];
            let a2 = m[0] * m[7] - m[3] * m[4];
            let a3 = m[1] * m[6] - m[2] * m[5];
            let a4 = m[1] * m[7] - m[3] * m[5];
            let a5 = m[2] * m[7] - m[3] * m[6];

            let b0 = m[8] * m[13] - m[9] * m[12];
            let b1 = m[8] * m[14] - m[10] * m[12];
            let b2 = m[8] * m[15] - m[11] * m[12];
            let b3 = m[9] * m[14] - m[10] * m[13];
            let b4 = m[9] * m[15] - m[11] * m[13];
            let b5 = m[10] * m[15] - m[11] * m[14];

            a0 * b5 - a1 * b4 + a2 * b3 + a3 * b2 - a4 * b1 + a5 * b0
        } else {
            unimplemented!(
                "Determinant calculation is only implemented for 2x2, 3x3 and 4x4 matrices"
            );
        }
    }

    pub fn data(&self) -> &[[T; C]; R] {
        &self.data
    }

    pub fn as_flat_data(&self) -> &[T] {
        let ptr = self.data.as_ptr() as *const T;
        unsafe { std::slice::from_raw_parts(ptr, R * C) }
    }
}

impl<T: Float, const R: usize, const C: usize> Matrix<T, R, C> {
    pub fn inverse(&self) -> Option<Matrix<T, R, C>> {
        assert_eq!(R, C, "Inverse can only be calculated for square matrices");

        let det = self.det();
        if det == T::zero() {
            return None;
        }

        if R == 2 && C == 2 {
            let a = self.data[0][0];
            let b = self.data[0][1];
            let c = self.data[1][0];
            let d = self.data[1][1];

            let inv_det = T::one() / det;
            let mut data = [[T::zero(); C]; R];
            data[0][0] = d * inv_det;
            data[0][1] = -b * inv_det;
            data[1][0] = -c * inv_det;
            data[1][1] = a * inv_det;
            Some(Matrix::new(data))
        } else if R == 3 && C == 3 {
            let a = self.data[0][0];
            let b = self.data[0][1];
            let c = self.data[0][2];
            let d = self.data[1][0];
            let e = self.data[1][1];
            let f = self.data[1][2];
            let g = self.data[2][0];
            let h = self.data[2][1];
            let i = self.data[2][2];

            let inv_det = T::one() / det;
            let mut data = [[T::zero(); C]; R];
            data[0][0] = (e * i - f * h) * inv_det;
            data[0][1] = (c * h - b * i) * inv_det;
            data[0][2] = (b * f - c * e) * inv_det;

            data[1][0] = (f * g - d * i) * inv_det;
            data[1][1] = (a * i - c * g) * inv_det;
            data[1][2] = (c * d - a * f) * inv_det;

            data[2][0] = (d * h - e * g) * inv_det;
            data[2][1] = (b * g - a * h) * inv_det;
            data[2][2] = (a * e - b * d) * inv_det;

            Some(Matrix::new(data))
        } else if R == 4 && C == 4 {
            let m = self.as_flat_data();
            let mut inv = [T::zero(); 16];

            inv[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
                + m[9] * m[7] * m[14]
                + m[13] * m[6] * m[11]
                - m[13] * m[7] * m[10];

            inv[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
                - m[8] * m[7] * m[14]
                - m[12] * m[6] * m[11]
                + m[12] * m[7] * m[10];

            inv[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
                + m[8] * m[7] * m[13]
                + m[12] * m[5] * m[11]
                - m[12] * m[7] * m[9];

            inv[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
                - m[8] * m[6] * m[13]
                - m[12] * m[5] * m[10]
                + m[12] * m[6] * m[9];

            inv[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
                - m[9] * m[3] * m[14]
                - m[13] * m[2] * m[11]
                + m[13] * m[3] * m[10];

            inv[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
                + m[8] * m[3] * m[14]
                + m[12] * m[2] * m[11]
                - m[12] * m[3] * m[10];

            inv[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
                - m[8] * m[3] * m[13]
                - m[12] * m[1] * m[11]
                + m[12] * m[3] * m[9];

            inv[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
                + m[8] * m[2] * m[13]
                + m[12] * m[1] * m[10]
                - m[12] * m[2] * m[9];

            inv[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
                + m[5] * m[3] * m[14]
                + m[13] * m[2] * m[7]
                - m[13] * m[3] * m[6];

            inv[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
                - m[4] * m[3] * m[14]
                - m[12] * m[2] * m[7]
                + m[12] * m[3] * m[6];

            inv[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
                + m[4] * m[3] * m[13]
                + m[12] * m[1] * m[7]
                - m[12] * m[3] * m[5];

            inv[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
                - m[4] * m[2] * m[13]
                - m[12] * m[1] * m[6]
                + m[12] * m[2] * m[5];

            inv[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
                - m[5] * m[3] * m[10]
                - m[9] * m[2] * m[7]
                + m[9] * m[3] * m[6];

            inv[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
                + m[4] * m[3] * m[10]
                + m[8] * m[2] * m[7]
                - m[8] * m[3] * m[6];

            inv[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
                - m[4] * m[3] * m[9]
                - m[8] * m[1] * m[7]
                + m[8] * m[3] * m[5];

            inv[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
                + m[4] * m[2] * m[9]
                + m[8] * m[1] * m[6]
                - m[8] * m[2] * m[5];

            let det = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];

            if det == T::zero() {
                return Matrix::new([[T::zero(); C]; R]).inverse();
            }

            let inv_det = T::one() / det;
            for i in 0..16 {
                inv[i] = inv[i] * inv_det;
            }

            let mut inv_matrix = [[T::zero(); C]; R];
            for i in 0..C {
                for j in 0..R {
                    inv_matrix[i][j] = inv[i * R + j];
                }
            }

            Some(Matrix::new(inv_matrix))
        } else {
            unimplemented!("Inverse calculation is only implemented for 2x2, 3x3 and 4x4 matrices");
        }
    }
}

impl<T: Num + Copy, const R: usize, const C: usize> std::ops::Mul<Matrix<T, C, R>>
    for Matrix<T, R, C>
{
    type Output = Matrix<T, R, R>;

    fn mul(self, rhs: Matrix<T, C, R>) -> Self::Output {
        let mut result_data = [[T::zero(); R]; R];
        for i in 0..R {
            for j in 0..R {
                for k in 0..C {
                    result_data[i][j] = result_data[i][j] + self.data[i][k] * rhs.data[k][j];
                }
            }
        }
        Matrix::new(result_data)
    }
}

impl<T: Num + Copy, const R: usize, const C: usize> std::ops::Mul<Vector<T, C>>
    for Matrix<T, R, C>
{
    type Output = Vector<T, R>;

    fn mul(self, rhs: Vector<T, C>) -> Self::Output {
        let mut result_data = [T::zero(); R];
        for i in 0..R {
            for j in 0..C {
                result_data[i] = result_data[i] + self.data[i][j] * rhs[j];
            }
        }
        Vector::from_data(result_data)
    }
}

impl<T: Num + Copy, const R: usize, const C: usize> std::ops::Index<usize> for Matrix<T, R, C> {
    type Output = [T; C];

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: Num + Copy, const R: usize, const C: usize> std::ops::IndexMut<usize> for Matrix<T, R, C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}
