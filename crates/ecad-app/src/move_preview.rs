//! Resolve an explicit selected origin to exact or snapped placement, then preview real layout.

use crate::*;
use ecad_domain::*;
use std::{error::Error, fmt};

/// Transient host choice within the current ranked candidate list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapChoice {
    /// Use the highest-ranked candidate, or the unsnapped point when none is available.
    Best,
    /// Use an explicit zero-based candidate; stale/out-of-range cycling indices are errors.
    Candidate(usize),
    /// Temporarily suppress snapping entirely, without changing saved preferences.
    Suppressed,
}
/// Numeric input always takes precedence over snapping; pointer input uses explicit settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovePlacement {
    /// Exact destination of the explicitly selected reference origin.
    Exact(DrawingPoint),
    /// Pointer destination with explicit document-space snap settings and cycling choice.
    Pointer {
        /// Unsnapped coordinate in micrometres.
        position: DrawingPoint,
        /// Current host-supplied snapping policy.
        settings: SnapSettings,
        /// Candidate selection or temporary suppression.
        choice: SnapChoice,
    },
}
/// Failure to resolve a move preview, preserving the underlying layout/model error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MovePreviewError {
    /// Original layout validation failure.
    Layout(LayoutError),
    /// Reference origin must belong to the explicit selection; no hover fallback is inferred.
    ReferenceNotSelected(ObjectRef),
    /// Explicit candidate cycling index is unavailable in this query.
    CandidateOutOfRange {
        /// Requested zero-based index.
        index: usize,
        /// Actual ranked candidate count.
        count: usize,
    },
}
impl fmt::Display for MovePreviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Layout(error) => error.fmt(f),
            Self::ReferenceNotSelected(target) => {
                write!(f, "move reference {target:?} is not selected")
            }
            Self::CandidateOutOfRange { index, count } => write!(
                f,
                "snap candidate {index} is unavailable; {count} candidates exist"
            ),
        }
    }
}
impl Error for MovePreviewError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Layout(error) => Some(error),
            _ => None,
        }
    }
}
impl From<LayoutError> for MovePreviewError {
    fn from(error: LayoutError) -> Self {
        Self::Layout(error)
    }
}
/// Closed move proposal with exact preview geometry, candidates, and original revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovePreview {
    layout: LayoutPreview,
    reference: ObjectRef,
    destination: DrawingPoint,
    candidates: Vec<SnapCandidate>,
    chosen: Option<usize>,
}
impl MovePreview {
    /// Exact revision-bound geometry and captured selection proposed for commit.
    pub fn layout(&self) -> &LayoutPreview {
        &self.layout
    }
    /// Explicit selected origin used as the placement reference.
    pub fn reference(&self) -> ObjectRef {
        self.reference
    }
    /// Final numeric/snapped/suppressed destination, never recalculated during commit.
    pub fn destination(&self) -> DrawingPoint {
        self.destination
    }
    /// Ranked targets available for visible feedback/cycling; numeric/suppressed inputs yield none.
    pub fn candidates(&self) -> &[SnapCandidate] {
        &self.candidates
    }
    /// Chosen target and coordinate, or `None` for unsnapped/numeric movement.
    pub fn chosen(&self) -> Option<&SnapCandidate> {
        self.chosen.map(|index| &self.candidates[index])
    }
    /// Consume the move's exact layout proposal for normal document/session commit.
    pub fn into_layout(self) -> LayoutPreview {
        self.layout
    }
}

/// Resolve one selected origin and preview movement without mutating the document.
///
/// Excludes the entire selected geometry and routes touching selected anchors from
/// snapping, so a moving route cannot attract itself. Other views are ignored.
/// Snapping changes coordinates only: no terminal binding or junction is created.
/// # Errors
/// Rejects unsupported selections/references, invalid cycling indices, i64 translation
/// offset overflow, or normal local-layout validation failures. Invalid winners do not
/// silently fall back to another candidate; the host can explicitly cycle or suppress.
pub fn preview_move(
    session: &Session,
    selection: impl IntoIterator<Item = ObjectRef>,
    reference: ObjectRef,
    placement: MovePlacement,
) -> Result<MovePreview, MovePreviewError> {
    let selection: std::collections::BTreeSet<_> = selection.into_iter().collect();
    let (view, origins) =
        crate::layout::layout_origins(session.project(), selection.iter().copied())?;
    let origin = *origins
        .get(&reference)
        .ok_or(MovePreviewError::ReferenceNotSelected(reference))?;
    let (destination, candidates, chosen) = match placement {
        MovePlacement::Exact(position)
        | MovePlacement::Pointer {
            position,
            choice: SnapChoice::Suppressed,
            ..
        } => (position, vec![], None),
        MovePlacement::Pointer {
            position,
            settings,
            choice,
        } => {
            let mut excluded = selection.clone();
            let anchors = selection.iter().filter_map(|target| match target {
                ObjectRef::Anchor(id) => Some(*id),
                _ => None,
            });
            let touching = session
                .project()
                .drawing()
                .route_layout(anchors)
                .map_err(LayoutError::from)?;
            excluded.extend(touching.routes.iter().map(|r| ObjectRef::Route(r.id)));
            let candidates = session
                .project()
                .drawing()
                .snap_candidates(&SnapQuery {
                    view,
                    position,
                    settings,
                    excluded,
                })
                .map_err(LayoutError::from)?;
            let chosen = match choice {
                SnapChoice::Best => (!candidates.is_empty()).then_some(0),
                SnapChoice::Candidate(index) if index < candidates.len() => Some(index),
                SnapChoice::Candidate(index) => {
                    return Err(MovePreviewError::CandidateOutOfRange {
                        index,
                        count: candidates.len(),
                    });
                }
                SnapChoice::Suppressed => unreachable!("suppression bypasses query"),
            };
            (
                chosen.map_or(position, |index| candidates[index].position),
                candidates,
                chosen,
            )
        }
    };
    let delta = DrawingPoint {
        x: i64::try_from(i128::from(destination.x) - i128::from(origin.x))
            .map_err(|_| LayoutError::from(DrawingError::CoordinateOverflow))?,
        y: i64::try_from(i128::from(destination.y) - i128::from(origin.y))
            .map_err(|_| LayoutError::from(DrawingError::CoordinateOverflow))?,
    };
    let layout = preview_layout(session, selection, LayoutOperation::Translate(delta))?;
    Ok(MovePreview {
        layout,
        reference,
        destination,
        candidates,
        chosen,
    })
}
