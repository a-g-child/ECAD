//! Typed template catalogue/publication failures, preserving original sources.
use crate::{ProjectFileError, TemplateFileError};
use ecad_domain::{TemplateLibraryError, TemplateRevisionRef};
use std::{error::Error, fmt, io, path::PathBuf};
/// A complete template-library load/save failed; successful publication is not implied.
#[derive(Debug)]
pub enum TemplateLibraryFileError {
    /// Filesystem failure.
    Io(io::Error),
    /// Strict JSON, identity, binding or layout validation failed.
    Format(TemplateFileError),
    /// Revision/catalogue invariants failed.
    Catalog(TemplateLibraryError),
    /// Shared staging/version-token infrastructure failed.
    Storage(ProjectFileError),
    /// Exact destination/content differs from the observed version.
    ExternalChange(PathBuf),
    /// Another cooperating writer or retained lock blocks publication.
    Busy(PathBuf),
    /// The replacement would remove/rewrite an already published complete revision.
    HistoryRewrite(TemplateRevisionRef),
}
impl fmt::Display for TemplateLibraryFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Template library IO failed: {error}"),
            Self::Format(error) => error.fmt(f),
            Self::Catalog(error) => error.fmt(f),
            Self::Storage(error) => error.fmt(f),
            Self::ExternalChange(path) => write!(
                f,
                "Template library changed or was not observed: {}",
                path.display()
            ),
            Self::Busy(path) => {
                write!(f, "Template library writer lock exists: {}", path.display())
            }
            Self::HistoryRewrite(pin) => write!(
                f,
                "Cannot remove or rewrite published template revision {pin:?}"
            ),
        }
    }
}
impl Error for TemplateLibraryFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
            Self::Catalog(error) => Some(error),
            Self::Storage(error) => Some(error),
            _ => None,
        }
    }
}
impl From<io::Error> for TemplateLibraryFileError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<ProjectFileError> for TemplateLibraryFileError {
    fn from(error: ProjectFileError) -> Self {
        Self::Storage(error)
    }
}
impl From<TemplateFileError> for TemplateLibraryFileError {
    fn from(error: TemplateFileError) -> Self {
        Self::Format(error)
    }
}
impl From<TemplateLibraryError> for TemplateLibraryFileError {
    fn from(error: TemplateLibraryError) -> Self {
        Self::Catalog(error)
    }
}
impl From<serde_json::Error> for TemplateLibraryFileError {
    fn from(error: serde_json::Error) -> Self {
        Self::Format(error.into())
    }
}
impl From<ecad_domain::IdError> for TemplateLibraryFileError {
    fn from(error: ecad_domain::IdError) -> Self {
        Self::Format(error.into())
    }
}
