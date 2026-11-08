//! Logical drawing-package metadata, distinct from the drawing geometry aggregate and app session.
use crate::{DrawingDocumentId, ProjectId, SheetId};
use std::{error::Error, fmt};
/// A logical drawing package owning ordered sheets within one project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectDrawingDocument {
    /// Persistent document identity, independent of project/sheet/view identities.
    pub id: DrawingDocumentId,
    /// Unique nonblank document number within the project.
    pub number: String,
    /// Literal multiline package title.
    pub title: String,
    /// Explicit engineering revision text, unrelated to undo/Git revisions.
    pub revision: String,
    /// Immutable bundled frame inherited by every sheet in this package.
    pub default_template: crate::SheetTemplate,
}
impl ProjectDrawingDocument {
    /// Deterministic initial package for new/legacy projects. Its typed ID uses the project ID's
    /// canonical value in a distinct namespace, without random IDs or timestamps during decode.
    pub fn initial(project: ProjectId) -> Self {
        Self {
            id: project
                .to_string()
                .parse()
                .expect("canonical project identity"),
            number: "1".into(),
            title: "Drawings".into(),
            revision: String::new(),
            default_template: crate::SheetTemplate::StandardV1,
        }
    }
    /// Validate required display metadata without changing literal text.
    /// # Errors
    /// Rejects blank document number/title; revision may be empty.
    pub fn validate(&self) -> Result<(), DrawingDocumentError> {
        if self.number.trim().is_empty() || self.title.trim().is_empty() {
            return Err(DrawingDocumentError::Blank);
        }
        Ok(())
    }
}
/// Ordered membership for one package; each project sheet must occur exactly once overall.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingDocumentLayout {
    /// Package metadata.
    pub document: ProjectDrawingDocument,
    /// Stable sheet IDs in the package's explicit order.
    pub sheets: Vec<SheetId>,
}
/// Invalid document metadata, ownership or ordering; rejected changes are atomic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawingDocumentError {
    /// Document number/title cannot be blank.
    Blank,
    /// An identity already exists.
    Duplicate(DrawingDocumentId),
    /// Another document uses this number.
    NumberInUse(String),
    /// A referenced package is unavailable.
    Missing(DrawingDocumentId),
    /// Remove/move its sheets before removing the package.
    InUse(DrawingDocumentId),
    /// Projects retain at least one package, including an empty initial package.
    LastDocument,
    /// Membership references an unavailable sheet.
    MissingSheet(SheetId),
    /// A sheet is missing from membership or appears more than once.
    Membership(SheetId),
    /// Document/sheet ordinal is outside the target collection.
    Order,
    /// Reusable electrical bodies cannot discard custom document metadata.
    ReusableDocumentsUnsupported,
}
impl fmt::Display for DrawingDocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blank => f.write_str("Drawing document number and title must not be blank"),
            Self::Duplicate(id) => write!(f, "Drawing document {id} already exists"),
            Self::NumberInUse(n) => write!(f, "Drawing document number {n} is already used"),
            Self::Missing(id) => write!(f, "Drawing document {id} is unavailable"),
            Self::InUse(id) => write!(f, "Drawing document {id} still owns sheets"),
            Self::LastDocument => {
                f.write_str("A project must retain at least one drawing document")
            }
            Self::MissingSheet(id) => write!(f, "Document references unavailable sheet {id}"),
            Self::Membership(id) => {
                write!(f, "Sheet {id} must belong to exactly one drawing document")
            }
            Self::Order => f.write_str("Drawing document or sheet order is outside its owner"),
            Self::ReusableDocumentsUnsupported => f.write_str(
                "Custom drawing documents are not reusable electrical subsystem content",
            ),
        }
    }
}
impl Error for DrawingDocumentError {}
