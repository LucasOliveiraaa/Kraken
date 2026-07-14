//! FFI interface for the contract backend.
//!
//! The Foreign Function Interface (FFI) is designed to allow the backend to be
//! used by the Editor or Renderer using the Contract defined in this crate.
//!
//! # ABI
//! The ABI holds iff both the backend and the Editor/Renderer are compiled with
//! the same compiler version and optimization level. This is a limitation of
//! Rust's ABI, as it is not stable across the compiler version.
//!
//! # Safety
//! The FFI interface is unsafe. The backend must ensure that the proper entrypoint
//! is defined and that the backend is initialized properly. The Editor/Renderer must ensure that:
//!  - The backend is loaded properly and that the entrypoint is called with
//!    the proper arguments.
//!  - The backend is not unloaded while it is still in use.
//!  - The backend must ensure that the Editor/Renderer is not calling any
//!    functions after the backend is unloaded.

use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::{DeviceGateway, GpuResult};

#[derive(Debug, Clone)]
pub struct WindowHandles {
    pub display_handle: RawDisplayHandle,
    pub window_handle: RawWindowHandle,
}

/// The entrypoint for the backend. This function returns a leaked pointer to 
/// the GatewayBuilder, which is used to build the DeviceGateway.
/// 
/// The returned pointer must be reconstructed into a Box and dropped when the
/// backend is unloaded.
pub const GATEWAY_ENTRYPOINT_NAME: &[u8] = b"gateway_entrypoint";

/// The entrypoint for the backend. This function returns a leaked pointer to 
/// the GatewayBuilder, which is used to build the DeviceGateway.
/// 
/// The returned pointer must be reconstructed into a Box and dropped when the
/// backend is unloaded.
#[allow(improper_ctypes_definitions)]
pub type GatewayBuilderEntrypoint = extern "C" fn() -> *mut dyn GatewayBuilder;

pub trait GatewayBuilder {
    fn set_name(&mut self, name: &str) -> &mut dyn GatewayBuilder;
    fn set_version(&mut self, version: u32) -> &mut dyn GatewayBuilder;
    fn set_engine_version(&mut self, engine_version: u32) -> &mut dyn GatewayBuilder;
    fn set_main_window(&mut self, window: &WindowHandles) -> &mut dyn GatewayBuilder;

    /// Consumes the Gateway and returns a reference to the DeviceGateway.
    ///
    /// This msut be the last action taken, as after building the DeviceGateway
    /// with the provided configuration, the Builder is no longer available.
    fn build(self: Box<Self>) -> GpuResult<Box<dyn DeviceGateway>>;
}
