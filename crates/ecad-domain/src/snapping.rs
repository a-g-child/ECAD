//! Explicit transient snapping inputs and ranked targets; snapping never creates connectivity.

use crate::{AnchorId, DrawingLength, DrawingPoint, ObjectRef, PolylineId, RouteId, ViewId};
use std::collections::BTreeSet;

/// Integer drawing grid, independent of viewport pixels and persisted style defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawingGrid {
    /// Exact reference point in document micrometres.
    pub origin: DrawingPoint,
    /// Positive equal x/y spacing in micrometres.
    pub spacing: DrawingLength,
}
/// Enabled snap sources and an inclusive circular document-space capture radius.
/// Hosts convert a screen-space tolerance explicitly; no zoom-dependent state lives here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SnapSettings {
    /// Inclusive radius in micrometres; zero allows exact coincidences only.
    pub tolerance_um: u32,
    /// Include placed real-terminal anchors.
    pub terminals: bool,
    /// Include the first and last vertices of routes and graphical paths.
    pub endpoints: bool,
    /// Include each segment's midpoint, rounded down independently on x/y.
    pub midpoints: bool,
    /// Include the nearest representable grid point, with half ties toward smaller coordinates.
    pub grid: Option<DrawingGrid>,
}
/// A query's exact geometry and scope. Exclusions refer to representations, not real wires.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapQuery {
    /// Existing drawing view; other views never supply candidates.
    pub view: ViewId,
    /// Unsnapped pointer coordinate in document micrometres.
    pub position: DrawingPoint,
    /// Explicit current snap policy; no hidden fallback or persisted constraint.
    pub settings: SnapSettings,
    /// Objects whose geometry must not attract the operation, such as a moving selection.
    /// Excluding an anchor also excludes route endpoint candidates attached to that anchor.
    pub excluded: BTreeSet<ObjectRef>,
}
/// Explicit route/path endpoint index, avoiding unchecked caller-supplied numeric ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SnapEnd {
    /// First ordered vertex.
    First,
    /// Last ordered vertex.
    Last,
}
/// Stable target identity within one queried document revision.
/// Queries rank source classes terminal, endpoint, midpoint, then grid; typed identity breaks distance ties within each class.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SnapTarget {
    /// Placed real-terminal identity; snapping to it does not attach a conductor.
    Terminal(AnchorId),
    /// Endpoint of a retained conductor route.
    RouteEndpoint {
        /// Existing route identity.
        route: RouteId,
        /// Explicit ordered endpoint.
        end: SnapEnd,
    },
    /// Endpoint of a graphical path.
    PolylineEndpoint {
        /// Existing path identity.
        polyline: PolylineId,
        /// Explicit ordered endpoint.
        end: SnapEnd,
    },
    /// Rounded midpoint of a route segment; indices are transient revision-local targets.
    RouteMidpoint {
        /// Existing route identity.
        route: RouteId,
        /// Zero-based segment index.
        segment: usize,
    },
    /// Rounded midpoint of a graphical segment.
    PolylineMidpoint {
        /// Existing path identity.
        polyline: PolylineId,
        /// Zero-based segment index.
        segment: usize,
    },
    /// Nearest representable point of the explicit grid.
    Grid,
}
impl SnapTarget {
    pub(crate) fn priority(self) -> u8 {
        match self {
            Self::Terminal(_) => 0,
            Self::RouteEndpoint { .. } | Self::PolylineEndpoint { .. } => 1,
            Self::RouteMidpoint { .. } | Self::PolylineMidpoint { .. } => 2,
            Self::Grid => 3,
        }
    }
}
/// A candidate suitable for visible feedback/cycling, sorted by source priority,
/// squared distance, then typed target identity. Coincident identities remain distinct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapCandidate {
    /// The actual reference chosen, independent of its formatted display label.
    pub target: SnapTarget,
    /// Exact snapped coordinate in document micrometres.
    pub position: DrawingPoint,
    /// Exact squared distance from the query point, in square micrometres.
    pub distance_squared_um: u128,
}
