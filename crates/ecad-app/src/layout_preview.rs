//! Immutable revision-bound preview projections backed by the actual validated layout commands.

use crate::*;
use ecad_domain::*;
use std::collections::BTreeSet;

/// Exact proposed drawing record; projections contain no authoritative mutable state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreviewObject {
    /// Selected annotation with its complete text and style intent.
    Label(DrawingLabel),
    /// Selected retained graphical path.
    Polyline(DrawingPolyline),
    /// Selected real-terminal placement, retaining its electrical binding.
    Anchor(TerminalAnchor),
    /// A route affected by selected anchor movement, included exactly once.
    Route(DrawingRoute),
}
/// Inspectable geometry and a closed command proposal at one session revision.
/// Dropping it cancels without changing history. It is transient and never serialized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutPreview {
    revision: Revision,
    view: ViewId,
    selection: BTreeSet<ObjectRef>,
    operation: LayoutOperation,
    plan: LayoutPlan,
    objects: Vec<PreviewObject>,
}
impl LayoutPreview {
    /// Exact originating session epoch/sequence required for commit, even for no-ops.
    pub fn revision(&self) -> Revision {
        self.revision
    }
    /// Single drawing view containing every selected origin.
    pub fn view(&self) -> ViewId {
        self.view
    }
    /// Deduplicated original selection, unaffected by later host focus changes.
    pub fn selection(&self) -> &BTreeSet<ObjectRef> {
        &self.selection
    }
    /// Explicit numeric policy already used to produce the proposed geometry.
    pub fn operation(&self) -> LayoutOperation {
        self.operation
    }
    /// Changed selected origins, distinct from affected unselected route geometry.
    pub fn changes(&self) -> &[LayoutChange] {
        &self.plan.changes
    }
    /// Selected origins that already matched the requested coordinates.
    pub fn unchanged(&self) -> &BTreeSet<ObjectRef> {
        &self.plan.unchanged
    }
    /// Union of affected routes, preserving the existing conductor identities.
    pub fn affected_routes(&self) -> &BTreeSet<RouteId> {
        &self.plan.affected_routes
    }
    /// Exact candidate records for all selected objects plus affected routes, in typed ID order.
    pub fn objects(&self) -> &[PreviewObject] {
        &self.objects
    }
}

/// Prepare exact preview geometry using the same validated commands that commit will execute.
/// No document/history/checkpoint mutation occurs; dropping the result is cancellation.
/// # Errors
/// Returns the original layout selection, overflow, or local-routing validation failure.
pub fn preview_layout(
    session: &Session,
    selection: impl IntoIterator<Item = ObjectRef>,
    operation: LayoutOperation,
) -> Result<LayoutPreview, LayoutError> {
    let selection: BTreeSet<_> = selection.into_iter().collect();
    let (view, _) = crate::layout::layout_origins(session.project(), selection.iter().copied())?;
    let (plan, candidate) = crate::layout::layout_plan_candidate(
        session.project(),
        selection.iter().copied(),
        operation,
    )?;
    let mut represented = selection.clone();
    represented.extend(plan.affected_routes.iter().copied().map(ObjectRef::Route));
    let drawing = candidate.drawing();
    let objects = represented
        .into_iter()
        .map(|target| match target {
            ObjectRef::Anchor(id) => PreviewObject::Anchor(
                drawing
                    .terminal_anchor(id)
                    .expect("validated anchor")
                    .clone(),
            ),
            ObjectRef::Label(id) => {
                PreviewObject::Label(drawing.label(id).expect("validated label").clone())
            }
            ObjectRef::Polyline(id) => {
                PreviewObject::Polyline(drawing.polyline(id).expect("validated path").clone())
            }
            ObjectRef::Route(id) => {
                PreviewObject::Route(drawing.route(id).expect("validated touching route").clone())
            }
            _ => unreachable!("validated layout selection"),
        })
        .collect();
    Ok(LayoutPreview {
        revision: session.revision(),
        view,
        selection,
        operation,
        plan,
        objects,
    })
}
impl Session {
    /// Commit the exact preview proposal as one operation, with no fresh snapping or targets.
    /// # Errors
    /// Rejects changed/different sessions (including no-op previews) or normal command failures.
    pub fn apply_layout_preview(&mut self, preview: LayoutPreview) -> Result<bool, EditError> {
        self.execute_batch(preview.revision, preview.plan.commands)
    }
}
impl<S: VersionedProjectStore> Document<S> {
    /// Commit a prepared layout through the document's normal history/checkpoint boundary.
    /// # Errors
    /// Rejects stale/different-session previews or normal command failures without mutation.
    pub fn apply_layout_preview(&mut self, preview: LayoutPreview) -> Result<bool, EditError> {
        self.execute_batch(preview.revision, preview.plan.commands)
    }
}
