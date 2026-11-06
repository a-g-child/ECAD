//! Editing sessions, reversible commands, and persistence contracts.
//!
//! Callers submit commands against the session's current revision. A rejected or
//! unchanged edit does not consume history or invalidate redo. The document is
//! exposed read-only; desktop integration is not required.
//!
//! ```
//! use ecad_app::{Command, Session};
//! use ecad_domain::{Device, DeviceId, Project};
//! let mut session = Session::new(Project::default());
//! let device = Device::new(DeviceId::new(1)?, "-X1", [])?;
//! session.execute(session.revision(), Command::InsertDevice(device))?;
//! session.undo(session.revision())?;
//! assert_eq!(session.project().devices().count(), 0);
//! session.redo(session.revision())?;
//! assert_eq!(session.project().devices().count(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod command;
mod error;
mod revision;
mod session;
mod transaction;

pub use command::Command;
pub use error::EditError;
pub use revision::{Revision, SessionEpoch};
pub use session::Session;

/// IO boundary implemented by project storage adapters, not by the domain.
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
