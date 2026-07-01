use crate::{
    kmath::{Mat4, Vec3}, scene::{GlPlane, GlQuadNode, GlVolume, SceneData},
};

pub struct PlaneDef {
    pub height: f32,
    pub control_points: [f32; 9],
}

impl PlaneDef {
    pub fn new(height: f32, control_points: [f32; 9]) -> Self {
        Self {
            height,
            control_points,
        }
    }
}

pub struct MockSceneBuilder {
    pub control_points: Vec<f32>,
    pub planes: Vec<GlPlane>,
    pub quad_nodes: Vec<GlQuadNode>,
    pub volumes: Vec<GlVolume>,
}

impl MockSceneBuilder {
    pub fn new() -> Self {
        Self {
            control_points: Vec::new(),
            planes: Vec::new(),
            quad_nodes: Vec::new(),
            volumes: Vec::new(),
        }
    }

    pub fn add_volume(
        mut self,
        position: Vec3,
        rotation: Vec3,
        size: Vec3,
        planes: Vec<PlaneDef>,
    ) -> Self {
        let rotation = Vec3::new((rotation.x + 90.0).to_radians(), rotation.y.to_radians(), rotation.z.to_radians());

        let planes = planes
            .into_iter()
            .map(|p| {
                let first_cp_idx = self.control_points.len();
                let max_h = p.control_points.iter().copied().fold(f32::MIN, f32::max) + p.height;
                let min_h = p.control_points.iter().copied().fold(f32::MAX, f32::min) + p.height;

                self.control_points.extend_from_slice(&p.control_points);

                GlPlane {
                    height: p.height,
                    max_h,
                    min_h,
                    first_cp_idx: first_cp_idx as u32,
                }
            })
            .collect::<Vec<_>>();

        // Ensure the planes are sorted by height and does not overlap.
        for i in 1..planes.len() {
            assert!(planes[i].min_h >= planes[i - 1].max_h);
        }

        let domain = GlQuadNode {
            children: 0, // leaf
            first_idx: self.planes.len() as u32,
            plane_count: planes.len() as u32,
            _pad: 0,
            position: [-0.5, -0.5],
            size: 1.0,
            _pad2: 0.0,
        };

        let first_node_idx = self.quad_nodes.len();
        let volume_to_world = Mat4::from_transform(position, rotation, size);
        let volume = GlVolume {
            base_node_idx: first_node_idx as u32,
            _pad: [0; 3],
            world_to_volume: volume_to_world.inverse().into_cols_array_2d(),
            volume_to_world: volume_to_world.into_cols_array_2d(),
        };

        self.planes.extend(planes);
        self.quad_nodes.push(domain);
        self.volumes.push(volume);

        self
    }

    pub fn build(self) -> SceneData {
        SceneData {
            control_points: self.control_points,
            planes: self.planes,
            quad_nodes: self.quad_nodes,
            volumes: self.volumes,
        }
    }
}
