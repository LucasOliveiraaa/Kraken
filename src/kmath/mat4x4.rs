use std::ops::Mul;

use crate::kmath::{Vec3, vec4::Vec4};

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Mat4 {
    pub m: [f32; 16],
}

impl Mat4 {
    pub const ZERO: Self = Self { m: [0.0; 16] };

    pub const IDENTITY: Self = Self {
        m: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    };

    #[inline]
    pub fn new(m: [f32; 16]) -> Self {
        Self { m }
    }

    #[inline]
    pub fn col(&self, i: usize) -> Vec4 {
        let b = i * 4;
        Vec4::new(self.m[b], self.m[b + 1], self.m[b + 2], self.m[b + 3])
    }

    #[inline]
    pub fn row(&self, i: usize) -> Vec4 {
        Vec4::new(self.m[i], self.m[i + 4], self.m[i + 8], self.m[i + 12])
    }

    #[inline]
    pub fn transpose(self) -> Self {
        let m = self.m;
        Self {
            m: [
                m[0], m[4], m[8], m[12], m[1], m[5], m[9], m[13], m[2], m[6], m[10], m[14], m[3],
                m[7], m[11], m[15],
            ],
        }
    }

    pub fn det(&self) -> f32 {
        let m = &self.m;

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
    }

    pub fn inverse(&self) -> Self {
        let m = &self.m;
        let mut inv = [0.0f32; 16];

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

        if det == 0.0 {
            return Mat4::ZERO;
        }

        let inv_det = 1.0 / det;
        for i in 0..16 {
            inv[i] *= inv_det;
        }

        Mat4 { m: inv }
    }
}

impl Mat4 {
    pub fn from_euler_rotation_x(rx: f32) -> Self {
        let c = rx.cos();
        let s = rx.sin();

        Mat4::new([
            1.0, 0.0, 0.0, 0.0, 0.0, c, -s, 0.0, 0.0, s, c, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }
    pub fn from_euler_rotation_y(rx: f32) -> Self {
        let c = rx.cos();
        let s = rx.sin();

        Mat4::new([
            c, 0.0, s, 0.0, 0.0, 1.0, 0.0, 0.0, -s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }
    pub fn from_euler_rotation_z(rx: f32) -> Self {
        let c = rx.cos();
        let s = rx.sin();

        Mat4::new([
            c, -s, 0.0, 0.0, s, c, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }
    pub fn from_euler_rotation(r: Vec3) -> Self {
        Self::from_euler_rotation_x(r.x)
            * Self::from_euler_rotation_y(r.y)
            * Self::from_euler_rotation_z(r.z)
    }

    pub fn from_translation(t: Vec3) -> Self {
        Mat4::new([
            1.0, 0.0, 0.0, t.x, 0.0, 1.0, 0.0, t.y, 0.0, 0.0, 1.0, t.z, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn from_scale(s: Vec3) -> Self {
        Mat4::new([
            s.x, 0.0, 0.0, 0.0, 0.0, s.y, 0.0, 0.0, 0.0, 0.0, s.z, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn from_transform(t: Vec3, r: Vec3, s: Vec3) -> Self {
        Self::from_translation(t) * Self::from_euler_rotation(r) * Self::from_scale(s)
    }

    pub fn from_perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y * 0.5).tan();

        Mat4::new([
            f / aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            f,
            0.0,
            0.0,
            0.0,
            0.0,
            far / (near - far),
            (far * near) / (near - far),
            0.0,
            0.0,
            -1.0,
            0.0,
        ])
    }

    pub fn transform_point3(&self, p: Vec3) -> Vec3 {
        let x = self.m[0] * p.x + self.m[4] * p.y + self.m[8] * p.z + self.m[12];
        let y = self.m[1] * p.x + self.m[5] * p.y + self.m[9] * p.z + self.m[13];
        let z = self.m[2] * p.x + self.m[6] * p.y + self.m[10] * p.z + self.m[14];
        let w = self.m[3] * p.x + self.m[7] * p.y + self.m[11] * p.z + self.m[15];

        if w != 0.0 {
            Vec3::new(x / w, y / w, z / w)
        } else {
            Vec3::new(x, y, z)
        }
    }

    pub fn as_ptr(&self) -> *const f32 {
        self.m.as_ptr()
    }

    pub fn into_cols_array_2d(self) -> [[f32; 4]; 4] {
        unsafe { *(self.m.as_ptr() as *const [[f32; 4]; 4]) }
    }

    pub fn from_cols(c0: Vec4, c1: Vec4, c2: Vec4, c3: Vec4) -> Self {
        Mat4::new([
            c0.x, c0.y, c0.z, c0.w, c1.x, c1.y, c1.z, c1.w, c2.x, c2.y, c2.z, c2.w, c3.x, c3.y,
            c3.z, c3.w,
        ])
    }
}

impl Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        let mut out = [0.0; 16];

        for c in 0..4 {
            for r in 0..4 {
                out[c * 4 + r] = self.m[0 * 4 + r] * rhs.m[c * 4 + 0]
                    + self.m[1 * 4 + r] * rhs.m[c * 4 + 1]
                    + self.m[2 * 4 + r] * rhs.m[c * 4 + 2]
                    + self.m[3 * 4 + r] * rhs.m[c * 4 + 3];
            }
        }

        Mat4 { m: out }
    }
}
