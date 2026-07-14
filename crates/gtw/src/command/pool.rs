use contract::command::CommandPoolId;

use crate::command::CommandGateway;

pub use contract::command::{CommandPoolDesc, CommandPoolResetMode, CommandBufferLifetime};

create_handle_wrapper!(CommandPool, CommandGateway, CommandPoolId, destroy_command_pool);