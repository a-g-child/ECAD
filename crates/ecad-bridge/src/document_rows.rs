//! Read-only logical document projection; separate from persistence and session controllers.
use ecad_domain::{DrawingDocumentLayout, Project, ProjectDrawingDocument};
use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
pub(crate) struct DrawingDocumentRow {
    id: String,
    // Exact initial metadata and identity; elide only when this is the sole package.
    is_default: bool,
    number: String,
    title: String,
    revision: String,
    default_template: &'static str,
    sheets: Vec<String>,
}
impl DrawingDocumentRow {
    fn new(layout: DrawingDocumentLayout, initial: &ProjectDrawingDocument) -> Self {
        let is_default = layout.document == *initial;
        let d = layout.document;
        Self {
            id: d.id.to_string(),
            is_default,
            number: d.number,
            title: d.title,
            revision: d.revision,
            default_template: d.default_template.key(),
            sheets: layout.sheets.into_iter().map(|id| id.to_string()).collect(),
        }
    }
}
pub(crate) fn project(project: &Project) -> Vec<DrawingDocumentRow> {
    let initial = ProjectDrawingDocument::initial(project.id());
    project
        .drawing_document_layout()
        .into_iter()
        .map(|layout| DrawingDocumentRow::new(layout, &initial))
        .collect()
}
