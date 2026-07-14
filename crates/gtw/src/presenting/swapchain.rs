use contract::{GpuResult, presenting::{SwapchainId, SwapchainImageId}};

use crate::{
    presenting::PresentingGateway,
    sync::{Fence, Semaphore},
};

create_handle_wrapper!(SwapchainImage, PresentingGateway, SwapchainImageId);

create_handle_wrapper!(Swapchain, PresentingGateway, SwapchainId, destroy_swapchain);

pub struct SwapchainAcquiredImage {
    pub image: SwapchainImage,
    pub suboptimal: bool,
}

impl Swapchain {
    pub fn acquire_next_image(
        &self,
        semaphore: Option<Semaphore>,
        fence: Option<Fence>,
    ) -> contract::GpuResult<SwapchainAcquiredImage> {
        let result = self.raw_gtw().acquire_next_image(
            self.handle(),
            semaphore.map(|s| s.handle()),
            fence.map(|f| f.handle()),
        )?;

        Ok(SwapchainAcquiredImage {
            image: SwapchainImage::from_handle(self.gtw().clone(), result.image_id),
            suboptimal: result.suboptimal,
        })
    }

    pub fn present_image(
        &self,
        image: SwapchainImage,
        wait_semaphores: Vec<Semaphore>,
    ) -> GpuResult<bool> {
        let wait_semaphore_ids = wait_semaphores
            .iter()
            .map(|s| s.handle())
            .collect::<Vec<_>>();

        self.raw_gtw()
            .present_image(self.handle(), image.handle(), wait_semaphore_ids)
    }
}
