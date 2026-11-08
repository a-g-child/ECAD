//! Canonical closed template-library schema, independent of project persistence.
use crate::{TemplateFileError, TemplateLibraryFileError, template_json::VariantData};
use ecad_domain::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::Arc};
#[derive(Serialize, Deserialize)]
enum Kind {
    #[serde(rename = "ecad-template-library")]
    TemplateLibrary,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LibraryData {
    kind: Kind,
    schema_version: u32,
    revisions: Vec<RevisionData>,
    recommended: Vec<PinData>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PinData {
    definition: String,
    revision: String,
}
impl From<TemplateRevisionRef> for PinData {
    fn from(pin: TemplateRevisionRef) -> Self {
        Self {
            definition: pin.definition.to_string(),
            revision: pin.revision.to_string(),
        }
    }
}
impl TryFrom<PinData> for TemplateRevisionRef {
    type Error = TemplateLibraryFileError;
    fn try_from(pin: PinData) -> Result<Self, Self::Error> {
        Ok(Self {
            definition: pin.definition.parse()?,
            revision: pin.revision.parse()?,
        })
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RevisionData {
    definition: String,
    revision: String,
    variants: Vec<VariantData>,
}
/// Encode complete revisions and recommendations in stable identity order with a final LF.
/// Nested variant bodies use their own strict v1 schema; no drafts or project values enter it.
/// # Errors
/// Returns serialization failures without changing the catalogue or writing files.
pub fn encode_template_library(
    catalog: &TemplateLibrary,
) -> Result<String, TemplateLibraryFileError> {
    let data = LibraryData {
        kind: Kind::TemplateLibrary,
        schema_version: 1,
        revisions: catalog.revisions().map(RevisionData::from).collect(),
        recommended: catalog
            .recommendations()
            .map(|(_, pin)| pin.into())
            .collect(),
    };
    Ok(serde_json::to_string_pretty(&data)? + "\n")
}
/// Decode every body and recommendation before exposing a catalogue. Exact duplicate
/// records are rejected, even though identical in-memory re-publication is a no-op.
/// # Errors
/// Rejects future/wrong-kind/unknown data, duplicates, invalid bodies and dangling pointers.
pub fn decode_template_library(text: &str) -> Result<TemplateLibrary, TemplateLibraryFileError> {
    let envelope: crate::schema::VersionEnvelope = serde_json::from_str(text)?;
    if envelope.schema_version != 1 {
        return Err(TemplateFileError::Version(envelope.schema_version).into());
    }
    let data: LibraryData = serde_json::from_str(text)?;
    let mut catalog = TemplateLibrary::default();
    for revision in data.revisions {
        let revision = TemplateRevision::try_from(revision)?;
        let pin = revision.reference();
        if catalog.revision(pin).is_some() {
            return Err(TemplateLibraryError::DuplicateRevision(pin).into());
        }
        catalog.publish(Arc::new(revision))?;
    }
    let mut seen = BTreeSet::new();
    for pin in data.recommended {
        let pin = TemplateRevisionRef::try_from(pin)?;
        if !seen.insert(pin.definition) {
            return Err(TemplateLibraryError::DuplicateRecommendation(pin.definition).into());
        }
        catalog.set_recommended(pin.definition, Some(pin))?;
    }
    Ok(catalog)
}

impl From<&TemplateRevision> for RevisionData {
    fn from(revision: &TemplateRevision) -> Self {
        let pin = revision.reference();
        Self {
            definition: pin.definition.to_string(),
            revision: pin.revision.to_string(),
            variants: revision.variants().map(VariantData::from).collect(),
        }
    }
}
impl TryFrom<RevisionData> for TemplateRevision {
    type Error = TemplateLibraryFileError;
    fn try_from(data: RevisionData) -> Result<Self, Self::Error> {
        let pin = TemplateRevisionRef {
            definition: data.definition.parse()?,
            revision: data.revision.parse()?,
        };
        let variants = data
            .variants
            .into_iter()
            .map(SavedTemplateVariant::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(pin, variants)?)
    }
}
