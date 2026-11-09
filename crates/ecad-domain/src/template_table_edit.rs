//! Exact immutable table transformations. Every result revalidates the complete partition.
use crate::{
    FrameRect, TemplateCell, TemplateCellId, TemplateError, TemplateTable, TemplateTextRun,
};
use std::{collections::BTreeSet, error::Error, fmt};

/// Direction normal to a table boundary; coordinates are local integer micrometres.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableAxis {
    /// X coordinate of a vertical boundary.
    X,
    /// Y coordinate of a horizontal boundary.
    Y,
}
/// Explicit handling of captions and bindings removed by a merge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeContents {
    /// Keep only the designated surviving cell's caption and runs.
    KeepPrimary,
    /// Join nonempty captions/runs by newlines in top-to-bottom, left-to-right order.
    /// Field runs remain typed; no values are resolved or copied from a project.
    Combine,
}
/// Invalid structural edit; the source table always remains unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableEditError {
    /// Selection contains unavailable, repeated or insufficient cell identities.
    Selection,
    /// Selected cells do not form one complete rectangle.
    Rectangle,
    /// Counts/coordinates collapse cells, overflow or cross another boundary.
    Geometry,
    /// Fresh IDs must exactly cover new cells and cannot reuse existing identities.
    Identities,
    /// Complete partition validation failed.
    Partition(TemplateError),
}
impl fmt::Display for TableEditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Selection => f.write_str("Select existing distinct cells in one table"),
            Self::Rectangle => f.write_str("Merged cells must fill one rectangle without gaps"),
            Self::Geometry => f.write_str(
                "Table dimensions and boundaries must preserve positive cells and their order",
            ),
            Self::Identities => {
                f.write_str("Split requires fresh unique identities for every additional cell")
            }
            Self::Partition(e) => e.fmt(f),
        }
    }
}
impl Error for TableEditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Partition(e) => Some(e),
            _ => None,
        }
    }
}
impl TemplateTable {
    /// Divide one cell into a row-major grid. Original identity/content stays upper-left;
    /// all other cells start blank with caller-allocated IDs in row-major order.
    /// Integer remainders use floor-mapped boundaries, preserving the exact outer extent.
    /// # Errors
    /// Rejects zero/one-cell grids, collapsed cells, missing targets or reused/incorrect IDs.
    pub fn split_cell(
        &self,
        target: TemplateCellId,
        columns: u32,
        rows: u32,
        fresh: &[TemplateCellId],
    ) -> Result<Self, TableEditError> {
        let source = self
            .cells()
            .iter()
            .find(|c| c.id == target)
            .ok_or(TableEditError::Selection)?;
        let count = u64::from(columns) * u64::from(rows);
        if count < 2 || columns > source.bounds.width || rows > source.bounds.height {
            return Err(TableEditError::Geometry);
        }
        if u64::try_from(fresh.len()).ok() != Some(count - 1) {
            return Err(TableEditError::Identities);
        }
        let mut ids: BTreeSet<_> = self.cells().iter().map(|c| c.id).collect();
        if fresh.iter().any(|id| !ids.insert(*id)) {
            return Err(TableEditError::Identities);
        }
        let mut replacements = Vec::new();
        let mut next = fresh.iter();
        for row in 0..rows {
            for col in 0..columns {
                let mut cell = if row == 0 && col == 0 {
                    source.clone()
                } else {
                    TemplateCell {
                        text_styles: source.text_styles.clone(),
                        id: *next.next().expect("validated fresh count"),
                        bounds: source.bounds,
                        caption: String::new(),
                        contents: vec![],
                    }
                };
                let x = map(col, source.bounds.width, columns);
                let y = map(row, source.bounds.height, rows);
                cell.bounds = FrameRect {
                    x: source.bounds.x + x,
                    y: source.bounds.y + y,
                    width: map(col + 1, source.bounds.width, columns) - x,
                    height: map(row + 1, source.bounds.height, rows) - y,
                };
                replacements.push(cell);
            }
        }
        let mut cells = Vec::new();
        for cell in self.cells() {
            if cell.id == target {
                cells.extend(replacements.iter().cloned());
            } else {
                cells.push(cell.clone());
            }
        }
        Self::new(self.id(), self.bounds(), cells).map_err(TableEditError::Partition)
    }
    /// Merge a rectangular selection, retaining `primary` identity and its presentation slot.
    /// Content loss is permitted only through explicit KeepPrimary. Combine retains all typed
    /// bindings in spatial reading order, separated by literal newlines.
    /// # Errors
    /// Rejects duplicates, missing/insufficient selections, absent primary or nonrectangular union.
    pub fn merge_cells(
        &self,
        selected: &[TemplateCellId],
        primary: TemplateCellId,
        contents: MergeContents,
    ) -> Result<Self, TableEditError> {
        let ids: BTreeSet<_> = selected.iter().copied().collect();
        if ids.len() != selected.len() || ids.len() < 2 || !ids.contains(&primary) {
            return Err(TableEditError::Selection);
        }
        let mut chosen: Vec<_> = self
            .cells()
            .iter()
            .filter(|c| ids.contains(&c.id))
            .collect();
        if chosen.len() != ids.len() {
            return Err(TableEditError::Selection);
        }
        let x = chosen.iter().map(|c| c.bounds.x).min().expect("nonempty");
        let y = chosen.iter().map(|c| c.bounds.y).min().expect("nonempty");
        let right = chosen
            .iter()
            .map(|c| c.bounds.x + c.bounds.width)
            .max()
            .expect("validated bounds");
        let bottom = chosen
            .iter()
            .map(|c| c.bounds.y + c.bounds.height)
            .max()
            .expect("validated bounds");
        let area: u128 = chosen
            .iter()
            .map(|c| u128::from(c.bounds.width) * u128::from(c.bounds.height))
            .sum();
        if area != u128::from(right - x) * u128::from(bottom - y) {
            return Err(TableEditError::Rectangle);
        }
        let mut merged = chosen
            .iter()
            .find(|c| c.id == primary)
            .expect("selected primary")
            .to_owned()
            .clone();
        merged.bounds = FrameRect {
            x,
            y,
            width: right - x,
            height: bottom - y,
        };
        if contents == MergeContents::Combine {
            chosen.sort_by_key(|c| (c.bounds.y, c.bounds.x, c.id));
            merged.caption = chosen
                .iter()
                .filter(|c| !c.caption.is_empty())
                .map(|c| c.caption.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            merged.contents = vec![];
            for cell in chosen.iter().filter(|c| !c.contents.is_empty()) {
                if !merged.contents.is_empty() {
                    merged.contents.push(TemplateTextRun::Literal("\n".into()));
                }
                merged.contents.extend(cell.contents.clone());
            }
        }
        let cells = self
            .cells()
            .iter()
            .filter_map(|c| {
                if c.id == primary {
                    Some(merged.clone())
                } else if ids.contains(&c.id) {
                    None
                } else {
                    Some(c.clone())
                }
            })
            .collect();
        Self::new(self.id(), self.bounds(), cells).map_err(TableEditError::Partition)
    }
    /// Move/resize the whole table, mapping every shared edge once by exact floor ratios.
    /// Preserves IDs/content/order and spans. The owning layout must separately check paper fit.
    /// # Errors
    /// Rejects invalid extents or scaling that collapses any cell to zero size.
    pub fn resized(&self, bounds: FrameRect) -> Result<Self, TableEditError> {
        let old = self.bounds();
        let mut cells = self.cells().to_vec();
        for c in &mut cells {
            let b = c.bounds;
            let x = map(b.x, bounds.width, old.width);
            let y = map(b.y, bounds.height, old.height);
            c.bounds = FrameRect {
                x,
                y,
                width: map(b.x + b.width, bounds.width, old.width) - x,
                height: map(b.y + b.height, bounds.height, old.height) - y,
            };
        }
        Self::new(self.id(), bounds, cells).map_err(TableEditError::Partition)
    }
    /// Move every endpoint on one interior coordinate together. Spans across it stay intact.
    /// Other distinct coordinates keep their order; this is an aligned seam edit, not a row drag.
    /// # Errors
    /// Rejects absent/outer boundaries and crossing/collapsing coordinates or invalid partitions.
    pub fn move_boundary(
        &self,
        axis: TableAxis,
        from: u32,
        to: u32,
    ) -> Result<Self, TableEditError> {
        let extent = match axis {
            TableAxis::X => self.bounds().width,
            TableAxis::Y => self.bounds().height,
        };
        let mut edges = BTreeSet::new();
        for c in self.cells() {
            let (start, size) = match axis {
                TableAxis::X => (c.bounds.x, c.bounds.width),
                TableAxis::Y => (c.bounds.y, c.bounds.height),
            };
            edges.extend([start, start + size]);
        }
        if from == 0
            || from == extent
            || !edges.contains(&from)
            || to == 0
            || to >= extent
            || edges
                .iter()
                .any(|e| (*e < from && *e >= to) || (*e > from && *e <= to))
        {
            return Err(TableEditError::Geometry);
        }
        let mut cells = self.cells().to_vec();
        for c in &mut cells {
            let (start, size) = match axis {
                TableAxis::X => (&mut c.bounds.x, &mut c.bounds.width),
                TableAxis::Y => (&mut c.bounds.y, &mut c.bounds.height),
            };
            let end = *start + *size;
            let new_start = if *start == from { to } else { *start };
            let new_end = if end == from { to } else { end };
            *start = new_start;
            *size = new_end - new_start;
        }
        Self::new(self.id(), self.bounds(), cells).map_err(TableEditError::Partition)
    }
}
// Inputs are bounded by the source extent, so the quotient always fits u32.
fn map(value: u32, extent: u32, divisor: u32) -> u32 {
    u32::try_from(u64::from(value) * u64::from(extent) / u64::from(divisor)).expect("bounded ratio")
}
