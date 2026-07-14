use contract::{GpuResult, presenting::SurfaceId, resources::SurfaceFormat};

use crate::presenting::PresentingGateway;

create_handle_wrapper!(Surface, PresentingGateway, SurfaceId, destroy_surface);

impl Surface {
    pub fn query_surface_formats(&self) -> GpuResult<Vec<SurfaceFormat>> {
        self.raw_gtw().query_surface_formats(self.handle())
    }
}