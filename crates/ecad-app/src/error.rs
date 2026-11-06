//! Recoverable failures at the document editing boundary.

use crate::Revision;
use ecad_domain::ModelError;
use std::{error::Error, fmt};

/// An edit could not be committed. The document and both history stacks are intact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditError {
    /// The caller references another session or an outdated read projection.
    StaleRevision {
        /// Token supplied by the caller, including its session epoch.
        expected: Revision,
        /// Token currently owned by the session.
        actual: Revision,
    },
    /// The proposed electrical edit is structurally invalid.
    Model(ModelError),
    /// The monotonic revision counter cannot advance further.
    RevisionExhausted,
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaleRevision { expected, actual } => write!(
                f,
                "stale session/revision {}:{}, current token is {}:{}",
                expected.epoch().get(),
                expected.get(),
                actual.epoch().get(),
                actual.get()
            ),
            Self::Model(error) => error.fmt(f),
            Self::RevisionExhausted => f.write_str("session revision counter exhausted"),
        }
    }
}

impl Error for EditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Model(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ModelError> for EditError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}
