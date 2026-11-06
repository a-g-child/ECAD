//! Replaceable project IO, separate from the editing model and session.
//!
//! The initial adapter saves a complete experimental v1 project in one JSON file.
//! Domain structs are not serialized directly. See [`JsonFile`] for save guarantees
//! and [`encode_project`] for deterministic text suitable for Git.

mod error;
mod json;
mod schema;

pub use error::ProjectFileError;
pub use json::{JsonFile, decode_project, encode_project};
