//! Read-only logical document projection; separate from persistence and session controllers.
use ecad_domain::{DrawingDocumentLayout, Project};
use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
pub(crate) struct DrawingDocumentRow {
    id: String,
    number: String,
    title: String,
    revision: String,
    sheets: Vec<String>,
}
impl From<DrawingDocumentLayout> for DrawingDocumentRow {
    fn from(layout: DrawingDocumentLayout) -> Self {
        let d = layout.document;
        Self {
            id: d.id.to_string(),
            number: d.number,
            title: d.title,
            revision: d.revision,
            sheets: layout.sheets.into_iter().map(|id| id.to_string()).collect(),
        }
    }
}
pub(crate) fn project(project: &Project) -> Vec<DrawingDocumentRow> {
    project
        .drawing_document_layout()
        .into_iter()
        .map(Into::into)
        .collect()
}
