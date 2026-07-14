use contract::resources::ImageId;

use crate::resources::ResourcesGateway;

create_handle_wrapper!(Image, ResourcesGateway, ImageId, destroy_image);