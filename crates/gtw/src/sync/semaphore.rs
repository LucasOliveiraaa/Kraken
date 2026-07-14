use contract::sync::SemaphoreId;

use crate::sync::SyncGateway;

create_handle_wrapper!(Semaphore, SyncGateway, SemaphoreId);