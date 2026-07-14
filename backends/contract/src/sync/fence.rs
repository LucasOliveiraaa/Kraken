#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FenceId(pub u32);
impl std::fmt::Display for FenceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FenceId({})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FenceState {
    Signaled,
    Unsignaled,
}