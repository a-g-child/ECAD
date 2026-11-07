//! Desktop transport/projection boundary. The Rust document remains authoritative.
//! Tauri and the local browser harness both use this crate; the core depends on neither.
mod annotation;
mod controller;
mod defaults;
mod dto;
mod layout_tool;
mod movement;
mod path;
mod placement;
mod preview_scene;
mod projection;
mod properties;
pub use controller::DesktopBridge;
pub use dto::{BridgeError, Request, Response};
