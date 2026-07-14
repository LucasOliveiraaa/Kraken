#[macro_use]
mod macros;

pub mod ffi;
mod core;
pub mod resources;
pub mod sync;
pub mod presenting;
pub mod pipeline;
pub mod command;

pub use core::*;

pub use contract::ffi::WindowHandles;
pub use contract::{GpuError, GpuResult};