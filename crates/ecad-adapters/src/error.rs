//! Typed format and filesystem failures, retaining their original sources.

use ecad_domain::{IdError, ModelError};
use std::{error::Error, fmt, io, path::PathBuf};

/// A project could not be loaded or saved.
#[derive(Debug)]
pub enum ProjectFileError {
    /// Filesystem operation failed.
    Io(io::Error),
    /// JSON syntax, field types, or allowed fields are invalid.
    Json(serde_json::Error),
    /// A newer or otherwise unsupported format must not be interpreted as v1.
    UnsupportedVersion(u32),
    /// Legacy data needs an explicit migration assigning a persistent project ID.
    MigrationRequired(u32),
    /// The target differs from the observed revision, including a new destination.
    ExternalChange(PathBuf),
    /// An identity does not use the canonical nonzero hexadecimal format.
    Identity(IdError),
    /// The parsed objects violate electrical-model invariants.
    Model(ModelError),
}

impl fmt::Display for ProjectFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "project IO failed: {error}"),
            Self::Json(error) => write!(f, "invalid project JSON: {error}"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported project schema version {version}")
            }
            Self::Identity(error) => error.fmt(f),
            Self::MigrationRequired(version) => {
                write!(f, "project schema {version} requires explicit migration")
            }
            Self::ExternalChange(path) => write!(
                f,
                "project file changed or was not observed: {}",
                path.display()
            ),
            Self::Model(error) => write!(f, "invalid project model: {error}"),
        }
    }
}

impl Error for ProjectFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Identity(error) => Some(error),
            Self::Model(error) => Some(error),
            Self::UnsupportedVersion(_) | Self::MigrationRequired(_) | Self::ExternalChange(_) => {
                None
            }
        }
    }
}

impl From<io::Error> for ProjectFileError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<serde_json::Error> for ProjectFileError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
impl From<IdError> for ProjectFileError {
    fn from(error: IdError) -> Self {
        Self::Identity(error)
    }
}
impl From<ModelError> for ProjectFileError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}
