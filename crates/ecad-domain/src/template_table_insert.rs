//! Coherent row/column insertion, preserving spans and stable existing cell identities.
use crate::{FrameRect, TableAxis, TableEditError, TemplateCell, TemplateCellId, TemplateTable};
use std::collections::BTreeSet;

/// Which table edge stays fixed when a band grows the outer extent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableGrowthAnchor {
    /// Keep the top/left position; grow toward the bottom/right.
    Start,
    /// Keep the bottom/right position; grow toward the top/left.
    End,
}
impl TemplateTable {
    /// Number of blank cells needed at an existing local boundary. Cells spanning the
    /// boundary extend rather than receiving a new cell. Outer insertions use adjacent divisions.
    /// # Errors
    /// Rejects coordinates which are not existing cell boundaries or lie outside the table.
    pub fn insertion_cell_count(&self, axis: TableAxis, at: u32) -> Result<usize, TableEditError> {
        Ok(self.insertion_pattern(axis, at)?.len())
    }
    /// Insert a positive local row/column band using caller-allocated IDs for its blank cells.
    /// Existing IDs/captions/bindings/order remain intact. New cells follow spatial order and
    /// are appended to presentation order. Spanning cells extend across the new band.
    /// The owner must separately validate paper fit. No existing cell is split implicitly.
    /// # Errors
    /// Rejects absent boundaries, nonpositive/overflowing geometry, unavailable anchor space,
    /// incorrect/reused IDs or invalid partitions. The source remains unchanged.
    pub fn insert_band(
        &self,
        axis: TableAxis,
        at: u32,
        size: u32,
        anchor: TableGrowthAnchor,
        fresh: &[TemplateCellId],
    ) -> Result<Self, TableEditError> {
        let pattern = self.insertion_pattern(axis, at)?;
        let mut used: BTreeSet<_> = self.cells().iter().map(|c| c.id).collect();
        if pattern.len() != fresh.len() || fresh.iter().any(|id| !used.insert(*id)) {
            return Err(TableEditError::Identities);
        }
        if size == 0 {
            return Err(TableEditError::Geometry);
        }
        let mut bounds = self.bounds();
        let (origin, extent) = match axis {
            TableAxis::X => (&mut bounds.x, &mut bounds.width),
            TableAxis::Y => (&mut bounds.y, &mut bounds.height),
        };
        *extent = extent.checked_add(size).ok_or(TableEditError::Geometry)?;
        if anchor == TableGrowthAnchor::End {
            *origin = origin.checked_sub(size).ok_or(TableEditError::Geometry)?;
        }
        let mut cells = self.cells().to_vec();
        for c in &mut cells {
            let (start, span) = match axis {
                TableAxis::X => (&mut c.bounds.x, &mut c.bounds.width),
                TableAxis::Y => (&mut c.bounds.y, &mut c.bounds.height),
            };
            if *start >= at {
                *start = start.checked_add(size).ok_or(TableEditError::Geometry)?;
            } else if *start + *span > at {
                *span = span.checked_add(size).ok_or(TableEditError::Geometry)?;
            }
        }
        for (mut rect, id) in pattern.into_iter().zip(fresh) {
            match axis {
                TableAxis::X => {
                    rect.x = at;
                    rect.width = size;
                }
                TableAxis::Y => {
                    rect.y = at;
                    rect.height = size;
                }
            }
            cells.push(TemplateCell {
                picture: None,
                text_styles: Default::default(),
                id: *id,
                bounds: rect,
                caption: String::new(),
                contents: vec![],
            });
        }
        Self::new(self.id(), bounds, cells).map_err(TableEditError::Partition)
    }
    fn insertion_pattern(
        &self,
        axis: TableAxis,
        at: u32,
    ) -> Result<Vec<FrameRect>, TableEditError> {
        let extent = match axis {
            TableAxis::X => self.bounds().width,
            TableAxis::Y => self.bounds().height,
        };
        let edge = |b: FrameRect| match axis {
            TableAxis::X => (b.x, b.width),
            TableAxis::Y => (b.y, b.height),
        };
        if at > extent
            || !self.cells().iter().any(|c| {
                let (p, n) = edge(c.bounds);
                p == at || p + n == at
            })
        {
            return Err(TableEditError::Geometry);
        }
        let mut pattern: Vec<_> = self
            .cells()
            .iter()
            .filter(|c| {
                let (p, n) = edge(c.bounds);
                if at == extent { p + n == at } else { p == at }
            })
            .map(|c| c.bounds)
            .collect();
        pattern.sort_by_key(|b| match axis {
            TableAxis::X => b.y,
            TableAxis::Y => b.x,
        });
        Ok(pattern)
    }
}
