//! Explicit route/layout conflicts; no silent detachment or electrical remapping.

use crate::{AnchorId, ConductorId, DeviceId, RouteId};
use std::{error::Error, fmt};

/// Route identity, attachment, or geometry validation failed before committing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteError {
    /// Internal derived route ownership indexes disagree with stored records.
    InvalidIndexState,
    /// Placed anchor is unavailable.
    MissingAnchor(AnchorId),
    /// Placed anchor identity already exists.
    DuplicateAnchor(AnchorId),
    /// Anchor still has touching routes.
    AnchorInUse(AnchorId),
    /// Whole route is unavailable.
    MissingRoute(RouteId),
    /// Whole route identity already exists.
    DuplicateRoute(RouteId),
    /// One conductor may have at most one whole route per view.
    RepeatedRepresentation(ConductorId),
    /// Path violates minimum/nonzero-segment/orthogonal requirements.
    InvalidGeometry(RouteId),
    /// Route ends must match conductor attachments and same-view anchor positions.
    AttachmentConflict(RouteId),
    /// Removing/changing this conductor requires explicit route resolution first.
    ConductorInUse(ConductorId),
    /// Device terminals still have view-local anchor placements.
    DeviceAnchored(DeviceId),
    /// Edit index is outside its allowed vertex/segment range.
    InvalidIndex(RouteId),
    /// Moving an attached endpoint requires moving its placed anchor.
    AttachedEndpoint(RouteId),
    /// Perpendicular segment dragging is defined only for orthogonal routes.
    OrthogonalRequired(RouteId),
    /// An exact layout snapshot has missing/extra/rebound routes or anchor identity changes.
    LayoutMismatch(AnchorId),
}
impl fmt::Display for RouteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "route/layout conflict: {self:?}")
    }
}
impl Error for RouteError {}
