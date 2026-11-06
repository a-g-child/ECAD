//! Canonical JSON encoding and same-directory temporary-file replacement.

use crate::{
    ProjectFileError,
    schema::{ProjectData, VersionEnvelope},
};
use ecad_app::ProjectStore;
use ecad_domain::Project;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// Encode v1 JSON in identity order, with two-space indentation and a final LF.
///
/// No timestamps, session history, or platform line endings are embedded.
/// # Errors
/// Returns a structured model or JSON encoding error without changing the project.
pub fn encode_project(project: &Project) -> Result<String, ProjectFileError> {
    project.validate()?;
    Ok(serde_json::to_string_pretty(&ProjectData::from(project))? + "\n")
}

/// Decode and validate a complete project without exposing intermediate objects.
///
/// Free conductor ends are valid. Unknown fields are rejected so loading and
/// resaving cannot silently discard data that this implementation does not know.
/// # Errors
/// Rejects invalid syntax, unsupported versions, unknown fields, duplicate objects,
/// malformed identities, or dangling terminal references.
pub fn decode_project(text: &str) -> Result<Project, ProjectFileError> {
    let envelope: VersionEnvelope = serde_json::from_str(text)?;
    if envelope.schema_version != 1 {
        return Err(ProjectFileError::UnsupportedVersion(
            envelope.schema_version,
        ));
    }
    let data: ProjectData = serde_json::from_str(text)?;
    Project::try_from(data)
}

/// A single-file project store, independent of editing history.
///
/// Saves encode first, write a temporary file beside the destination, synchronize
/// its contents, then replace the destination. This avoids truncating the existing
/// file before a complete replacement is available. Parent-directory durability,
/// external-change detection, permission preservation, multi-file transactions,
/// and hostile/concurrent filesystem writers are not supported by this prototype.
#[derive(Clone, Debug)]
pub struct JsonFile {
    path: PathBuf,
}

impl JsonFile {
    /// Target an explicit path; the parent directory must already exist.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Return the configured project path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ProjectStore for JsonFile {
    type Error = ProjectFileError;

    fn load(&self) -> Result<Project, Self::Error> {
        decode_project(&fs::read_to_string(&self.path)?)
    }

    fn save(&self, project: &Project) -> Result<(), Self::Error> {
        let text = encode_project(project)?;
        let parent = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut staged = tempfile::NamedTempFile::new_in(parent)?;
        staged.write_all(text.as_bytes())?;
        staged.as_file().sync_all()?;
        staged
            .persist(&self.path)
            .map_err(|error| ProjectFileError::Io(error.error))?;
        Ok(())
    }
}
