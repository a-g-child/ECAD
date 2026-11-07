//! Retained profile-file observations and immutable project-copy proposals.
use crate::{dto::*, metadata};
use ecad_adapters::{ProfileFile, ProfileSnapshot, RandomIds};
use ecad_app::*;
use ecad_domain::*;
use serde::Serialize;
use std::{collections::BTreeMap, path::PathBuf};
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProfileRow {
    id: String,
    name: String,
    values: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProfileCatalog {
    token: String,
    entries: Vec<ProfileRow>,
    fields: Vec<metadata::MetadataRow>,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ImportChange {
    key: &'static str,
    label: &'static str,
    before: Option<String>,
    after: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ImportRow {
    token: String,
    changes: Vec<ImportChange>,
}
#[derive(Default)]
pub(crate) struct ProfilePanel {
    file: Option<ProfileFile>,
    observed: Option<ProfileSnapshot>,
    token: Option<String>,
    preview: Option<(String, ProfileImportPreview)>,
}
fn error(e: impl std::fmt::Display) -> BridgeError {
    BridgeError::new("profiles", e)
}
fn identity() -> Result<String, BridgeError> {
    RandomIds::profile().map(|id| id.to_string()).map_err(error)
}
impl ProfilePanel {
    pub fn configure(&mut self, path: PathBuf) {
        *self = Self {
            file: Some(ProfileFile::new(path)),
            ..Self::default()
        };
    }
    pub fn invalidate(&mut self) {
        self.preview = None;
    }
    fn observed(&self, token: &str) -> Result<&ProfileSnapshot, BridgeError> {
        if self.token.as_deref() != Some(token) {
            return Err(error(
                "Profiles were refreshed; reload before saving or previewing",
            ));
        }
        self.observed
            .as_ref()
            .ok_or_else(|| error("Load profiles first"))
    }
    pub fn load(&mut self) -> Result<Response, BridgeError> {
        self.preview = None;
        // A failed refresh must not retain authorization to overwrite unreadable bytes.
        self.observed = None;
        self.token = None;
        let file = self
            .file
            .as_ref()
            .ok_or_else(|| error("The host has not configured profile storage"))?;
        let observed = file.load().map_err(error)?;
        self.token = Some(identity()?);
        self.observed = Some(observed);
        self.response()
    }
    pub fn save(
        &mut self,
        token: &str,
        id: Option<String>,
        name: String,
        values: BTreeMap<String, String>,
    ) -> Result<Response, BridgeError> {
        self.preview = None;
        let observed = self.observed(token)?;
        for key in values.keys() {
            if !PROFILE_FIELDS.iter().any(|f| f.key() == key) {
                return Err(error(format!("{key} is not a company/contact field")));
            }
        }
        let id = match id {
            Some(id) => {
                let id = id.parse::<ProfileId>().map_err(error)?;
                if !observed.profiles.contains_key(&id) {
                    return Err(error("Profile no longer exists"));
                }
                id
            }
            None => {
                let id = RandomIds::profile().map_err(error)?;
                if observed.profiles.contains_key(&id) {
                    return Err(error(
                        "Generated profile identity already exists; retry creation",
                    ));
                }
                id
            }
        };
        let profile = CompanyProfile::new(id, name, metadata::parse(values)?).map_err(error)?;
        let mut profiles = observed.profiles.clone();
        profiles.insert(id, profile);
        let next = identity()?;
        let version = self
            .file
            .as_ref()
            .expect("observed configured file")
            .save(&profiles, observed.version.as_ref())
            .map_err(error)?;
        self.observed = Some(ProfileSnapshot {
            profiles,
            version: Some(version),
        });
        self.token = Some(next);
        let mut response = self.response()?;
        response.message = Some("Profile saved · project unchanged".into());
        Ok(response)
    }
    pub fn prepare(
        &mut self,
        session: &Session,
        token: &str,
        id: &str,
        fields: Vec<String>,
    ) -> Result<Response, BridgeError> {
        self.preview = None;
        let observed = self.observed(token)?;
        let id = id.parse::<ProfileId>().map_err(error)?;
        let profile = observed
            .profiles
            .get(&id)
            .ok_or_else(|| error("Profile no longer exists"))?;
        let fields = fields
            .into_iter()
            .map(|key| MetadataField::from_key(&key).ok_or_else(|| error("Unknown import field")))
            .collect::<Result<_, _>>()?;
        let preview = preview_profile_import(session, profile, fields).map_err(error)?;
        let token = identity()?;
        let row = ImportRow {
            token: token.clone(),
            changes: preview
                .changes()
                .iter()
                .map(|c| ImportChange {
                    key: c.field.key(),
                    label: c.field.label(),
                    before: c.before.clone(),
                    after: c.after.clone(),
                })
                .collect(),
        };
        self.preview = Some((token, preview));
        Ok(Response {
            profile_import: Some(row),
            ..Response::default()
        })
    }
    pub fn take(&mut self, token: &str) -> Result<ProfileImportPreview, BridgeError> {
        if self.preview.as_ref().map(|p| p.0.as_str()) != Some(token) {
            return Err(error("Import preview was replaced or invalidated"));
        }
        Ok(self.preview.take().expect("checked preview").1)
    }
    fn response(&self) -> Result<Response, BridgeError> {
        let token = self
            .token
            .clone()
            .ok_or_else(|| error("Load profiles first"))?;
        let observed = self.observed(&token)?;
        let entries = observed
            .profiles
            .values()
            .map(|p| ProfileRow {
                id: p.id().to_string(),
                name: p.name().into(),
                values: p
                    .values()
                    .values()
                    .iter()
                    .map(|(field, value)| (field.key().into(), value.clone()))
                    .collect(),
            })
            .collect();
        Ok(Response {
            profiles: Some(ProfileCatalog {
                token,
                entries,
                fields: metadata::selected(&ProjectMetadata::default(), PROFILE_FIELDS),
            }),
            ..Response::default()
        })
    }
}
