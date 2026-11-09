//! Immutable bundled paper frames and live fields; projections never alter source drawings.
use crate::{DrawingValues, MetadataField, Project, SheetError, SheetId};

/// Pinned bundled template identity. New layouts require a new variant, never a silent revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetTemplate {
    /// Exact project-bundled immutable template variant.
    Saved(crate::TemplateVariantRef),
    /// Margin border and a bottom-right, two-column drawing-information table.
    StandardV1,
    /// Centred information panel for a cover page.
    CoverV1,
    /// Compact four-column footer leaving space for drawing-list entries.
    DrawingListV1,
}
impl SheetTemplate {
    /// Frozen legacy catalogue for migration; actual saved variants live in project bundles.
    pub const ALL: [Self; 3] = [Self::StandardV1, Self::CoverV1, Self::DrawingListV1];
    /// Resolve known identities without guessing missing/future templates.
    pub fn from_key(key: &str) -> Option<Self> {
        if let Some(rest) = key.strip_prefix("saved:") {
            let parts: Vec<_> = rest.split(':').collect();
            if parts.len() != 3 {
                return None;
            }
            return Some(Self::Saved(crate::TemplateVariantRef {
                definition: parts[0].parse().ok()?,
                revision: parts[1].parse().ok()?,
                variant: parts[2].parse().ok()?,
            }));
        }
        Self::ALL.into_iter().find(|t| t.key() == key)
    }
    /// Reserved capture-v1 lineage for the frozen built-in frame. These constants match
    /// the versioned migration recipe and keep UI identity checks independent of names.
    pub fn preserved_definition(self) -> Option<crate::TemplateDefinitionId> {
        let id = match self {
            Self::StandardV1 => 0xefe8c6f7729ac337099733ea2598c674,
            Self::CoverV1 => 0x10dac139e1dcdbaf38ab96d808be7006,
            Self::DrawingListV1 => 0x8e294cb9711674ec016914a50e010e52,
            Self::Saved(_) => return None,
        };
        Some(crate::TemplateDefinitionId::new(id).expect("nonzero reserved identity"))
    }
    /// Display name independent of its pinned identity.
    pub fn label(self) -> &'static str {
        match self {
            Self::Saved(_) => "Saved drawing template",
            Self::StandardV1 => "Standard drawing",
            Self::CoverV1 => "Cover page",
            Self::DrawingListV1 => "Drawing list",
        }
    }
    /// Stable portable identity, independent of a translated display name.
    pub fn key(self) -> String {
        match self {
            Self::Saved(pin) => {
                format!("saved:{}:{}:{}", pin.definition, pin.revision, pin.variant)
            }
            Self::StandardV1 => "standard-v1".into(),
            Self::CoverV1 => "cover-v1".into(),
            Self::DrawingListV1 => "drawing-list-v1".into(),
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
    /// Resolve a known semantic key without treating an unknown field as literal text.
    pub fn from_key(key: &str) -> Option<Self> {
        if let Some(field) = MetadataField::from_key(key) {
            return Some(Self::Metadata(field));
        }
        [
            Self::DocumentTitle,
            Self::DocumentNumber,
            Self::DocumentRevision,
            Self::SheetTitle,
            Self::SheetNumber,
            Self::DocumentSheetCount,
        ]
        .into_iter()
        .find(|field| field.key() == key)
    }
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
    pub(crate) fn caption(self) -> &'static str {
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
    /// Portable cell picture, fitted by the renderer without changing content.
    pub picture: Option<crate::Picture>,
    /// Stable template-cell identity, independent of its value/binding.
    pub id: crate::TemplateCellId,
    /// Single typed binding, or None for literal/composed text; distinct from cell identity.
    pub field: Option<FrameField>,
    /// Literal caption.
    pub caption: String,
    /// Cell overrides resolved over template defaults; project fallbacks remain in appearance.
    pub text_styles: crate::TemplateTextStyles,
    /// Exact literal multiline value; missing optional project metadata is None.
    pub value: Option<String>,
    /// Physical cell bounds.
    pub bounds: FrameRect,
}
/// Derived paper geometry and values; never inserted into the electrical drawing aggregate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetFrame {
    /// Resolved explicit sheet choice or inherited owning-document template.
    pub template: SheetTemplate,
    /// Margin border, always present even on a very small valid sheet.
    pub border: FrameRect,
    /// Paper-space region above the information block, available for generated content.
    pub content: FrameRect,
    /// Resolved title-block cells, in stable row-major order.
    pub cells: Vec<FrameCell>,
    /// Paper artwork with text variables resolved for this sheet; retained layout stays unchanged.
    pub graphics: Vec<crate::TemplateGraphic>,
    /// True when the interior cannot contain the selected template's fixed information panel.
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
        let template = self
            .sheet_template_pin(sheet, document.default_template)
            .map(SheetTemplate::Saved)
            .unwrap_or(sheet.template.unwrap_or(document.default_template));
        let layout = self.sheet_template_layout(sheet, document.default_template)?;
        let resolve_field = |field| match field {
            FrameField::Metadata(field) => self.metadata().value(field).map(str::to_owned),
            FrameField::DocumentTitle => Some(document.title.clone()),
            FrameField::DocumentNumber => Some(document.number.clone()),
            FrameField::DocumentRevision => Some(document.revision.clone()),
            FrameField::SheetTitle => Some(sheet.title.clone()),
            FrameField::SheetNumber => Some(sheet.number.clone()),
            FrameField::DocumentSheetCount => Some(format!("{} / {count}", ordinal + 1)),
        };
        let mut cells = Vec::new();
        for table in layout.tables() {
            for cell in table.cells() {
                let field = match cell.contents.as_slice() {
                    [crate::TemplateTextRun::Field(field)] => Some(*field),
                    _ => None,
                };
                let resolved = cell.resolve(resolve_field);
                cells.push(FrameCell {
                    picture: cell.picture.clone(),
                    id: cell.id,
                    field,
                    caption: cell.caption.clone(),
                    text_styles: crate::TemplateTextStyles {
                        caption: cell
                            .text_styles
                            .caption
                            .over(&layout.text_defaults().caption),
                        content: cell
                            .text_styles
                            .content
                            .over(&layout.text_defaults().content),
                    },
                    value: (resolved.missing.is_empty() || field.is_none())
                        .then_some(resolved.value),
                    bounds: FrameRect {
                        x: table.bounds().x + cell.bounds.x,
                        y: table.bounds().y + cell.bounds.y,
                        width: cell.bounds.width,
                        height: cell.bounds.height,
                    },
                });
            }
        }
        Ok(SheetFrame {
            template,
            border: layout.border(),
            content: layout.content(),
            cells,
            graphics: layout
                .graphics()
                .iter()
                .map(|g| {
                    let mut resolved = g.clone();
                    if let crate::TemplateGraphicShape::Text(runs) = &g.shape {
                        let text = crate::resolve_template_text(runs, resolve_field);
                        resolved.shape = crate::TemplateGraphicShape::Text(vec![
                            crate::TemplateTextRun::Literal(text.value),
                        ]);
                    }
                    resolved
                })
                .collect(),
            title_block_too_small: match self.sheet_template_pin(sheet, document.default_template) {
                Some(pin) => !self.saved_template(pin)?.allows_new_assignments(),
                None => layout.tables().is_empty(),
            },
            appearance: self.drawing().settings().base.clone(),
        })
    }
}
