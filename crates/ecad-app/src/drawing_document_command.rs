//! Reversible logical-package metadata, ordering and sheet ownership; no geometry snapshots.
use ecad_domain::*;
/// One committed edit to logical drawing packages, distinct from application session lifecycle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawingDocumentCommand {
    /// Insert an empty package at an explicit position.
    Insert {
        /// Complete literal metadata.
        document: ProjectDrawingDocument,
        /// Zero-based package ordinal.
        index: usize,
    },
    /// Rename/renumber/revise one package, retaining membership.
    Replace(ProjectDrawingDocument),
    /// Remove an empty package; at least one must remain.
    Remove(DrawingDocumentId),
    /// Reorder a package together with its sheets.
    Move {
        /// Existing package.
        id: DrawingDocumentId,
        /// Final ordinal.
        index: usize,
    },
    /// Move/reorder a sheet without renumbering or changing its source drawing.
    MoveSheet {
        /// Existing sheet.
        sheet: SheetId,
        /// Target owner.
        document: DrawingDocumentId,
        /// Final owner-local ordinal.
        index: usize,
    },
    /// Install reviewed metadata/membership; stores IDs/order, not duplicate source geometry.
    Layout(Vec<DrawingDocumentLayout>),
}
impl DrawingDocumentCommand {
    pub(crate) fn apply(&self, project: &mut Project) -> Result<Option<Self>, ModelError> {
        Ok(Some(match self {
            Self::Insert { document, index } => {
                project.insert_drawing_document(document.clone(), *index)?;
                Self::Remove(document.id)
            }
            Self::Replace(value) => {
                let old = project.replace_drawing_document(value.clone())?;
                if old == *value {
                    return Ok(None);
                }
                Self::Replace(old)
            }
            Self::Remove(id) => {
                let (document, index) = project.remove_drawing_document(*id)?;
                Self::Insert { document, index }
            }
            Self::Move { id, index } => {
                let old = project.move_drawing_document(*id, *index)?;
                if old == *index {
                    return Ok(None);
                }
                Self::Move {
                    id: *id,
                    index: old,
                }
            }
            Self::MoveSheet {
                sheet,
                document,
                index,
            } => {
                let (old_document, old_index) =
                    project.move_sheet_to_document(*sheet, *document, *index)?;
                if old_document == *document && old_index == *index {
                    return Ok(None);
                }
                Self::MoveSheet {
                    sheet: *sheet,
                    document: old_document,
                    index: old_index,
                }
            }
            Self::Layout(value) => {
                let old = project.replace_drawing_document_layout(value.clone())?;
                if old == *value {
                    return Ok(None);
                }
                Self::Layout(old)
            }
        }))
    }
}
