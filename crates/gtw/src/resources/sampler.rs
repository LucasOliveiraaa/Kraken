use contract::resources::SamplerId;

use crate::resources::ResourcesGateway;

create_handle_wrapper!(Sampler, ResourcesGateway, SamplerId, destroy_sampler);