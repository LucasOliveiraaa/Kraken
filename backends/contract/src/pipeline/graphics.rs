#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GraphicsPipelineId(pub u32);
impl std::fmt::Display for GraphicsPipelineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GraphicsPipelineId({})", self.0)
    }
}
