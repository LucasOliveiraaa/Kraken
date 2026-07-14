use contract::resources::ShaderId;

use crate::resources::ResourcesGateway;

create_handle_wrapper!(Shader, ResourcesGateway, ShaderId, destroy_shader);