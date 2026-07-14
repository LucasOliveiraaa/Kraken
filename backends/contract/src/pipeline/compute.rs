use crate::resources::ShaderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComputePipelineId(pub u32);
impl std::fmt::Display for ComputePipelineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ComputePipelineId({})", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputePipelineDesc {
    pub shader_id: ShaderId,
}