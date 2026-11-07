//! Exact localized route/anchor history; moving back is not a substitute for exact undo.

use ecad_domain::*;

/// Committed whole-route or placed-terminal layout operation, independent of UI tools.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteCommand {
    /// Rotate or mirror selected terminal placements and complete internally connected routes.
    TransformAnchors {
        /// Deduplicated existing terminal placements; route bindings never change.
        anchors: std::collections::BTreeSet<AnchorId>,
        /// Exact pivot and orthogonal operation.
        transform: DrawingTransform,
    },
    /// Move several anchors together; each affected route is adjusted once.
    MoveAnchors(std::collections::BTreeMap<AnchorId, DrawingPoint>),
    /// Restore exact geometry for several anchors and their touching route union.
    RestoreLayout(RouteLayout),
    /// Create a representation using the current route creation preset.
    Create {
        /// Fresh whole-route identity.
        id: RouteId,
        /// Existing drawing view.
        view: ViewId,
        /// Existing electrical conductor; this operation never creates a duplicate wire.
        conductor: ConductorId,
        /// Explicit A/B placed anchors, matching that conductor's attachments.
        anchors: [Option<AnchorId>; 2],
        /// Explicit ordered document geometry.
        points: Vec<DrawingPoint>,
        /// Geometry policy.
        mode: RouteMode,
    },
    /// Place a real terminal in a drawing view.
    InsertAnchor(TerminalAnchor),
    /// Remove an unused placement.
    RemoveAnchor(AnchorId),
    /// Move a placement with deterministic local connection-preserving route adjustment.
    MoveAnchor {
        /// Existing placed-anchor identity.
        anchor: AnchorId,
        /// Explicit document position.
        position: DrawingPoint,
    },
    /// Restore exact local layout rather than re-running movement in reverse.
    RestoreAnchorLayout(AnchorLayout),
    /// Insert exact whole-route state with electrical continuity validation.
    Insert(DrawingRoute),
    /// Explicitly replace whole-route state after validation.
    Replace(DrawingRoute),
    /// Remove a representation without changing its electrical conductor.
    Remove(RouteId),
    /// Perform an indexed vertex/segment edit on one retained whole path.
    Edit {
        /// Existing route identity.
        route: RouteId,
        /// Deterministic geometry-only operation.
        edit: RouteEdit,
    },
}
impl RouteCommand {
    pub(crate) fn apply(&self, project: &mut Project) -> Result<Option<Self>, ModelError> {
        let inverse = match self {
            Self::TransformAnchors { anchors, transform } => {
                let old = project.transform_terminal_anchors(anchors.clone(), *transform)?;
                if old == project.drawing().route_layout(anchors.iter().copied())? {
                    return Ok(None);
                }
                Self::RestoreLayout(old)
            }
            Self::MoveAnchors(positions) => {
                let old = project.move_terminal_anchors(positions.clone())?;
                if old.anchors.iter().all(|a| positions[&a.id] == a.position) {
                    return Ok(None);
                }
                Self::RestoreLayout(old)
            }
            Self::RestoreLayout(layout) => {
                let old = project.restore_route_layout(layout.clone())?;
                if old == *layout {
                    return Ok(None);
                }
                Self::RestoreLayout(old)
            }
            Self::Create {
                id,
                view,
                conductor,
                anchors,
                points,
                mode,
            } => {
                project.insert_drawing_route(DrawingRoute {
                    id: *id,
                    view: *view,
                    conductor: *conductor,
                    anchors: *anchors,
                    points: points.clone(),
                    mode: *mode,
                    appearance: project.drawing().settings().route_creation.clone(),
                })?;
                Self::Remove(*id)
            }
            Self::InsertAnchor(anchor) => {
                project.insert_terminal_anchor(anchor.clone())?;
                Self::RemoveAnchor(anchor.id)
            }
            Self::RemoveAnchor(id) => Self::InsertAnchor(project.remove_terminal_anchor(*id)?),
            Self::MoveAnchor { anchor, position } => {
                let old = project.move_terminal_anchor(*anchor, *position)?;
                if old.anchor.position == *position {
                    return Ok(None);
                }
                Self::RestoreAnchorLayout(old)
            }
            Self::RestoreAnchorLayout(layout) => {
                let old = project.restore_anchor_layout(layout.clone())?;
                if old == *layout {
                    return Ok(None);
                }
                Self::RestoreAnchorLayout(old)
            }
            Self::Insert(route) => {
                project.insert_drawing_route(route.clone())?;
                Self::Remove(route.id)
            }
            Self::Replace(route) => {
                let old = project.replace_drawing_route(route.clone())?;
                if old == *route {
                    return Ok(None);
                }
                Self::Replace(old)
            }
            Self::Remove(id) => Self::Insert(project.remove_drawing_route(*id)?),
            Self::Edit { route, edit } => {
                let current = project
                    .drawing()
                    .route(*route)
                    .ok_or(RouteError::MissingRoute(*route))?;
                let changed = current.edited(edit)?;
                if &changed == current {
                    return Ok(None);
                }
                Self::Replace(project.replace_drawing_route(changed)?)
            }
        };
        Ok(Some(inverse))
    }
}
