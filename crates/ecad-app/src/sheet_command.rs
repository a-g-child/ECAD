//! Reversible paper operations and closed layout proposals; drawing geometry is untouched.
use crate::{Command, DrawingCommand, Revision, Session};
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
    frame: SheetFrame,
    document: DrawingDocumentId,
    list: Option<DrawingListPage>,
    creation: bool,
    pub(crate) commands: Vec<Command>,
}
impl SheetPreview {
    /// Exact owner in the validated candidate.
    pub fn document(&self) -> DrawingDocumentId {
        self.document
    }
    /// Generated navigation content from the candidate, never copied into the document.
    pub fn list(&self) -> Option<&DrawingListPage> {
        self.list.as_ref()
    }
    /// Whether committing creates a new sheet.
    pub fn is_creation(&self) -> bool {
        self.creation
    }
    /// Resolved frame from the validated candidate, including new owner-local sheet counts.
    pub fn frame(&self) -> &SheetFrame {
        &self.frame
    }
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
    let document = if create {
        session
            .project()
            .drawing_documents()
            .last()
            .expect("project owner")
            .id
    } else {
        session.project().document_sheet_position(sheet.id)?.0
    };
    preview_sheet_in_document(session, sheet, create, document, None)
}
/// Validate a sheet in an explicit package, optionally creating its fresh schematic view atomically.
/// Replacements cannot transfer ownership; dedicated document commands perform transfers.
/// # Errors
/// Rejects invalid owners, role/view mismatch, duplicates or any invalid candidate without mutation.
pub fn preview_sheet_in_document(
    session: &Session,
    sheet: DrawingSheet,
    create: bool,
    document: DrawingDocumentId,
    new_view: Option<DrawingView>,
) -> Result<SheetPreview, ModelError> {
    let mut commands = Vec::new();
    if let Some(view) = new_view {
        if !create
            || sheet.kind != SheetKind::Schematic
            || sheet.placement.as_ref().map(|p| p.view) != Some(view.id)
        {
            return Err(SheetError::CreationViewMismatch.into());
        }
        commands.push(Command::Drawing(DrawingCommand::InsertView(view)));
    }
    let command = if create {
        SheetCommand::InsertInDocument {
            sheet: sheet.clone(),
            document,
            index: session.project().sheets_in_document(document)?.len(),
        }
    } else {
        if session.project().sheet_document(sheet.id) != Some(document) {
            return Err(DrawingDocumentError::Membership(sheet.id).into());
        }
        SheetCommand::Replace(sheet.clone())
    };
    commands.push(Command::Sheet(command));
    let mut candidate = session.project().clone();
    for command in &commands {
        command.apply(&mut candidate)?;
    }
    candidate.validate()?;
    Ok(SheetPreview {
        revision: session.revision(),
        frame: candidate.sheet_frame(sheet.id)?,
        list: candidate.drawing_list(sheet.id)?,
        document,
        creation: create,
        sheet,
        commands,
    })
}
