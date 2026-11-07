//! Reusable contact defaults and closed, revision-bound copy proposals. No profile IO here.
use crate::{Revision, Session};
use ecad_domain::{MetadataField, ProfileId, ProjectMetadata};
use std::{collections::BTreeSet, error::Error, fmt};

/// The only metadata fields supplied by company/contact profiles.
pub const PROFILE_FIELDS: [MetadataField; 3] = [
    MetadataField::CompanyName,
    MetadataField::CompanyAddress,
    MetadataField::EngineerName,
];
/// A rejected profile or import selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileError {
    /// A profile needs a nonblank display name.
    BlankName,
    /// Project/sheet/engineering issue data cannot be supplied by a personal profile.
    UnsupportedField(MetadataField),
}
impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankName => f.write_str("Enter a profile name"),
            Self::UnsupportedField(field) => {
                write!(f, "{} is not a company/contact profile field", field.key())
            }
        }
    }
}
impl Error for ProfileError {}
/// A reusable starting-value snapshot, separate from an engineering document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompanyProfile {
    id: ProfileId,
    name: String,
    values: ProjectMetadata,
}
impl CompanyProfile {
    /// Validate a stable identity, literal name and company/contact values.
    /// # Errors
    /// Rejects blank names and project-specific fields.
    pub fn new(id: ProfileId, name: String, values: ProjectMetadata) -> Result<Self, ProfileError> {
        if name.trim().is_empty() {
            return Err(ProfileError::BlankName);
        }
        for field in values.values().keys() {
            if !PROFILE_FIELDS.contains(field) {
                return Err(ProfileError::UnsupportedField(*field));
            }
        }
        Ok(Self { id, name, values })
    }
    /// Stable identity retained when renaming or updating the profile.
    pub fn id(&self) -> ProfileId {
        self.id
    }
    /// Literal profile display name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Values to copy deliberately; never live links into a project.
    pub fn values(&self) -> &ProjectMetadata {
        &self.values
    }
}
/// One changed field in a reviewed import; None means absent, including deliberate clearing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileFieldChange {
    /// Semantic project field identity.
    pub field: MetadataField,
    /// Exact previous value.
    pub before: Option<String>,
    /// Exact proposed value.
    pub after: Option<String>,
}
/// Closed project copy proposal; changing a saved profile never changes this reviewed content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileImportPreview {
    revision: Revision,
    pub(crate) proposed: ProjectMetadata,
    changes: Vec<ProfileFieldChange>,
}
impl ProfileImportPreview {
    /// Session/revision against which the copy was reviewed.
    pub fn revision(&self) -> Revision {
        self.revision
    }
    /// Changed selected fields only, with exact old/new values.
    pub fn changes(&self) -> &[ProfileFieldChange] {
        &self.changes
    }
}
/// Copy only selected company/contact fields into a retained project proposal.
/// Empty profile fields clear project entries only when explicitly selected.
/// # Errors
/// Rejects project/sheet fields; successful preparation does not edit the session.
pub fn preview_profile_import(
    session: &Session,
    profile: &CompanyProfile,
    fields: BTreeSet<MetadataField>,
) -> Result<ProfileImportPreview, ProfileError> {
    let mut values = session.project().metadata().values().clone();
    let mut changes = vec![];
    for field in fields {
        if !PROFILE_FIELDS.contains(&field) {
            return Err(ProfileError::UnsupportedField(field));
        }
        let before = values.get(&field).cloned();
        let after = profile.values.value(field).map(str::to_owned);
        if before != after {
            changes.push(ProfileFieldChange {
                field,
                before,
                after: after.clone(),
            });
            if let Some(value) = after {
                values.insert(field, value);
            } else {
                values.remove(&field);
            }
        }
    }
    Ok(ProfileImportPreview {
        revision: session.revision(),
        proposed: ProjectMetadata::new(values),
        changes,
    })
}
