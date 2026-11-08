//! Portable exact paper layouts and immutable saved variant bodies, without library IO.
use crate::template_table::{bottom, right};
use crate::{
    FrameRect, SheetPaper, TemplateDefinitionId, TemplateError, TemplateRevisionId, TemplateTable,
    TemplateVariantId,
};
use std::collections::BTreeSet;

/// Exact pinned variant identity. Saving another revision never changes this reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TemplateVariantRef {
    /// Reusable definition lineage, distinct from project identity.
    pub definition: TemplateDefinitionId,
    /// Immutable revision identity.
    pub revision: TemplateRevisionId,
    /// Stable paper/layout variant within the definition.
    pub variant: TemplateVariantId,
}
/// Retained paper geometry, separate from source-view placement and resolved field values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateLayout {
    paper: SheetPaper,
    content: FrameRect,
    tables: Vec<TemplateTable>,
}
impl TemplateLayout {
    /// Construct a layout with explicit generated-content region and structured tables.
    /// Tables must fit the margin interior. A zero-width/height content region is retained
    /// to preserve legacy tiny-sheet rendering, and means no generated-content capacity.
    /// Paper is custom/unverified: dimensions do not imply a standards-compliant border.
    /// # Errors
    /// Rejects invalid paper, out-of-interior tables/content, or repeated table/cell IDs.
    pub fn new(
        paper: SheetPaper,
        content: FrameRect,
        tables: Vec<TemplateTable>,
    ) -> Result<Self, TemplateError> {
        paper.validate().map_err(TemplateError::Paper)?;
        let interior = FrameRect {
            x: paper.left,
            y: paper.top,
            width: paper.width - paper.left - paper.right,
            height: paper.height - paper.top - paper.bottom,
        };
        let contained = |b: FrameRect| {
            b.x >= interior.x
                && b.y >= interior.y
                && right(b) <= right(interior)
                && bottom(b) <= bottom(interior)
        };
        if !contained(content) {
            return Err(TemplateError::Bounds);
        }
        let mut table_ids = BTreeSet::new();
        let mut cell_ids = BTreeSet::new();
        for table in &tables {
            if !contained(table.bounds()) {
                return Err(TemplateError::Bounds);
            }
            if !table_ids.insert(table.id()) {
                return Err(TemplateError::DuplicateTable(table.id()));
            }
            for cell in table.cells() {
                if !cell_ids.insert(cell.id) {
                    return Err(TemplateError::DuplicateCell(cell.id));
                }
            }
        }
        Ok(Self {
            paper,
            content,
            tables,
        })
    }
    /// Exact oriented dimensions and guide margins in micrometres.
    pub fn paper(&self) -> &SheetPaper {
        &self.paper
    }
    /// Paper-space margin border, derived without rounding.
    pub fn border(&self) -> FrameRect {
        FrameRect {
            x: self.paper.left,
            y: self.paper.top,
            width: self.paper.width - self.paper.left - self.paper.right,
            height: self.paper.height - self.paper.top - self.paper.bottom,
        }
    }
    /// Explicit paper-space content extent; independent of title-block positioning.
    pub fn content(&self) -> FrameRect {
        self.content
    }
    /// Structured blocks with stable tables/cells; no resolved project values are stored.
    pub fn tables(&self) -> &[TemplateTable] {
        &self.tables
    }
}
/// An immutable variant body suitable for canonical storage and future project bundling.
/// Constructing/encoding this value does not publish it or make it assignable in a UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedTemplateVariant {
    reference: TemplateVariantRef,
    name: String,
    layout: TemplateLayout,
    new_assignments: bool,
}
impl SavedTemplateVariant {
    /// Validate the name and retain an already validated layout. Identity allocation and
    /// revision-conflict/publication checks belong to the owning library controller.
    /// # Errors
    /// Rejects blank names, retaining literal multiline names otherwise.
    pub fn new(
        reference: TemplateVariantRef,
        name: String,
        layout: TemplateLayout,
    ) -> Result<Self, TemplateError> {
        if name.trim().is_empty() {
            return Err(TemplateError::BlankName);
        }
        Ok(Self {
            reference,
            name,
            layout,
            new_assignments: true,
        })
    }
    /// Preserve a legacy frame exactly. Omitted legacy title blocks remain renderable,
    /// but require repair in a new revision before new assignment is allowed.
    /// # Errors
    /// Rejects blank names, as with normal variant construction.
    pub fn preserve_legacy(
        reference: TemplateVariantRef,
        name: String,
        layout: TemplateLayout,
    ) -> Result<Self, TemplateError> {
        let allowed = !layout.tables().is_empty();
        let mut value = Self::new(reference, name, layout)?;
        value.new_assignments = allowed;
        Ok(value)
    }
    /// Whether this revision is eligible for new sheet/default assignments.
    pub fn allows_new_assignments(&self) -> bool {
        self.new_assignments
    }
    /// Exact definition/revision/variant pin.
    pub fn reference(&self) -> TemplateVariantRef {
        self.reference
    }
    /// Literal library display name, independent of identity.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Immutable validated paper/table body.
    pub fn layout(&self) -> &TemplateLayout {
        &self.layout
    }
}
