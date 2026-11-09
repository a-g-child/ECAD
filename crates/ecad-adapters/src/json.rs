//! Canonical JSON encoding and same-directory temporary-file replacement.

use crate::schema_documents::{DocumentProjectData, TemplatedDocumentData};
use crate::schema_metadata::MetadataProjectData;
use crate::schema_project_templates::CurrentProjectData;
use crate::schema_reuse::ReusableProjectData;
use crate::schema_sheets::SheetProjectData;
use crate::schema_styled_project::{RoutedProjectData, StyledProjectData};
use crate::{
    ProjectFileError,
    schema::{FlatProjectData, LegacyProjectData, ProjectData, VersionEnvelope},
};
use ecad_app::ProjectStore;
use ecad_domain::{Project, ProjectId};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// Encode v13 (v14 with PNG pictures or v15 with JPEG/SVG pictures) JSON in identity order, with two-space indentation and a final LF.
///
/// No timestamps, session history, or platform line endings are embedded.
/// # Errors
/// Returns a structured model or JSON encoding error without changing the project.
pub fn encode_project(project: &Project) -> Result<String, ProjectFileError> {
    project.validate()?;
    Ok(serde_json::to_string_pretty(
        &crate::schema_template_assignments::AssignedProjectData::from(project),
    )? + "\n")
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
    if envelope.schema_version == 1 {
        return Err(ProjectFileError::MigrationRequired(1));
    }
    if envelope.schema_version == 2 {
        return serde_json::from_str::<FlatProjectData>(text)?.upgrade();
    }
    if envelope.schema_version == 3 {
        return Project::try_from(serde_json::from_str::<ProjectData>(text)?);
    }
    if [13, 14, 15].contains(&envelope.schema_version) {
        return Project::try_from(serde_json::from_str::<
            crate::schema_template_assignments::AssignedProjectData,
        >(text)?);
    }
    if envelope.schema_version == 12 {
        return Project::try_from(serde_json::from_str::<
            crate::schema_template_bundles::BundledProjectData,
        >(text)?);
    }
    if envelope.schema_version == 11 {
        return Project::try_from(serde_json::from_str::<CurrentProjectData>(text)?);
    }
    if envelope.schema_version == 10 {
        return Project::try_from(serde_json::from_str::<
            DocumentProjectData<TemplatedDocumentData>,
        >(text)?);
    }
    if envelope.schema_version == 9 {
        return Project::try_from(serde_json::from_str::<DocumentProjectData>(text)?);
    }
    if envelope.schema_version == 8 {
        return Project::try_from(serde_json::from_str::<SheetProjectData>(text)?);
    }
    if envelope.schema_version == 7 {
        return Project::try_from(serde_json::from_str::<MetadataProjectData>(text)?);
    }
    if envelope.schema_version == 6 {
        return Project::try_from(serde_json::from_str::<RoutedProjectData>(text)?);
    }
    if envelope.schema_version == 5 {
        return Project::try_from(serde_json::from_str::<StyledProjectData>(text)?);
    }
    if envelope.schema_version != 4 {
        return Err(ProjectFileError::UnsupportedVersion(
            envelope.schema_version,
        ));
    }
    let data: ReusableProjectData = serde_json::from_str(text)?;
    Project::try_from(data)
}

/// Explicitly migrate a v1 project, preserving all object IDs and assigning its lineage ID.
///
/// This only converts in memory and never rewrites the source. Save the result to
/// a new destination, then use the current format normally. The caller selects the ID.
/// # Errors
/// Rejects unsupported versions, unknown fields, or invalid electrical references.
pub fn migrate_v1(text: &str, project_id: ProjectId) -> Result<Project, ProjectFileError> {
    let envelope: VersionEnvelope = serde_json::from_str(text)?;
    if envelope.schema_version != 1 {
        return Err(ProjectFileError::UnsupportedVersion(
            envelope.schema_version,
        ));
    }
    let legacy: LegacyProjectData = serde_json::from_str(text)?;
    legacy.migrate(project_id)
}

/// A single-file project store, independent of editing history.
///
/// Saves encode first, write a temporary file beside the destination, synchronize
/// its contents, then replace the destination. This avoids truncating the existing
/// file before a complete replacement is available. Parent-directory durability,
/// external-change detection, permission preservation, multi-file transactions,
/// and hostile/concurrent filesystem writers are not supported by this low-level API.
/// Use the versioned store contract through `ecad_app::Document` for normal editing.
#[derive(Clone, Debug)]
pub struct JsonFile {
    path: PathBuf,
    saved_template_loading: bool,
}

impl JsonFile {
    /// Target an explicit path; the parent directory must already exist.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            saved_template_loading: false,
        }
    }

    /// Convert legacy frame assignments in memory on load before a desktop clean checkpoint.
    /// The source bytes/version remain observed exactly; only an explicit save writes v13.
    pub fn with_saved_templates(mut self) -> Self {
        self.saved_template_loading = true;
        self
    }
    pub(crate) fn decode(&self, text: &str) -> Result<Project, ProjectFileError> {
        let project = decode_project(text)?;
        if self.saved_template_loading {
            crate::migrate_template_assignments(&project)
        } else {
            Ok(project)
        }
    }
    /// Return the configured project path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn stage(&self, text: &str) -> Result<tempfile::NamedTempFile, ProjectFileError> {
        let parent = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut staged = tempfile::NamedTempFile::new_in(parent)?;
        staged.write_all(text.as_bytes())?;
        staged.as_file().sync_all()?;
        Ok(staged)
    }
}

impl ProjectStore for JsonFile {
    type Error = ProjectFileError;

    fn load(&self) -> Result<Project, Self::Error> {
        self.decode(&fs::read_to_string(&self.path)?)
    }

    fn save(&self, project: &Project) -> Result<(), Self::Error> {
        let text = encode_project(project)?;
        let staged = self.stage(&text)?;
        staged
            .persist(&self.path)
            .map_err(|error| ProjectFileError::Io(error.error))?;
        Ok(())
    }
}
