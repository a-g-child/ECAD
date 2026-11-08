//! Paper-space sheets and view placements, independent of schematic geometry.
use crate::{DrawingPoint, SheetId, ViewId};
use std::{error::Error, fmt};
/// Physical paper dimensions and nonnegative guide margins in micrometres.
/// Width/height are already oriented; changing orientation swaps them explicitly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetPaper {
    /// Oriented page width.
    pub width: u32,
    /// Oriented page height.
    pub height: u32,
    /// Left guide margin.
    pub left: u32,
    /// Right guide margin.
    pub right: u32,
    /// Top guide margin.
    pub top: u32,
    /// Bottom guide margin.
    pub bottom: u32,
}
impl Default for SheetPaper {
    fn default() -> Self {
        Self {
            width: 297_000,
            height: 210_000,
            left: 10_000,
            right: 10_000,
            top: 10_000,
            bottom: 10_000,
        }
    }
}
impl SheetPaper {
    /// Validate positive paper and a positive interior after guide margins.
    /// # Errors
    /// Rejects zero dimensions or margins consuming either entire axis.
    pub fn validate(&self) -> Result<(), SheetError> {
        if self.width == 0 || self.height == 0 {
            return Err(SheetError::Paper);
        }
        if u64::from(self.left) + u64::from(self.right) >= u64::from(self.width)
            || u64::from(self.top) + u64::from(self.bottom) >= u64::from(self.height)
        {
            return Err(SheetError::Margins);
        }
        Ok(())
    }
}
/// One view placement: paper = paper_origin + (view_point - view_origin) * numerator/denominator.
/// Scale changes presentation, never drawing coordinates, text intent or electrical topology.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetViewPlacement {
    /// Existing drawing view; the view may appear on more than one sheet.
    pub view: ViewId,
    /// Paper-space position in micrometres, measured from the upper-left page corner.
    pub paper_origin: DrawingPoint,
    /// Model-space point placed at paper_origin.
    pub view_origin: DrawingPoint,
    /// Positive paper/model scale numerator.
    pub numerator: u32,
    /// Positive paper/model scale denominator.
    pub denominator: u32,
}
/// A sheet in the project's initial ordered drawing set; stable ID is independent of number/order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingSheet {
    /// Persistent identity.
    pub id: SheetId,
    /// Content role; generated and cover pages cannot carry source-view placements.
    pub kind: crate::SheetKind,
    /// Pinned frame override; None retains the owning document's template.
    pub template: Option<crate::SheetTemplate>,
    /// Nonblank unique drawing number, separate from its ordinal position.
    pub number: String,
    /// Literal multiline sheet title.
    pub title: String,
    /// Paper dimensions and guides.
    pub paper: SheetPaper,
    /// Optional single schematic view; cover and generated-list roles stay unplaced.
    pub placement: Option<SheetViewPlacement>,
}
impl DrawingSheet {
    /// Validate local sheet values; project insertion also checks view references and uniqueness.
    /// # Errors
    /// Rejects blank number/title, invalid paper/margins/scale or a source on a cover/list.
    pub fn validate(&self) -> Result<(), SheetError> {
        if self.number.trim().is_empty() || self.title.trim().is_empty() {
            return Err(SheetError::Blank);
        }
        self.paper.validate()?;
        if self.kind != crate::SheetKind::Schematic && self.placement.is_some() {
            return Err(SheetError::RolePlacement);
        }
        if self
            .placement
            .as_ref()
            .is_some_and(|p| p.numerator == 0 || p.denominator == 0)
        {
            return Err(SheetError::Scale);
        }
        Ok(())
    }
}
/// Sheet validation failed without changing existing content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SheetError {
    /// Cover/generated-list content cannot also place an editable schematic source.
    RolePlacement,
    /// A proposed new view must belong to a new schematic sheet and match its placement.
    CreationViewMismatch,
    /// Invalid drawing-document metadata or sheet ownership.
    Document(crate::DrawingDocumentError),
    /// Paper dimensions must be positive.
    Paper,
    /// Margins must leave a positive interior.
    Margins,
    /// Scale terms must be positive.
    Scale,
    /// Number and title must not be blank.
    Blank,
    /// A stable sheet ID already exists.
    Duplicate(SheetId),
    /// A drawing number is already used by another sheet.
    NumberInUse(String),
    /// A requested sheet is unavailable.
    Missing(SheetId),
    /// A referenced drawing view is unavailable.
    MissingView(ViewId),
    /// The requested ordering position is outside the drawing set.
    Order,
    /// Electrical subsystem bodies cannot silently discard paper layouts.
    ReusableSheetsUnsupported,
}
impl fmt::Display for SheetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreationViewMismatch => {
                f.write_str("The new view must match the new schematic sheet placement")
            }
            Self::RolePlacement => {
                f.write_str("Cover and drawing-list sheets cannot place a schematic view")
            }
            Self::Document(error) => error.fmt(f),
            Self::Paper => f.write_str("Paper width and height must be positive"),
            Self::Margins => f.write_str("Margins must leave a positive paper interior"),
            Self::Scale => f.write_str("Both view scale terms must be positive"),
            Self::Blank => f.write_str("Sheet number and title must not be blank"),
            Self::Duplicate(id) => write!(f, "Sheet {id} already exists"),
            Self::NumberInUse(n) => write!(f, "Sheet number {n} is already in use"),
            Self::Missing(id) => write!(f, "Sheet {id} is unavailable"),
            Self::MissingView(id) => write!(f, "Sheet drawing view {id} is unavailable"),
            Self::Order => f.write_str("Sheet order is outside the drawing set"),
            Self::ReusableSheetsUnsupported => {
                f.write_str("Project sheets are not reusable electrical subsystem content")
            }
        }
    }
}
impl Error for SheetError {}

impl From<crate::DrawingDocumentError> for SheetError {
    fn from(error: crate::DrawingDocumentError) -> Self {
        Self::Document(error)
    }
}
