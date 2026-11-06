//! Lifecycle failures preserve the current session and its storage checkpoint.

use crate::EditError;
use ecad_domain::ProjectId;
use std::{error::Error, fmt};

/// Loading/replacing a document could not complete safely.
#[derive(Debug)]
pub enum DocumentError<E> {
    /// A command references a stale or unrelated session.
    Edit(EditError),
    /// Storage failed or the candidate file is invalid.
    Store(E),
    /// Reload would discard edits without an explicit discard policy.
    UnsavedChanges,
    /// The candidate belongs to another lineage; open it as a separate document.
    DifferentProject {
        /// Identity currently open.
        expected: ProjectId,
        /// Identity found in the replacement.
        actual: ProjectId,
    },
}

impl<E: fmt::Display> fmt::Display for DocumentError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => error.fmt(f),
            Self::Store(error) => error.fmt(f),
            Self::UnsavedChanges => {
                f.write_str("reload requires saving or explicitly discarding local changes")
            }
            Self::DifferentProject { expected, actual } => write!(
                f,
                "replacement project {actual} differs from open project {expected}"
            ),
        }
    }
}

impl<E: Error + 'static> Error for DocumentError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Store(error) => Some(error),
            _ => None,
        }
    }
}
