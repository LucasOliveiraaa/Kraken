use ash::vk;
use contract::GpuResult;
use kmath::{Vec2f, Vec2u, Vec3f, Vec3u};

pub trait FromContract<T>: Sized {
    #[must_use]
    fn from_contract(value: T) -> Self;
}
pub trait IntoVk<T>: Sized {
    fn into_vk(self) -> T;
}
impl<T, U> IntoVk<U> for T
where
    U: FromContract<T>,
{
    #[inline]
    #[track_caller]
    fn into_vk(self) -> U {
        U::from_contract(self)
    }
}

pub trait FromVk<T> {
    fn from_vk(value: T) -> Self;
}
pub trait IntoContract<T>: Sized {
    fn into_contract(self) -> T;
}
impl<T, U> IntoContract<U> for T
where
    U: FromVk<T>,
{
    #[inline]
    #[track_caller]
    fn into_contract(self) -> U {
        U::from_vk(self)
    }
}

pub trait TryFromVk<T>: Sized {
    fn try_from_vk(value: T) -> GpuResult<Self>;
}
pub trait TryIntoContract<T>: Sized {
    fn try_into_contract(self) -> GpuResult<T>;
}
impl<T, U> TryIntoContract<U> for T
where
    U: TryFromVk<T>,
{
    #[inline]
    #[track_caller]
    fn try_into_contract(self) -> GpuResult<U> {
        U::try_from_vk(self)
    }
}

//////////////////////////////////////////////////////
// COMMON CONVERSIONS
//////////////////////////////////////////////////////

impl FromContract<bool> for vk::Bool32 {
    #[inline]
    #[track_caller]
    fn from_contract(value: bool) -> Self {
        if value { 1 } else { 0 }
    }
}
impl FromVk<vk::Bool32> for bool {
    #[inline]
    #[track_caller]
    fn from_vk(value: vk::Bool32) -> Self {
        value != 0
    }
}

impl FromContract<Vec2u> for vk::Extent2D {
    fn from_contract(value: Vec2u) -> Self {
        vk::Extent2D {
            width: value.x(),
            height: value.y(),
        }
    }
}
impl FromVk<vk::Extent2D> for Vec2u {
    fn from_vk(value: vk::Extent2D) -> Self {
        Vec2u::new(value.width, value.height)
    }
}

impl FromContract<Vec2f> for vk::Extent2D {
    fn from_contract(value: Vec2f) -> Self {
        vk::Extent2D {
            width: value.x() as u32,
            height: value.y() as u32,
        }
    }
}
impl FromVk<vk::Extent2D> for Vec2f {
    fn from_vk(value: vk::Extent2D) -> Self {
        Vec2f::new(value.width as f32, value.height as f32)
    }
}

impl FromContract<Vec3u> for vk::Extent3D {
    fn from_contract(value: Vec3u) -> Self {
        vk::Extent3D {
            width: value.x(),
            height: value.y(),
            depth: value.z(),
        }
    }
}
impl FromVk<vk::Extent3D> for Vec3u {
    fn from_vk(value: vk::Extent3D) -> Self {
        Vec3u::new(value.width, value.height, value.depth)
    }
}

impl FromContract<Vec3f> for vk::Extent3D {
    fn from_contract(value: Vec3f) -> Self {
        vk::Extent3D {
            width: value.x() as u32,
            height: value.y() as u32,
            depth: value.z() as u32,
        }
    }
}
impl FromVk<vk::Extent3D> for Vec3f {
    fn from_vk(value: vk::Extent3D) -> Self {
        Vec3f::new(value.width as f32, value.height as f32, value.depth as f32)
    }
}
