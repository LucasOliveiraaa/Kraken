#![allow(unsafe_op_in_unsafe_fn)]

use crate::gl_util::upload_ssbo;
use crate::kmath::Mat4;
use crate::structure::{MockSceneBuilder, PlaneDef};
use std::f32::consts::FRAC_PI_2;

use crate::kmath::Vec3;

/// SSBO binding points shared between Rust and `bicubic.frag`.
pub mod bindings {
    pub const CONTROL_POINTS: u32 = 1;
    pub const PLANES: u32 = 2;
    pub const QUAD_TREE: u32 = 3;
    pub const VOLUMES: u32 = 4;
    pub const PROFILE: u32 = 5;
    pub const CONFIG: u32 = 6;
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GlPlane {
    pub height: f32,
    pub max_h: f32,
    pub min_h: f32,
    pub first_cp_idx: u32, // First control point index in the control point buffer
}

#[repr(C)]
pub struct GlQuadNode {
    pub children: u32,
    pub first_idx: u32,
    pub plane_count: u32,
    pub _pad: u32,
    pub position: [f32; 2],
    pub size: f32,
    pub _pad2: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GlVolume {
    pub base_node_idx: u32,
    pub _pad: [u32; 3], // 12-byte padding

    pub world_to_volume: [[f32; 4]; 4],
    pub volume_to_world: [[f32; 4]; 4],
}

/// CPU-side scene description, prior to upload.
pub struct SceneData {
    pub control_points: Vec<f32>,
    pub planes: Vec<GlPlane>,
    pub quad_nodes: Vec<GlQuadNode>,
    pub volumes: Vec<GlVolume>,
}

/// Small hardcoded test scene: a single leaf quad node covering
/// `[-0.5, -0.5]..[0.5, 0.5]`, two overlapping planes, one volume rotated
/// 90 degrees about X.
pub fn build_mock_scene() -> SceneData {
    MockSceneBuilder::new()
        .add_volume(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 1.0),
            vec![
                PlaneDef::new(1.0, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
                PlaneDef::new(1.1, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
                // PlaneDef::new(4.0, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ],
        )
        // .add_volume(
        //     Vec3::new(0.0, 10.0, 0.0),
        //     Vec3::new(0.0, 90.0, 0.0),
        //     Vec3::new(10.0, 10.0, 1.0),
        //     vec![
        //         PlaneDef::new(-4.0, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        //         PlaneDef::new(4.0, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        //     ],
        // )
        // .add_volume(
        //     Vec3::new(0.0, 10.0, 0.0),
        //     Vec3::new(90.0, 0.0, 0.0),
        //     Vec3::new(10.0, 10.0, 1.0),
        //     vec![
        //         PlaneDef::new(-4.0, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        //         PlaneDef::new(4.0, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
        //     ],
        // )
        .build()
}

/// GL buffer handles for an uploaded [`SceneData`].
pub struct SceneBuffers {
    pub control_points: u32,
    pub planes: u32,
    pub quad_nodes: u32,
    pub volumes: u32,
}

impl SceneBuffers {
    pub unsafe fn upload(data: &SceneData) -> Self {
        Self {
            control_points: upload_ssbo(&data.control_points, bindings::CONTROL_POINTS),
            planes: upload_ssbo(&data.planes, bindings::PLANES),
            quad_nodes: upload_ssbo(&data.quad_nodes, bindings::QUAD_TREE),
            volumes: upload_ssbo(&data.volumes, bindings::VOLUMES),
        }
    }

    pub unsafe fn destroy(&self) {
        let handles = [
            self.control_points,
            self.planes,
            self.quad_nodes,
            self.volumes,
        ];
        gl::DeleteBuffers(handles.len() as i32, handles.as_ptr());
    }
}