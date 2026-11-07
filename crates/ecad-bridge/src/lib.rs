//! Desktop transport/projection boundary. The Rust document remains authoritative.
//! Tauri and the local browser harness both use this crate; the core depends on neither.
mod controller;
mod dto;
mod movement;
mod projection;
mod properties;
pub use controller::DesktopBridge;
pub use dto::{BridgeError, Request, Response};
