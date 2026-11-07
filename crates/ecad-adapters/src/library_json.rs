//! Independent library schema v1; immutable bodies reuse the strict project-body DTO.

use crate::{
    LibraryFileError, ProjectFileError, schema::VersionEnvelope, schema_definition::DefinitionData,
};
use ecad_domain::{DefinitionRef, LibraryError, SubsystemLibrary, SubsystemRevision};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::Arc};

#[derive(Serialize, Deserialize)]
enum Kind {
    #[serde(rename = "ecad-subsystem-library")]
    SubsystemLibrary,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LibraryData {
    kind: Kind,
    schema_version: u32,
    revisions: Vec<DefinitionData>,
    recommended: Vec<RecommendedData>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecommendedData {
    definition: String,
    revision: String,
}
fn json_error(error: serde_json::Error) -> LibraryFileError {
    ProjectFileError::from(error).into()
}

/// Encode a canonical library catalog, in identity order with final LF.
/// Project format versions are independent of this library schema (currently v1).
/// # Errors
/// Returns a structured encoding failure without changing the catalog.
pub fn encode_library(catalog: &SubsystemLibrary) -> Result<String, LibraryFileError> {
    let data = LibraryData {
        kind: Kind::SubsystemLibrary,
        schema_version: 1,
        revisions: catalog.revisions().map(DefinitionData::from).collect(),
        recommended: catalog
            .recommendations()
            .map(|(id, source)| RecommendedData {
                definition: id.to_string(),
                revision: source.revision().to_string(),
            })
            .collect(),
    };
    Ok(serde_json::to_string_pretty(&data).map_err(json_error)? + "\n")
}

/// Decode all bodies and resolve recommendations before exposing a catalog.
/// # Errors
/// Rejects unsupported schemas, wrong document kind, unknown fields, duplicates,
/// malformed identities, invalid bodies, or recommendations to missing revisions.
pub fn decode_library(text: &str) -> Result<SubsystemLibrary, LibraryFileError> {
    let envelope: VersionEnvelope = serde_json::from_str(text).map_err(json_error)?;
    if envelope.schema_version != 1 {
        return Err(ProjectFileError::UnsupportedVersion(envelope.schema_version).into());
    }
    let data: LibraryData = serde_json::from_str(text).map_err(json_error)?;
    let mut catalog = SubsystemLibrary::default();
    for data in data.revisions {
        let revision = SubsystemRevision::try_from(data)?;
        let source = revision.source();
        if catalog.revision(source).is_some() {
            return Err(LibraryError::DuplicateRevision(source).into());
        }
        catalog.publish(Arc::new(revision))?;
    }
    let mut seen = BTreeSet::new();
    for data in data.recommended {
        let id = data.definition.parse().map_err(ProjectFileError::from)?;
        let revision = data.revision.parse().map_err(ProjectFileError::from)?;
        if !seen.insert(id) {
            return Err(LibraryError::DuplicateRecommendation(id).into());
        }
        catalog.set_recommended(id, Some(DefinitionRef::new(id, revision)))?;
    }
    Ok(catalog)
}
