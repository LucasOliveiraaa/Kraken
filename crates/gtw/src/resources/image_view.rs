use contract::resources::{ImageViewId};

use crate::resources::ResourcesGateway;

create_handle_wrapper!(ImageView, ResourcesGateway, ImageViewId, destroy_image_view);