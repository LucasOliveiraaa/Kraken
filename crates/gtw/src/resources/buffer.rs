use bytemuck::{Pod, bytes_of, bytes_of_mut, cast_slice, cast_slice_mut};
use contract::resources::BufferId;

use crate::resources::ResourcesGateway;

create_handle_wrapper!(Buffer, ResourcesGateway, BufferId, destroy_buffer);

impl Buffer {
    pub fn write<T: Pod>(&self, offset: u64, data: &T) -> contract::GpuResult<()> {
        self.raw_gtw()
            .write_buffer(self.handle(), offset, bytes_of(data))
    }

    pub fn write_slice<T: Pod>(&self, offset: u64, data: &[T]) -> contract::GpuResult<()> {
        self.raw_gtw()
            .write_buffer(self.handle(), offset, cast_slice(data))
    }

    pub fn read<T: Pod>(&self, offset: u64, data: &mut T) -> contract::GpuResult<()> {
        self.raw_gtw()
            .read_buffer(self.handle(), offset, bytes_of_mut(data))
    }

    pub fn read_slice<T: Pod>(&self, offset: u64, data: &mut [T]) -> contract::GpuResult<()> {
        self.raw_gtw()
            .read_buffer(self.handle(), offset, cast_slice_mut(data))
    }
}
