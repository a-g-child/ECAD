//! Multi-project lifecycle failures preserve every unrelated editing session.
use crate::{ActionDispatchError, DocumentError, EditError, WorkspaceProjectId};
use std::{error::Error, fmt};
/// A project-targeted workspace operation could not complete.
#[derive(Debug)]
pub enum WorkspaceError<E> {
    /// A closed or foreign runtime project handle was supplied.
    UnknownProject(WorkspaceProjectId),
    /// A host-supplied normalized store location was empty.
    EmptyLocation,
    /// Creation/save-as would target a location already reserved by an open project.
    LocationInUse(String),
    /// Closing would discard project edits without an explicit discard policy.
    UnsavedChanges(WorkspaceProjectId),
    /// A project revision or command failed.
    Edit(EditError),
    /// Project loading/saving failed.
    Store(E),
    /// Reload failed without replacing the project.
    Reload(DocumentError<E>),
    /// Central action validation/dispatch failed.
    Action(ActionDispatchError<E>),
}
impl<E: fmt::Display> fmt::Display for WorkspaceError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownProject(id) => write!(f, "Open project {} is unavailable", id.get()),
            Self::EmptyLocation => f.write_str("A normalized workspace location is required"),
            Self::LocationInUse(location) => {
                write!(f, "Workspace location is already open: {location}")
            }
            Self::UnsavedChanges(id) => write!(
                f,
                "Save or explicitly discard changes before closing project {}",
                id.get()
            ),
            Self::Edit(e) => e.fmt(f),
            Self::Store(e) => e.fmt(f),
            Self::Reload(e) => e.fmt(f),
            Self::Action(e) => e.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for WorkspaceError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(e) => Some(e),
            Self::Store(e) => Some(e),
            Self::Reload(e) => Some(e),
            Self::Action(e) => Some(e),
            _ => None,
        }
    }
}
