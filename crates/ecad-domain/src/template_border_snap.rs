//! Exact pointer placement against a template's margin border; no renderer or document mutation.
use crate::{FrameRect, TableEditError};

/// A table's outer edge and corresponding paper margin border.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarginEdge {
    /// Left table/border edge.
    Left,
    /// Right table/border edge.
    Right,
    /// Top table/border edge.
    Top,
    /// Bottom table/border edge.
    Bottom,
}
/// Pointer proposal in exact paper micrometres. Signed inputs allow approaching a border from outside.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableBorderDrag {
    /// Translate the whole table to this paper-space origin, retaining its dimensions.
    Move {
        /// Proposed left coordinate in micrometres.
        x: i64,
        /// Proposed top coordinate in micrometres.
        y: i64,
    },
    /// Move one outer edge, retaining the opposite edge and the other axis.
    Resize {
        /// Captured outer edge being dragged.
        edge: MarginEdge,
        /// Proposed paper-space coordinate along that edge's normal, in micrometres.
        coordinate: i64,
    },
}
/// Deterministic geometric snap result, separate from table partition validation/history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableBorderPlacement {
    /// Exact positive table extent contained in the margin border.
    pub bounds: FrameRect,
    /// Applied corresponding borders, in X then Y order. Ties prefer left/top.
    pub snapped: Vec<MarginEdge>,
}
/// Place a retained table using explicit paper-space snap tolerance. `None` bypasses snapping;
/// zero snaps only exact coincidences. Translation chooses the nearest corresponding edge per
/// axis; resizing snaps only the dragged outer edge. No grid or unrelated object is consulted.
/// The caller validates the resized cell partition before committing a closed proposal.
/// # Errors
/// Rejects empty/overflowing source or border geometry and unrepresentable, collapsed or
/// out-of-border results, including bypassed positions outside the paper interior.
pub fn table_border_placement(
    table: FrameRect,
    border: FrameRect,
    drag: TableBorderDrag,
    tolerance: Option<u32>,
) -> Result<TableBorderPlacement, TableEditError> {
    let valid = |b: FrameRect| {
        b.width > 0
            && b.height > 0
            && u64::from(b.x) + u64::from(b.width) <= u64::from(u32::MAX)
            && u64::from(b.y) + u64::from(b.height) <= u64::from(u32::MAX)
    };
    if !valid(table) || !valid(border) {
        return Err(TableEditError::Geometry);
    }
    let mut x = i128::from(table.x);
    let mut y = i128::from(table.y);
    let mut right = x + i128::from(table.width);
    let mut bottom = y + i128::from(table.height);
    let bx = i128::from(border.x);
    let by = i128::from(border.y);
    let br = bx + i128::from(border.width);
    let bb = by + i128::from(border.height);
    let mut snapped = vec![];
    let close = |a: i128, b: i128| tolerance.is_some_and(|t| (a - b).abs() <= i128::from(t));
    match drag {
        TableBorderDrag::Move {
            x: proposed_x,
            y: proposed_y,
        } => {
            x = i128::from(proposed_x);
            y = i128::from(proposed_y);
            right = x + i128::from(table.width);
            bottom = y + i128::from(table.height);
            let horizontal = [(bx - x, MarginEdge::Left), (br - right, MarginEdge::Right)];
            let vertical = [(by - y, MarginEdge::Top), (bb - bottom, MarginEdge::Bottom)];
            if let Some((delta, edge)) = horizontal
                .into_iter()
                .filter(|(d, _)| close(*d, 0))
                .min_by_key(|(d, _)| d.abs())
            {
                x += delta;
                right += delta;
                snapped.push(edge);
            }
            if let Some((delta, edge)) = vertical
                .into_iter()
                .filter(|(d, _)| close(*d, 0))
                .min_by_key(|(d, _)| d.abs())
            {
                y += delta;
                bottom += delta;
                snapped.push(edge);
            }
        }
        TableBorderDrag::Resize { edge, coordinate } => {
            let target = match edge {
                MarginEdge::Left => bx,
                MarginEdge::Right => br,
                MarginEdge::Top => by,
                MarginEdge::Bottom => bb,
            };
            let mut value = i128::from(coordinate);
            if close(value, target) {
                value = target;
                snapped.push(edge);
            }
            match edge {
                MarginEdge::Left => x = value,
                MarginEdge::Right => right = value,
                MarginEdge::Top => y = value,
                MarginEdge::Bottom => bottom = value,
            }
        }
    }
    if x < bx || y < by || right > br || bottom > bb || right <= x || bottom <= y {
        return Err(TableEditError::Geometry);
    }
    let checked = |v| u32::try_from(v).map_err(|_| TableEditError::Geometry);
    Ok(TableBorderPlacement {
        bounds: FrameRect {
            x: checked(x)?,
            y: checked(y)?,
            width: checked(right - x)?,
            height: checked(bottom - y)?,
        },
        snapped,
    })
}
