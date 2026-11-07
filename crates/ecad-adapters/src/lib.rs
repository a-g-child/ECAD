//! Replaceable project IO, separate from the editing model and session.
//!
//! The adapter saves a complete experimental v3 project in one JSON file.
//! Domain structs are not serialized directly. See [`JsonFile`] for save guarantees
//! and [`encode_project`] for deterministic text suitable for Git.

mod error;
mod identities;
mod json;
mod schema;
mod schema_structure;
mod versioned_file;

pub use error::ProjectFileError;
pub use identities::RandomIds;
pub use json::{JsonFile, decode_project, encode_project, migrate_v1};
pub use versioned_file::FileVersion;
