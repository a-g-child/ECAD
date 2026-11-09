//! Shared selection predicates and atomic column splitting, independent of editor widgets.
use crate::{TableEditError, TemplateCell, TemplateCellId, TemplateTable};
use std::collections::BTreeSet;
impl TemplateTable {
    /// Resolve a single cell or contiguous vertical column with identical left/right edges.
    /// Returns cells in top-to-bottom order, irrespective of selection order.
    /// # Errors
    /// Rejects missing/duplicate cells, gaps, or differing horizontal extents.
    pub fn split_column(
        &self,
        selected: &[TemplateCellId],
    ) -> Result<Vec<&TemplateCell>, TableEditError> {
        let ids: BTreeSet<_> = selected.iter().copied().collect();
        if ids.is_empty() || ids.len() != selected.len() {
            return Err(TableEditError::Selection);
        }
        let mut cells: Vec<_> = self
            .cells()
            .iter()
            .filter(|c| ids.contains(&c.id))
            .collect();
        if cells.len() != ids.len() {
            return Err(TableEditError::Selection);
        }
        cells.sort_by_key(|c| (c.bounds.y, c.bounds.x, c.id));
        if cells.windows(2).any(|pair| {
            let a = pair[0].bounds;
            let b = pair[1].bounds;
            a.x != b.x || a.width != b.width || a.y + a.height != b.y
        }) {
            return Err(TableEditError::Rectangle);
        }
        Ok(cells)
    }
    /// Split each selected column cell, retaining each original identity and contents.
    /// Fresh IDs are consumed top-to-bottom, then row-major within each source cell.
    /// The source remains unchanged on failure, including a failure in a later cell.
    /// # Errors
    /// Rejects invalid column selections, collapsed subdivisions or incorrect/failing IDs.
    pub fn split_cells(
        &self,
        selected: &[TemplateCellId],
        columns: u32,
        rows: u32,
        fresh: &[TemplateCellId],
    ) -> Result<Self, TableEditError> {
        let cells = self.split_column(selected)?;
        let per = u64::from(columns) * u64::from(rows);
        if per < 2 {
            return Err(TableEditError::Geometry);
        }
        let per = usize::try_from(per - 1).map_err(|_| TableEditError::Geometry)?;
        if per.checked_mul(cells.len()) != Some(fresh.len()) {
            return Err(TableEditError::Identities);
        }
        let mut result = self.clone();
        for (cell, ids) in cells.iter().zip(fresh.chunks_exact(per)) {
            result = result.split_cell(cell.id, columns, rows, ids)?;
        }
        Ok(result)
    }
}
