//! Row-specific table repartitioning with stable surviving identities and exact outer bounds.
use crate::{FrameRect, TableEditError, TemplateCell, TemplateCellId, TemplateTable};
use std::collections::{BTreeMap, BTreeSet};

impl TemplateTable {
    fn layout_rows(&self) -> Vec<Vec<&TemplateCell>> {
        let mut rows = BTreeMap::<u32, Vec<&TemplateCell>>::new();
        for cell in self.cells() {
            rows.entry(cell.bounds.y).or_default().push(cell);
        }
        for row in rows.values_mut() {
            row.sort_by_key(|c| (c.bounds.x, c.id));
        }
        rows.into_values().collect()
    }
    /// Column counts grouped by cell top edge, in reading order. Spanning cells belong
    /// to their starting row; changing the layout converts spans into ordinary row cells.
    pub fn row_column_counts(&self) -> Vec<usize> {
        self.layout_rows().iter().map(Vec::len).collect()
    }
    /// Repartition within the existing outer bounds. Surviving row/column positions keep
    /// identities, captions, bindings and styles. Additional cells are blank/default styled.
    /// Unchanged counts are an exact no-op. With unchanged row count, row top edges stay;
    /// otherwise rows have equal heights. Changed rows have equal column widths. Unchanged
    /// ordinary rows keep their widths; spanning rows are normalized when the layout changes.
    /// Integer division distributes remainders without gaps. The caller supplies fresh IDs
    /// in reading order and must explicitly permit removal of surplus cells.
    /// # Errors
    /// Rejects empty/zero counts, more than 4096 output cells, collapsed geometry, incorrect
    /// or reused IDs, or unacknowledged removal. The source remains unchanged on failure.
    pub fn with_row_layout(
        &self,
        columns: &[u32],
        fresh: &[TemplateCellId],
        allow_removal: bool,
    ) -> Result<Self, TableEditError> {
        let bounds = self.bounds();
        let count: u64 = columns.iter().map(|n| u64::from(*n)).sum();
        if columns.is_empty()
            || count > 4096
            || columns.len() as u64 > u64::from(bounds.height)
            || columns.iter().any(|n| *n == 0 || *n > bounds.width)
        {
            return Err(TableEditError::Geometry);
        }
        let rows = self.layout_rows();
        let retained: usize = columns
            .iter()
            .enumerate()
            .map(|(i, n)| rows.get(i).map_or(0, |r| r.len().min(*n as usize)))
            .sum();
        if retained < self.cells().len() && !allow_removal {
            return Err(TableEditError::Selection);
        }
        let mut used: BTreeSet<_> = self.cells().iter().map(|c| c.id).collect();
        if fresh.len() as u64 != count - retained as u64 || fresh.iter().any(|id| !used.insert(*id))
        {
            return Err(TableEditError::Identities);
        }
        if columns
            .iter()
            .map(|n| *n as usize)
            .eq(rows.iter().map(Vec::len))
        {
            return Ok(self.clone());
        }
        let mut ids = fresh.iter();
        let mut cells = Vec::new();
        for (r, n) in columns.iter().enumerate() {
            let (y, end) = if columns.len() == rows.len() {
                (
                    rows[r][0].bounds.y,
                    rows.get(r + 1).map_or(bounds.height, |row| row[0].bounds.y),
                )
            } else {
                let edge =
                    |i: usize| (u64::from(bounds.height) * i as u64 / columns.len() as u64) as u32;
                (edge(r), edge(r + 1))
            };
            let row = rows.get(r);
            let preserve_widths = row.is_some_and(|row| {
                row.len() == *n as usize
                    && row.first().is_some_and(|c| c.bounds.x == 0)
                    && row
                        .windows(2)
                        .all(|pair| pair[0].bounds.x + pair[0].bounds.width == pair[1].bounds.x)
                    && row
                        .last()
                        .is_some_and(|c| c.bounds.x + c.bounds.width == bounds.width)
            });
            for c in 0..*n {
                let source = row.and_then(|row| row.get(c as usize));
                let mut cell = source
                    .map(|cell| (*cell).clone())
                    .unwrap_or_else(|| TemplateCell {
                        id: *ids.next().expect("validated fresh count"),
                        bounds,
                        caption: String::new(),
                        contents: vec![],
                        text_styles: Default::default(),
                    });
                let edge = |i: u32| (u64::from(bounds.width) * u64::from(i) / u64::from(*n)) as u32;
                let (x, width) = if preserve_widths {
                    (cell.bounds.x, cell.bounds.width)
                } else {
                    (edge(c), edge(c + 1) - edge(c))
                };
                cell.bounds = FrameRect {
                    x,
                    y,
                    width,
                    height: end - y,
                };
                cells.push(cell);
            }
        }
        Self::new(self.id(), bounds, cells).map_err(TableEditError::Partition)
    }
}
