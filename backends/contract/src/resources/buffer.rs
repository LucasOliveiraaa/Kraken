use crate::resources::MemoryLocation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferId(pub u32);
impl std::fmt::Display for BufferId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BufferId({})", self.0)
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
    pub struct BufferUsage: u32 {
        const VERTEX    = 1 << 0;
        const INDEX     = 1 << 1;
        const UNIFORM   = 1 << 2;
        const STORAGE   = 1 << 3;
        const INDIRECT  = 1 << 4;
        const TRANSFER_SRC = 1 << 5;
        const TRANSFER_DST = 1 << 6;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BufferDesc {
    pub name: String,
    pub size: u64,
    pub usage: BufferUsage,
    pub location: MemoryLocation,
}