//! Editing sessions, reversible commands, and persistence contracts.
//!
//! Callers submit commands against the session's current revision. A rejected or
//! unchanged edit does not consume history or invalidate redo. The document is
//! exposed read-only; desktop integration is not required.
//!
//! ```
//! use ecad_app::{Command, Session};
//! use ecad_domain::{Device, DeviceId, Project, ProjectId};
//! let mut session = Session::new(Project::new(ProjectId::new(1)?));
//! let device = Device::new(DeviceId::new(1)?, "-X1", [])?;
//! session.execute(session.revision(), Command::InsertDevice(device))?;
//! session.undo(session.revision())?;
//! assert_eq!(session.project().devices().count(), 0);
//! session.redo(session.revision())?;
//! assert_eq!(session.project().devices().count(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod command;
mod document;
mod document_error;
mod error;
mod revision;
mod session;
mod transaction;
mod versioned_store;

pub use command::Command;
pub use document::{Document, ReloadPolicy};
pub use document_error::DocumentError;
pub use error::EditError;
pub use revision::{Revision, SessionEpoch};
pub use session::Session;
pub use versioned_store::{ProjectSnapshot, VersionedProjectStore};

/// Low-level, unconditional IO boundary for fixtures/imports and standalone sessions.
/// Use [`VersionedProjectStore`] through [`Document`] for normal editing with
/// external-change checks. This contract does not track observed file versions.
pub trait ProjectStore {
    /// Concrete adapter failure, preserving structured source information.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Load a fully validated project; no partially loaded document escapes.
    /// # Errors
    /// Reports IO, format, schema, or model validation failures.
    fn load(&self) -> Result<ecad_domain::Project, Self::Error>;

    /// Save the complete document, without altering the editing session.
    /// # Errors
    /// Reports IO or encoding failures. Durability guarantees are adapter-specific.
    fn save(&self, project: &ecad_domain::Project) -> Result<(), Self::Error>;
}
