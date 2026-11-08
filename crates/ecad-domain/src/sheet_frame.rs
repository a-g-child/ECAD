//! Immutable bundled paper frames and live fields; projections never alter source drawings.
use crate::{DrawingValues, MetadataField, Project, SheetError, SheetId};

/// Pinned bundled template identity. New layouts require a new variant, never a silent revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetTemplate {
    /// Margin border and a bottom-right, two-column drawing-information table.
    StandardV1,
}
impl SheetTemplate {
    /// Stable portable identity, independent of a translated display name.
    pub fn key(self) -> &'static str {
        match self {
            Self::StandardV1 => "standard-v1",
        }
    }
}
/// Nonnegative paper-space rectangle, in integer micrometres.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameRect {
    /// Horizontal position from paper left.
    pub x: u32,
    /// Vertical position from paper top.
    pub y: u32,
    /// Physical width.
    pub width: u32,
    /// Physical height.
    pub height: u32,
}
/// Typed bindings; no executable expressions or machine-local profile lookups.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameField {
    /// Copied portable project metadata.
    Metadata(MetadataField),
    /// Literal drawing-package title.
    DocumentTitle,
    /// Literal drawing-package number.
    DocumentNumber,
    /// Engineering revision of the owning package, independent of Git/undo.
    DocumentRevision,
    /// Literal sheet title.
    SheetTitle,
    /// Literal drawing number, never inferred from order.
    SheetNumber,
    /// One-based owner-local ordinal and count; not an output-subset count.
    DocumentSheetCount,
}
impl FrameField {
    /// Stable semantic identity, also used by the bundled template's cells.
    pub fn key(self) -> &'static str {
        match self {
            Self::Metadata(field) => field.key(),
            Self::DocumentTitle => "document.title",
            Self::DocumentNumber => "document.number",
            Self::DocumentRevision => "document.revision",
            Self::SheetTitle => "sheet.title",
            Self::SheetNumber => "sheet.number",
            Self::DocumentSheetCount => "document.sheet_count",
        }
    }
    fn caption(self) -> &'static str {
        match self {
            Self::Metadata(field) => field.label(),
            Self::DocumentTitle => "Document",
            Self::DocumentNumber => "Document number",
            Self::DocumentRevision => "Revision",
            Self::SheetTitle => "Drawing title",
            Self::SheetNumber => "Drawing number",
            Self::DocumentSheetCount => "Sheet in document",
        }
    }
}
/// One resolved cell; an absent optional value stays absent rather than becoming fake metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameCell {
    /// Stable typed binding, distinct from its caption.
    pub field: FrameField,
    /// Literal caption.
    pub caption: &'static str,
    /// Exact literal multiline value; missing optional project metadata is None.
    pub value: Option<String>,
    /// Physical cell bounds.
    pub bounds: FrameRect,
}
/// Derived paper geometry and values; never inserted into the electrical drawing aggregate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetFrame {
    /// Template inherited from the owning drawing package.
    pub template: SheetTemplate,
    /// Margin border, always present even on a very small valid sheet.
    pub border: FrameRect,
    /// Resolved title-block cells, in stable row-major order.
    pub cells: Vec<FrameCell>,
    /// True when the interior cannot contain the fixed 180 × 40 mm title block.
    /// The border remains visible; no overlapping or zero-sized cells are manufactured.
    pub title_block_too_small: bool,
    /// Project drawing fallbacks; frame geometry and text remain in paper units.
    pub appearance: DrawingValues,
}
impl Project {
    /// Resolve an existing sheet's bundled frame from current owner/project/sheet values.
    /// Does not change geometry, electrical identities, metadata or project history.
    /// # Errors
    /// Rejects unavailable sheet/owner identities rather than guessing their metadata.
    pub fn sheet_frame(&self, id: SheetId) -> Result<SheetFrame, SheetError> {
        let sheet = self.sheet(id).ok_or(SheetError::Missing(id))?;
        let (owner, ordinal) = self.document_sheet_position(id)?;
        let document = self.drawing_document(owner).expect("validated owner");
        let count = self.sheets_in_document(owner)?.len();
        let p = &sheet.paper;
        let border = FrameRect {
            x: p.left,
            y: p.top,
            width: p.width - p.left - p.right,
            height: p.height - p.top - p.bottom,
        };
        let too_small = border.width < 180_000 || border.height < 40_000;
        let fields = [
            FrameField::Metadata(MetadataField::CompanyName),
            FrameField::Metadata(MetadataField::ProjectTitle),
            FrameField::DocumentTitle,
            FrameField::SheetTitle,
            FrameField::DocumentNumber,
            FrameField::SheetNumber,
            FrameField::DocumentRevision,
            FrameField::DocumentSheetCount,
        ];
        let cells = if too_small {
            Vec::new()
        } else {
            fields
                .into_iter()
                .enumerate()
                .map(|(i, field)| {
                    let value = match field {
                        FrameField::Metadata(field) => {
                            self.metadata().value(field).map(str::to_owned)
                        }
                        FrameField::DocumentTitle => Some(document.title.clone()),
                        FrameField::DocumentNumber => Some(document.number.clone()),
                        FrameField::DocumentRevision => Some(document.revision.clone()),
                        FrameField::SheetTitle => Some(sheet.title.clone()),
                        FrameField::SheetNumber => Some(sheet.number.clone()),
                        FrameField::DocumentSheetCount => {
                            Some(format!("{} / {count}", ordinal + 1))
                        }
                    };
                    FrameCell {
                        field,
                        caption: field.caption(),
                        value,
                        bounds: FrameRect {
                            x: p.width - p.right - 180_000 + (i as u32 % 2) * 90_000,
                            y: p.height - p.bottom - 40_000 + (i as u32 / 2) * 10_000,
                            width: 90_000,
                            height: 10_000,
                        },
                    }
                })
                .collect()
        };
        Ok(SheetFrame {
            template: document.default_template,
            border,
            cells,
            title_block_too_small: too_small,
            appearance: self.drawing().settings().base.clone(),
        })
    }
}
