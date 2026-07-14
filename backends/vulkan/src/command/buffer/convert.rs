use ash::vk;
use contract::pipeline::{Scissor, Viewport};
use kmath::{Vec2u, Vec3i};

use crate::{FromContract, FromVk};

impl FromContract<Viewport> for vk::Viewport {
    fn from_contract(value: Viewport) -> Self {
        vk::Viewport {
            x: value.position.x(),
            y: value.position.y(),
            width: value.size.x(),
            height: value.size.y(),
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
}
impl FromVk<vk::Viewport> for Viewport {
    fn from_vk(value: vk::Viewport) -> Self {
        Viewport {
            position: kmath::Vec2f::new(value.x, value.y),
            size: kmath::Vec2f::new(value.width, value.height),
        }
    }
}

impl FromContract<Scissor> for vk::Rect2D {
    fn from_contract(value: Scissor) -> Self {
        vk::Rect2D {
            offset: vk::Offset2D {
                x: value.position.x() as i32,
                y: value.position.y() as i32,
            },
            extent: vk::Extent2D {
                width: value.size.x(),
                height: value.size.y(),
            },
        }
    }
}
impl FromVk<vk::Rect2D> for Scissor {
    fn from_vk(value: vk::Rect2D) -> Self {
        Scissor {
            position: Vec2u::new(value.offset.x as u32, value.offset.y as u32),
            size: Vec2u::new(value.extent.width, value.extent.height),
        }
    }
}

impl FromContract<Vec3i> for vk::Offset3D {
    fn from_contract(value: Vec3i) -> Self {
        vk::Offset3D {
            x: value.x(),
            y: value.y(),
            z: value.z(),
        }
    }
}
impl FromVk<vk::Offset3D> for Vec3i {
    fn from_vk(value: vk::Offset3D) -> Self {
        Vec3i::new(value.x, value.y, value.z)
    }
}