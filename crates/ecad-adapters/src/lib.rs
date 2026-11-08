//! Replaceable project IO, separate from the editing model and session.
//!
//! The adapter saves a complete experimental v9 project in one JSON file.
//! Domain structs are not serialized directly. See [`JsonFile`] for save guarantees
//! and [`encode_project`] for deterministic text suitable for Git.

mod connection_csv;
mod error;
mod identities;
mod instance_ids;
mod json;
mod keymap_json;
mod known_projects;
mod library_error;
mod library_file;
mod library_json;
mod profile_file;
mod schema;
mod schema_definition;
mod schema_documents;
mod schema_drawing;
mod schema_drawing_values;
mod schema_metadata;
mod schema_reuse;
mod schema_routes;
mod schema_sheets;
mod schema_structure;
mod schema_styled_project;
mod versioned_file;

pub use error::ProjectFileError;
pub use identities::RandomIds;
pub use instance_ids::InstanceIdError;
pub use json::{JsonFile, decode_project, encode_project, migrate_v1};
pub use keymap_json::{KeymapFileError, decode_keymap, encode_keymap};
pub use known_projects::{KnownProjects, UsageScanFailure, UsageScanReport};
pub use library_error::LibraryFileError;
pub use library_file::LibraryFile;
pub use library_json::{decode_library, encode_library};
pub use versioned_file::FileVersion;

pub use connection_csv::{ConnectionCsvKind, encode_connection_csv, encode_current_connection_csv};

pub use profile_file::{ProfileFile, ProfileFileError, ProfileSnapshot};
