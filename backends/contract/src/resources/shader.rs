#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ShaderId(pub u32);
impl std::fmt::Display for ShaderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShaderId({})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}
impl std::fmt::Display for ShaderStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderStage::Vertex => write!(f, "Vertex"),
            ShaderStage::Fragment => write!(f, "Fragment"),
            ShaderStage::Compute => write!(f, "Compute"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShaderPass {
    pub entry_point: String,
    pub stage: ShaderStage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShaderDesc {
    pub spirv: Vec<u32>,
    pub passes: Vec<ShaderPass>,
}
