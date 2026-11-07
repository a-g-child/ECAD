//! Per-user profile storage with strict DTOs and optimistic staged replacement.
use crate::{FileVersion, JsonFile, ProjectFileError};
use ecad_app::{CompanyProfile, ProfileError};
use ecad_domain::{MetadataField, ProfileId, ProjectMetadata};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

/// Profile read/write failure; errors never authorize overwriting unknown bytes.
#[derive(Debug)]
pub enum ProfileFileError {
    /// Filesystem or JSON staging/version failure.
    Storage(ProjectFileError),
    /// Invalid strict JSON.
    Json(serde_json::Error),
    /// Unsupported profile format version.
    UnsupportedVersion(u32),
    /// Invalid profile content or repeated identity/name.
    Invalid(String),
}
impl fmt::Display for ProfileFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(e) => e.fmt(f),
            Self::Json(e) => write!(f, "Invalid profile JSON: {e}"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported company-profile schema {v}"),
            Self::Invalid(e) => f.write_str(e),
        }
    }
}
impl Error for ProfileFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}
impl From<ProjectFileError> for ProfileFileError {
    fn from(e: ProjectFileError) -> Self {
        Self::Storage(e)
    }
}
impl From<io::Error> for ProfileFileError {
    fn from(e: io::Error) -> Self {
        Self::Storage(e.into())
    }
}
impl From<serde_json::Error> for ProfileFileError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
impl From<ProfileError> for ProfileFileError {
    fn from(e: ProfileError) -> Self {
        Self::Invalid(e.to_string())
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Values {
    #[serde(rename = "company.name")]
    name: String,
    #[serde(rename = "company.address")]
    address: String,
    #[serde(rename = "engineer.name")]
    engineer: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileData {
    id: String,
    name: String,
    values: Values,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfilesData {
    schema_version: u32,
    profiles: Vec<ProfileData>,
}
/// Exact read snapshot, including an absent destination for safe first creation.
#[derive(Clone, Debug)]
pub struct ProfileSnapshot {
    /// Validated profiles in stable identity order.
    pub profiles: BTreeMap<ProfileId, CompanyProfile>,
    /// Observed bytes/location; None means no file existed.
    pub version: Option<FileVersion>,
}
/// A host-selected per-user file, never a path taken from a project.
/// Saves detect observed-byte conflicts, stage/sync complete JSON and replace it.
/// Uncooperative writers can race between check and rename; this is not a filesystem lock.
#[derive(Clone, Debug)]
pub struct ProfileFile {
    path: PathBuf,
}
impl ProfileFile {
    /// Select a destination; no IO occurs until load/save.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
    /// Inspect the configured profile location.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Read a validated profile set or an empty absent snapshot without creating a file.
    /// # Errors
    /// Rejects malformed, unsupported, duplicate or unreadable content.
    pub fn load(&self) -> Result<ProfileSnapshot, ProfileFileError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Ok(ProfileSnapshot {
                    profiles: BTreeMap::new(),
                    version: None,
                });
            }
            Err(e) => return Err(e.into()),
        };
        let data: ProfilesData = serde_json::from_slice(&bytes)?;
        if data.schema_version != 1 {
            return Err(ProfileFileError::UnsupportedVersion(data.schema_version));
        }
        let mut profiles = BTreeMap::new();
        let mut names = BTreeSet::new();
        for p in data.profiles {
            let id =
                p.id.parse()
                    .map_err(|e: ecad_domain::IdError| ProfileFileError::Invalid(e.to_string()))?;
            let values = ProjectMetadata::new(BTreeMap::from([
                (MetadataField::CompanyName, p.values.name),
                (MetadataField::CompanyAddress, p.values.address),
                (MetadataField::EngineerName, p.values.engineer),
            ]));
            let profile = CompanyProfile::new(id, p.name, values)?;
            if !names.insert(profile.name().to_owned()) || profiles.insert(id, profile).is_some() {
                return Err(ProfileFileError::Invalid(
                    "Duplicate profile identity or name".into(),
                ));
            }
        }
        Ok(ProfileSnapshot {
            profiles,
            version: Some(FileVersion::for_path(&self.path, &bytes)?),
        })
    }
    /// Persist a complete validated set only against its observed destination version.
    /// Parent directories are created on explicit save, not on load or application startup.
    /// # Errors
    /// Invalid content, IO failure or external modification preserves existing bytes.
    pub fn save(
        &self,
        profiles: &BTreeMap<ProfileId, CompanyProfile>,
        expected: Option<&FileVersion>,
    ) -> Result<FileVersion, ProfileFileError> {
        let mut names = BTreeSet::new();
        for (id, p) in profiles {
            if *id != p.id() || !names.insert(p.name()) {
                return Err(ProfileFileError::Invalid(
                    "Duplicate profile name or mismatched identity".into(),
                ));
            }
        }
        let data = ProfilesData {
            schema_version: 1,
            profiles: profiles
                .values()
                .map(|p| {
                    let value = |field| p.values().value(field).unwrap_or_default().to_owned();
                    ProfileData {
                        id: p.id().to_string(),
                        name: p.name().into(),
                        values: Values {
                            name: value(MetadataField::CompanyName),
                            address: value(MetadataField::CompanyAddress),
                            engineer: value(MetadataField::EngineerName),
                        },
                    }
                })
                .collect(),
        };
        let text = serde_json::to_string_pretty(&data)? + "\n";
        let parent = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let staged = JsonFile::new(&self.path).stage(&text)?;
        let current = match fs::read(&self.path) {
            Ok(bytes) => Some(FileVersion::for_path(&self.path, &bytes)?),
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        if current.as_ref() != expected {
            return Err(ProjectFileError::ExternalChange(self.path.clone()).into());
        }
        let version = FileVersion::for_path(&self.path, text.as_bytes())?;
        if expected.is_some() {
            staged
                .persist(&self.path)
                .map_err(|e| ProfileFileError::from(e.error))?;
        } else {
            staged
                .persist_noclobber(&self.path)
                .map_err(|e| ProfileFileError::from(e.error))?;
        }
        Ok(version)
    }
}
