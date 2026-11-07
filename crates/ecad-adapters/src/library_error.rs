//! Typed shared-library storage failures with source preservation.

use crate::ProjectFileError;
use ecad_domain::{DefinitionRef, LibraryError};
use std::{error::Error, fmt, io, path::PathBuf};

/// A library load or publication failed; no successful write is claimed.
#[derive(Debug)]
pub enum LibraryFileError {
    /// Filesystem operation failed.
    Io(io::Error),
    /// Strict schema, identity, JSON, or electrical-body validation failed.
    Format(ProjectFileError),
    /// Catalog immutability or recommendation validation failed.
    Catalog(LibraryError),
    /// Destination bytes/location no longer match the observed version.
    ExternalChange(PathBuf),
    /// Another cooperating writer holds the lock, or a stale lock needs review.
    Busy(PathBuf),
    /// A replacement would remove or change already published content.
    HistoryRewrite(DefinitionRef),
}
impl fmt::Display for LibraryFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "library IO failed: {error}"),
            Self::Format(error) => write!(f, "invalid library data: {error}"),
            Self::Catalog(error) => error.fmt(f),
            Self::ExternalChange(path) => {
                write!(f, "library changed or was not observed: {}", path.display())
            }
            Self::Busy(path) => write!(f, "library writer lock exists: {}", path.display()),
            Self::HistoryRewrite(source) => {
                write!(f, "cannot remove or rewrite published revision {source}")
            }
        }
    }
}
impl Error for LibraryFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
            Self::Catalog(error) => Some(error),
            _ => None,
        }
    }
}
impl From<io::Error> for LibraryFileError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<ProjectFileError> for LibraryFileError {
    fn from(error: ProjectFileError) -> Self {
        Self::Format(error)
    }
}
impl From<LibraryError> for LibraryFileError {
    fn from(error: LibraryError) -> Self {
        Self::Catalog(error)
    }
}
