pub mod alloc;
mod core;
pub mod ffi;
pub mod presenting;
pub mod resources;
pub mod sync;
pub mod pipeline;
pub mod command;

pub use core::*;

use crate::ffi::VkGatewayBuilder;

#[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn gateway_entrypoint() -> *mut dyn contract::ffi::GatewayBuilder {
    let gatway = Box::new(VkGatewayBuilder::new());

    Box::into_raw(gatway) as *mut dyn contract::ffi::GatewayBuilder
}
