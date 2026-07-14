#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeviceId(pub u32);
impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DevicePropsId(pub u32);
impl std::fmt::Display for DevicePropsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeviceType {
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Cpu,
    Other,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DeviceFeatures: u64 {
        const ROBUST_BUFFER_ACCESS = 1 << 0;
        const FULL_DRAW_INDEX_UINT32 = 1 << 1;
        const IMAGE_CUBE_ARRAY = 1 << 2;
        const INDEPENDENT_BLEND = 1 << 3;
        const GEOMETRY_SHADER = 1 << 4;
        const TESSELLATION_SHADER = 1 << 5;
        const SAMPLE_RATE_SHADING = 1 << 6;
        const DUAL_SRC_BLEND = 1 << 7;
        const LOGIC_OP = 1 << 8;
        const MULTI_DRAW_INDIRECT = 1 << 9;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MemoryHeapFlags: u32 {
        const DEVICE_LOCAL = 1 << 0;
        const MULTI_INSTANCE = 1 << 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryHeap {
    pub size: u64,
    pub flags: MemoryHeapFlags,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceProps {
    pub id: DevicePropsId,
    pub name: Option<String>,
    pub vendor_id: u32,
    pub device_type: DeviceType,
    pub features: DeviceFeatures,
    pub memory_heaps: Vec<MemoryHeap>,
}

pub struct DeviceCreationDesc {
    pub id: DevicePropsId,
    pub pipeline_cache_data: Option<Vec<u8>>,
}
