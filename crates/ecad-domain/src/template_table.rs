//! Retained title-block partitions, independent of widgets and electrical topology.
use crate::{FrameField, FrameRect, TemplateCellId, TemplateTableId};
use std::{collections::BTreeSet, error::Error, fmt};

/// Literal or typed field run. Every literal supports multiline text from creation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateTextRun {
    /// Exact literal text, including whitespace and line breaks.
    Literal(String),
    /// Portable metadata or drawing field; never an executable expression.
    Field(FrameField),
}
/// A retained cell. Bounds are relative to its table's upper-left corner, in µm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateCell {
    /// Stable identity, separate from caption, binding and position.
    pub id: TemplateCellId,
    /// Positive local rectangle forming part of the table partition.
    pub bounds: FrameRect,
    /// Literal multiline caption.
    pub caption: String,
    /// Ordered value runs; empty content is a valid blank cell.
    pub contents: Vec<TemplateTextRun>,
    /// Independent caption/content appearance, inherited per property.
    pub text_styles: crate::TemplateTextStyles,
}
impl TemplateCell {
    /// Resolve with an explicit sample/real binding context. Missing fields remain
    /// diagnostic identities; literal text and resolved values are never trimmed.
    pub fn resolve(&self, field: impl FnMut(FrameField) -> Option<String>) -> TemplateText {
        resolve_template_text(&self.contents, field)
    }
}
/// Resolve literal and variable runs without mutating their retained bindings.
pub fn resolve_template_text(
    contents: &[TemplateTextRun],
    mut field: impl FnMut(FrameField) -> Option<String>,
) -> TemplateText {
    let mut result = TemplateText {
        value: String::new(),
        missing: Vec::new(),
    };
    for run in contents {
        match run {
            TemplateTextRun::Literal(text) => result.value.push_str(text),
            TemplateTextRun::Field(key) => match field(*key) {
                Some(value) => result.value.push_str(&value),
                None => {
                    if !result.missing.contains(key) {
                        result.missing.push(*key);
                    }
                }
            },
        }
    }
    result
}
/// Resolved text with explicit missing-field diagnostics, separate from saved content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateText {
    /// Available literal/resolved runs in their original order.
    pub value: String,
    /// Distinct missing fields in first-use order.
    pub missing: Vec<FrameField>,
}
/// Immutable validated rectangular partition; supports unequal divisions per row and spans.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateTable {
    id: TemplateTableId,
    bounds: FrameRect,
    cells: Vec<TemplateCell>,
}
impl TemplateTable {
    /// Validate and retain a complete, nonoverlapping partition. Input cell order is
    /// retained as presentation order, independently of identity and physical position.
    /// # Errors
    /// Rejects empty/zero/out-of-bounds geometry, duplicate IDs, overlaps and gaps.
    /// Coordinates are widened before arithmetic, including on extreme u32 inputs.
    pub fn new(
        id: TemplateTableId,
        bounds: FrameRect,
        cells: Vec<TemplateCell>,
    ) -> Result<Self, TemplateError> {
        if bounds.width == 0
            || bounds.height == 0
            || right(bounds) > u64::from(u32::MAX)
            || bottom(bounds) > u64::from(u32::MAX)
        {
            return Err(TemplateError::Bounds);
        }
        let mut ids = BTreeSet::new();
        let mut area = 0u128;
        for (index, cell) in cells.iter().enumerate() {
            cell.text_styles.validate()?;
            if !ids.insert(cell.id) {
                return Err(TemplateError::DuplicateCell(cell.id));
            }
            let b = cell.bounds;
            if b.width == 0
                || b.height == 0
                || right(b) > u64::from(bounds.width)
                || bottom(b) > u64::from(bounds.height)
            {
                return Err(TemplateError::Bounds);
            }
            if cells[..index].iter().any(|other| overlaps(b, other.bounds)) {
                return Err(TemplateError::Overlap);
            }
            area += u128::from(b.width) * u128::from(b.height);
        }
        if area != u128::from(bounds.width) * u128::from(bounds.height) {
            return Err(TemplateError::Gap);
        }
        Ok(Self { id, bounds, cells })
    }
    /// Stable table identity within a layout.
    pub fn id(&self) -> TemplateTableId {
        self.id
    }
    /// Table extent in paper coordinates, in micrometres.
    pub fn bounds(&self) -> FrameRect {
        self.bounds
    }
    /// Validated local cells, in retained presentation order.
    pub fn cells(&self) -> &[TemplateCell] {
        &self.cells
    }
}
pub(crate) fn right(b: FrameRect) -> u64 {
    u64::from(b.x) + u64::from(b.width)
}
pub(crate) fn bottom(b: FrameRect) -> u64 {
    u64::from(b.y) + u64::from(b.height)
}
pub(crate) fn overlaps(a: FrameRect, b: FrameRect) -> bool {
    u64::from(a.x) < right(b)
        && u64::from(b.x) < right(a)
        && u64::from(a.y) < bottom(b)
        && u64::from(b.y) < bottom(a)
}
/// Invalid structured template input; constructors leave prior definitions untouched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateError {
    /// Positive geometry must be representable and contained in its owner.
    Bounds,
    /// Invalid or repeated artwork identity, geometry, style or resource budget.
    Graphic,
    /// Table cells intersect with positive area.
    Overlap,
    /// Cells do not cover their complete table extent.
    Gap,
    /// Cell identities must be unique across the whole layout.
    DuplicateCell(TemplateCellId),
    /// Table identities must be unique within a layout.
    DuplicateTable(TemplateTableId),
    /// Saved template names must contain visible text.
    BlankName,
    /// Paper dimensions/margins are invalid.
    Paper(crate::SheetError),
}
impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bounds => {
                f.write_str("Template geometry is empty, overflowing or outside its owner")
            }
            Self::Graphic => f.write_str("Invalid template artwork: check identity, paper bounds, geometry and positive style sizes"),
            Self::Overlap => f.write_str("Template cells overlap"),
            Self::Gap => f.write_str("Template cells do not completely cover their table"),
            Self::DuplicateCell(id) => write!(f, "Template cell {id} is repeated"),
            Self::DuplicateTable(id) => write!(f, "Template table {id} is repeated"),
            Self::BlankName => f.write_str("Template name must not be blank"),
            Self::Paper(error) => error.fmt(f),
        }
    }
}
impl Error for TemplateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Paper(error) => Some(error),
            _ => None,
        }
    }
}
