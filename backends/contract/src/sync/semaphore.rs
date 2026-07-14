#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SemaphoreId(pub u32);
impl std::fmt::Display for SemaphoreId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SemaphoreId({})", self.0)
    }
}