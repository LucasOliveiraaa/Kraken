use crate::command::QueueType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommandPoolId(pub u32);
impl std::fmt::Display for CommandPoolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CommandPoolId({})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CommandPoolResetMode {
    ResetIndividually,
    ResetAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CommandBufferLifetime {
    OneTimeSubmit,
    Reusable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandPoolDesc {
    pub queue_type: QueueType,
    pub reset_mode: CommandPoolResetMode,
    pub buffer_lifetime: CommandBufferLifetime,
}