//! Desktop transport/projection boundary. The Rust document remains authoritative.
//! Tauri and the local browser harness both use this crate; the core depends on neither.
mod annotation;
mod controller;
mod defaults;
mod dto;
mod layout_tool;
mod metadata;
mod movement;
mod path;
mod placement;
mod preview_scene;
mod profiles;
mod projection;
mod properties;
mod sheets;
pub use controller::DesktopBridge;
pub use dto::{BridgeError, Request, Response};
