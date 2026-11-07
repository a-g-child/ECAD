//! Whole conductor routes and independently placed real-terminal anchors.

use crate::{
    AnchorId, Appearance, ConductorId, DrawingPoint, RouteError, RouteId, TerminalId, ViewId,
};

/// A placed representation of an existing terminal, not a duplicate electrical anchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalAnchor {
    /// Stable placed-anchor identity; one terminal may have several representations.
    pub id: AnchorId,
    /// Drawing view containing this placement.
    pub view: ViewId,
    /// Actual device terminal whose electrical identity remains authoritative.
    pub terminal: TerminalId,
    /// Exact document location in micrometres.
    pub position: DrawingPoint,
}
/// Explicit geometry policy, independent of conductor connectivity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteMode {
    /// Every segment is horizontal or vertical; invalid edits are rejected.
    Orthogonal,
    /// Ordered nonzero straight segments with arbitrary angles.
    Free,
}
/// One retained route representing one conductor in one view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingRoute {
    /// Stable route identity, independent of its constituent segments.
    pub id: RouteId,
    /// View-local representation scope.
    pub view: ViewId,
    /// The single existing electrical conductor represented here.
    pub conductor: ConductorId,
    /// A/B placed terminal anchors; `None` corresponds to a genuinely free electrical end.
    pub anchors: [Option<AnchorId>; 2],
    /// Ordered complete path, with first/last points matching attached anchor positions.
    pub points: Vec<DrawingPoint>,
    /// Explicit geometry policy.
    pub mode: RouteMode,
    /// Stroke-role appearance, independent of physical conductor attributes.
    pub appearance: Appearance,
}
impl DrawingRoute {
    /// Validate whole-path geometry without guessing or changing electrical connections.
    /// # Errors
    /// Rejects fewer than two points, adjacent duplicates, or diagonal orthogonal segments.
    pub fn validate_geometry(&self) -> Result<(), RouteError> {
        if self.points.len() < 2 || self.points.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(RouteError::InvalidGeometry(self.id));
        }
        if self.mode == RouteMode::Orthogonal
            && self
                .points
                .windows(2)
                .any(|pair| pair[0].x != pair[1].x && pair[0].y != pair[1].y)
        {
            return Err(RouteError::InvalidGeometry(self.id));
        }
        Ok(())
    }
}
/// Exact local layout snapshot for moving an anchor and all touching routes coherently.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchorLayout {
    /// One anchor's complete placement; view/terminal identity cannot change during restoration.
    pub anchor: TerminalAnchor,
    /// Exactly the routes touching this anchor, in stable identity order.
    pub routes: Vec<DrawingRoute>,
}
/// Deterministic geometry-only route editing, with explicit vertex/segment indexing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteEdit {
    /// Set one vertex; attached endpoints cannot move independently of their anchor.
    MoveVertex {
        /// Zero-based whole-path vertex index.
        index: usize,
        /// Explicit document coordinate.
        position: DrawingPoint,
    },
    /// Insert a waypoint before an existing noninitial vertex.
    InsertVertex {
        /// Insertion index, from 1 through current length minus one.
        index: usize,
        /// Explicit document coordinate; geometry policy still applies.
        position: DrawingPoint,
    },
    /// Remove an interior waypoint only.
    RemoveVertex {
        /// Interior vertex index.
        index: usize,
    },
    /// Drag an orthogonal segment perpendicular to its current direction.
    MoveSegment {
        /// Zero-based segment index, addressing vertices index and index + 1.
        index: usize,
        /// Absolute y for a horizontal segment, or x for a vertical segment, in micrometres.
        coordinate_um: i64,
    },
}
