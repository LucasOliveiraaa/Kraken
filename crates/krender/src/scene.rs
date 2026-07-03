use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use glow::{HasContext, NativeBuffer};
use kmath::{Mat4, Vec3f};

use crate::bindings;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GlPlane {
    pub height: f32,
    pub max_h: f32,
    pub min_h: f32,
    pub first_cp_idx: u32, // First control point index in the control point buffer
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
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
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GlVolume {
    pub base_node_idx: u32, // 4-bit
    pub _pad: [u32; 3],     // 12-bit

    pub world_to_volume: [[f32; 4]; 4], // 16-bit
    pub volume_to_world: [[f32; 4]; 4], // 16-bit

    pub min_p: [f32; 3],
    pub _pad2: f32,
    pub max_p: [f32; 3],
    pub _pad3: f32,
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
    let volume_to_world = Mat4::from_transform(
        Vec3f::new(0.0, 0.0, 0.0),
        Vec3f::new(std::f32::consts::FRAC_PI_2, 0.0, 0.0),
        Vec3f::new(1.0, 1.0, 1.0),
    );

    SceneData {
        control_points: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        planes: vec![GlPlane {
            first_cp_idx: 0,
            height: 0.0,
            max_h: 0.0,
            min_h: 0.0,
        }],
        quad_nodes: vec![GlQuadNode {
            children: 0,
            first_idx: 0,
            plane_count: 1,
            _pad: 0,
            position: [-0.5, -0.5],
            size: 1.0,
            _pad2: 0.0,
        }],
        volumes: vec![GlVolume {
            base_node_idx: 0,
            _pad: [0; 3],
            world_to_volume: *volume_to_world.inverse().unwrap().data(),
            volume_to_world: *volume_to_world.data(),
            min_p: [-1.5, -1.5, -1.5],
            _pad2: 0.0,
            max_p: [1.5, 1.5, 1.5],
            _pad3: 0.0,
        }],
    }
}

/// GL buffer handles for an uploaded [`SceneData`].
pub struct SceneBuffers {
    gl: Arc<glow::Context>,

    volumes_count: u32,

    control_points: NativeBuffer,
    planes: NativeBuffer,
    quad_nodes: NativeBuffer,
    volumes: NativeBuffer,
}

impl SceneBuffers {
    pub unsafe fn upload(gl: Arc<glow::Context>, data: &SceneData) -> Self {
        fn upload_ssbo<T: Copy + Pod>(
            gl: Arc<glow::Context>,
            data: &[T],
            binding: u32,
        ) -> NativeBuffer {
            let buffer = unsafe { gl.create_buffer().unwrap() };
            unsafe {
                gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(buffer));
                gl.buffer_data_u8_slice(
                    glow::SHADER_STORAGE_BUFFER,
                    bytemuck::cast_slice(data),
                    glow::STATIC_DRAW,
                );
                gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, binding, Some(buffer));
            }
            buffer
        }

        Self {
            gl: gl.clone(),
            volumes_count: data.volumes.len() as u32,
            control_points: upload_ssbo(gl.clone(), &data.control_points, bindings::CONTROL_POINTS),
            planes: upload_ssbo(gl.clone(), &data.planes, bindings::PLANES),
            quad_nodes: upload_ssbo(gl.clone(), &data.quad_nodes, bindings::QUAD_TREE),
            volumes: upload_ssbo(gl.clone(), &data.volumes, bindings::VOLUMES),
        }
    }

    pub fn volumes_count(&self) -> u32 {
        self.volumes_count
    }
}

impl Drop for SceneBuffers {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.control_points);
            self.gl.delete_buffer(self.planes);
            self.gl.delete_buffer(self.quad_nodes);
            self.gl.delete_buffer(self.volumes);
        }
    }
}
