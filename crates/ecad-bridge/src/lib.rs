//! Desktop transport/projection boundary. The Rust document remains authoritative.
//! Tauri and the local browser harness both use this crate; the core depends on neither.
mod annotation;
mod controller;
mod defaults;
mod document_rows;
mod drawing_list_rows;
mod dto;
mod frame_rows;
mod layout_tool;
mod metadata;
mod movement;
mod path;
mod placement;
mod preview_scene;
mod profiles;
mod project_templates;
mod projection;
mod properties;
mod sheet_creation;
mod sheets;
pub use controller::DesktopBridge;
pub use dto::{BridgeError, Request, Response};

mod template_rows;
mod template_table_edit;
mod template_table_preview;
mod template_workspace;

mod captured_context;
