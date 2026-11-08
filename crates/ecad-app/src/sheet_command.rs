//! Reversible paper operations and closed layout proposals; drawing geometry is untouched.
use crate::{Revision, Session};
use ecad_domain::*;
/// A committed edit to the project's initial ordered drawing set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SheetCommand {
    /// Insert exact sheet state at its ordinal.
    Insert {
        /// Validated paper/view state.
        sheet: DrawingSheet,
        /// Zero-based final position.
        index: usize,
    },
    /// Insert exact sheet state into an explicit owner; used by ownership-safe undo.
    InsertInDocument {
        /// Validated sheet state.
        sheet: DrawingSheet,
        /// Existing owner.
        document: DrawingDocumentId,
        /// Owner-local ordinal.
        index: usize,
    },
    /// Replace sheet properties without reordering.
    Replace(DrawingSheet),
    /// Remove only a paper sheet, retaining its source view.
    Remove(SheetId),
    /// Reorder without changing drawing numbers.
    Move {
        /// Existing sheet.
        id: SheetId,
        /// Zero-based final position.
        index: usize,
    },
}
impl SheetCommand {
    pub(crate) fn apply(&self, project: &mut Project) -> Result<Option<Self>, ModelError> {
        Ok(Some(match self {
            Self::Insert { sheet, index } => {
                project.insert_sheet(sheet.clone(), *index)?;
                Self::Remove(sheet.id)
            }
            Self::InsertInDocument {
                sheet,
                document,
                index,
            } => {
                project.insert_sheet_in_document(sheet.clone(), *document, *index)?;
                Self::Remove(sheet.id)
            }
            Self::Replace(sheet) => {
                let old = project.replace_sheet(sheet.clone())?;
                if old == *sheet {
                    return Ok(None);
                }
                Self::Replace(old)
            }
            Self::Remove(id) => {
                let (document, index) = project.document_sheet_position(*id)?;
                let (sheet, _) = project.remove_sheet(*id)?;
                Self::InsertInDocument {
                    sheet,
                    document,
                    index,
                }
            }
            Self::Move { id, index } => {
                let old = project.move_sheet(*id, *index)?;
                if old == *index {
                    return Ok(None);
                }
                Self::Move {
                    id: *id,
                    index: old,
                }
            }
        }))
    }
}
/// An immutable validated paper edit at its original document revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetPreview {
    revision: Revision,
    sheet: DrawingSheet,
    pub(crate) command: SheetCommand,
}
impl SheetPreview {
    /// Revision and session which authorized the proposal.
    pub fn revision(&self) -> Revision {
        self.revision
    }
    /// Exact reviewed paper/view state.
    pub fn sheet(&self) -> &DrawingSheet {
        &self.sheet
    }
}
/// Validate a complete sheet proposal without editing the session.
/// New sheets append to the drawing set; replacements retain their existing position.
/// # Errors
/// Rejects missing/duplicate identities/numbers and invalid paper, margins, scale or view references.
pub fn preview_sheet(
    session: &Session,
    sheet: DrawingSheet,
    create: bool,
) -> Result<SheetPreview, ModelError> {
    let command = if create {
        SheetCommand::Insert {
            sheet: sheet.clone(),
            index: session.project().sheets().len(),
        }
    } else {
        SheetCommand::Replace(sheet.clone())
    };
    let mut candidate = session.project().clone();
    command.apply(&mut candidate)?;
    Ok(SheetPreview {
        revision: session.revision(),
        sheet,
        command,
    })
}
