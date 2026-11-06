//! Content-based change detection and optimistic single-file replacement.

use crate::{JsonFile, ProjectFileError, decode_project, encode_project};
use ecad_app::{ProjectSnapshot, VersionedProjectStore};
use ecad_domain::Project;
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Observed canonical-parent location and SHA-256 digest of exact file bytes.
///
/// Location is part of equality: a token from another path cannot authorize a save
/// merely because both files contain equal data. This is not a persistent model ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileVersion {
    location: PathBuf,
    digest: [u8; 32],
}

impl JsonFile {
    fn version_for(&self, bytes: &[u8]) -> Result<FileVersion, ProjectFileError> {
        let name = self.path().file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "project path needs a file name",
            )
        })?;
        let parent = self
            .path()
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        Ok(FileVersion {
            location: parent.canonicalize()?.join(name),
            digest: Sha256::digest(bytes).into(),
        })
    }
}

/// Optimistic checks detect edits, deletions, and newly created destinations.
///
/// The content check occurs immediately before replacement. An uncooperative
/// writer can still race between check and rename: this is not an atomic
/// compare-and-swap or a filesystem lock. Parent-directory durability, aliases,
/// symlink replacement, and metadata preservation need further policy.
impl VersionedProjectStore for JsonFile {
    type Version = FileVersion;
    type Error = ProjectFileError;

    fn load_snapshot(&self) -> Result<ProjectSnapshot<FileVersion>, Self::Error> {
        let bytes = fs::read(self.path())?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let project = decode_project(text)?;
        let version = self.version_for(&bytes)?;
        Ok(ProjectSnapshot { project, version })
    }

    fn current_version(&self) -> Result<Option<FileVersion>, Self::Error> {
        match fs::read(self.path()) {
            Ok(bytes) => Ok(Some(self.version_for(&bytes)?)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn save_if_unchanged(
        &self,
        project: &Project,
        expected: Option<&FileVersion>,
    ) -> Result<FileVersion, Self::Error> {
        let text = encode_project(project)?;
        let staged = self.stage(&text)?;
        let version = self.version_for(text.as_bytes())?;
        if self.current_version()?.as_ref() != expected {
            return Err(ProjectFileError::ExternalChange(self.path().to_owned()));
        }
        if expected.is_some() {
            staged
                .persist(self.path())
                .map_err(|error| ProjectFileError::Io(error.error))?;
        } else {
            staged.persist_noclobber(self.path()).map_err(|error| {
                if error.error.kind() == io::ErrorKind::AlreadyExists {
                    ProjectFileError::ExternalChange(self.path().to_owned())
                } else {
                    ProjectFileError::Io(error.error)
                }
            })?;
        }
        Ok(version)
    }
}
