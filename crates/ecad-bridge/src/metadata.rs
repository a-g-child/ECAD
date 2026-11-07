//! Metadata editor projection and strict semantic-key parsing, separate from persisted DTOs.
use crate::dto::BridgeError;
use ecad_domain::{MetadataField, ProjectMetadata};
use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Clone, Debug, Serialize)]
pub(crate) struct MetadataRow {
    key: &'static str,
    label: &'static str,
    value: String,
}
pub(crate) fn project(metadata: &ProjectMetadata) -> Vec<MetadataRow> {
    selected(metadata, MetadataField::ALL)
}
pub(crate) fn selected(
    metadata: &ProjectMetadata,
    fields: impl IntoIterator<Item = MetadataField>,
) -> Vec<MetadataRow> {
    fields
        .into_iter()
        .map(|field| MetadataRow {
            key: field.key(),
            label: field.label(),
            value: metadata.value(field).unwrap_or_default().into(),
        })
        .collect()
}
pub(crate) fn parse(values: BTreeMap<String, String>) -> Result<ProjectMetadata, BridgeError> {
    let fields = values
        .into_iter()
        .map(|(key, value)| {
            MetadataField::from_key(&key)
                .map(|field| (field, value))
                .ok_or_else(|| {
                    BridgeError::new("metadata", format!("Unknown metadata field: {key}"))
                })
        })
        .collect::<Result<_, _>>()?;
    Ok(ProjectMetadata::new(fields))
}
