//! Exact orthogonal document transforms with explicit pivots and checked integer coordinates.

use crate::{DrawingError, DrawingPoint};

/// Exact rotation in document coordinates, independent of a viewport's y-axis convention.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuarterTurn {
    /// Map a pivot-relative (x, y) to (-y, x).
    Positive90,
    /// Map a pivot-relative (x, y) to (-x, -y).
    Half,
    /// Map a pivot-relative (x, y) to (y, -x).
    Negative90,
}
/// Explicit line of reflection passing through the supplied pivot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MirrorAxis {
    /// Reflect x about the vertical line x = pivot.x; retain y.
    Vertical,
    /// Reflect y about the horizontal line y = pivot.y; retain x.
    Horizontal,
}
/// Orthogonal geometry operation; does not alter electrical identities, A/B order, or text glyphs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawingTransform {
    /// Rotate every geometry point around one explicit document pivot.
    Rotate {
        /// Exact document point in micrometres.
        pivot: DrawingPoint,
        /// Exact quarter-turn policy.
        turn: QuarterTurn,
    },
    /// Reflect every geometry point across a pivot-defined axis.
    Mirror {
        /// Defines the line's constant coordinate in micrometres.
        pivot: DrawingPoint,
        /// Explicit horizontal/vertical line of reflection.
        axis: MirrorAxis,
    },
}
impl DrawingTransform {
    /// Transform an exact coordinate without rounding, wrapping, or saturating.
    /// Intermediate offsets use wider arithmetic, so a large pivot-relative distance
    /// is allowed whenever the final coordinates fit the document range.
    /// # Errors
    /// Rejects a resulting coordinate outside i64 before returning any point.
    pub fn point(self, point: DrawingPoint) -> Result<DrawingPoint, DrawingError> {
        let pivot = match self {
            Self::Rotate { pivot, .. } | Self::Mirror { pivot, .. } => pivot,
        };
        let x = i128::from(point.x) - i128::from(pivot.x);
        let y = i128::from(point.y) - i128::from(pivot.y);
        let (x, y) = match self {
            Self::Rotate {
                turn: QuarterTurn::Positive90,
                ..
            } => (-y, x),
            Self::Rotate {
                turn: QuarterTurn::Half,
                ..
            } => (-x, -y),
            Self::Rotate {
                turn: QuarterTurn::Negative90,
                ..
            } => (y, -x),
            Self::Mirror {
                axis: MirrorAxis::Vertical,
                ..
            } => (-x, y),
            Self::Mirror {
                axis: MirrorAxis::Horizontal,
                ..
            } => (x, -y),
        };
        Ok(DrawingPoint {
            x: i64::try_from(i128::from(pivot.x) + x)
                .map_err(|_| DrawingError::CoordinateOverflow)?,
            y: i64::try_from(i128::from(pivot.y) + y)
                .map_err(|_| DrawingError::CoordinateOverflow)?,
        })
    }
}
