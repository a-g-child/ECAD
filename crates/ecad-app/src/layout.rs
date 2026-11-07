//! Deterministic one-shot origin layout, independent of renderer bounds and UI tools.

use crate::{Command, DrawingCommand, RouteCommand};
use ecad_domain::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

/// Document axis; coordinates increase according to the drawing's coordinate system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutAxis {
    /// Horizontal x coordinate.
    X,
    /// Vertical y coordinate.
    Y,
}
impl LayoutAxis {
    fn coordinate(self, point: DrawingPoint) -> i64 {
        match self {
            Self::X => point.x,
            Self::Y => point.y,
        }
    }
    fn set(self, point: &mut DrawingPoint, coordinate: i64) {
        match self {
            Self::X => point.x = coordinate,
            Self::Y => point.y = coordinate,
        }
    }
}

/// Explicit one-shot layout policy; no persistent constraint or implicit grid rounding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutOperation {
    /// Rotate or mirror complete selected graphical geometry around an explicit pivot.
    /// Annotations transform their position only; glyphs remain upright.
    Transform(DrawingTransform),
    /// Translate every selected origin by the same signed micrometre offset.
    Translate(DrawingPoint),
    /// Align selected origins to an explicit numeric coordinate, preserving the other axis.
    Align {
        /// Axis to change.
        axis: LayoutAxis,
        /// Exact coordinate in micrometres.
        coordinate_um: i64,
    },
    /// Space origins between their current extrema, retaining extrema and the other axis.
    /// Sort by coordinate then typed object identity; round fractional offsets down.
    /// Gaps differ by at most one micrometre. Requires three distinct selected objects.
    Distribute(LayoutAxis),
}

/// Invalid layout selection or geometry, preserving the original model error when present.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayoutError {
    /// Select at least one supported object.
    EmptySelection,
    /// Selection spans more than one drawing view.
    MixedViews,
    /// Select anchors to move attached routes; electrical objects have no drawing origin.
    UnsupportedTarget(ObjectRef),
    /// Distribution needs at least three distinct objects.
    TooFewObjects,
    /// Original missing-reference, coordinate, or route validation failure.
    Model(ModelError),
}
impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySelection => f.write_str("select drawing objects to arrange"),
            Self::MixedViews => f.write_str("arrange objects within one drawing view"),
            Self::UnsupportedTarget(target) => write!(
                f,
                "{target:?} has no supported layout origin; move connected routes through their anchors"
            ),
            Self::TooFewObjects => {
                f.write_str("distribution requires at least three distinct objects")
            }
            Self::Model(error) => error.fmt(f),
        }
    }
}
impl Error for LayoutError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Model(error) => Some(error),
            _ => None,
        }
    }
}
impl From<ModelError> for LayoutError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}
impl From<DrawingError> for LayoutError {
    fn from(error: DrawingError) -> Self {
        Self::Model(error.into())
    }
}

/// One explicitly selected origin's proposed change, suitable for preview/review.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutChange {
    /// Existing annotation, graphical path, or terminal placement.
    pub target: ObjectRef,
    /// Original origin in micrometres.
    pub before: DrawingPoint,
    /// Proposed origin in micrometres.
    pub after: DrawingPoint,
}
/// Preflighted layout proposal. Submit commands as one batch at the observed session revision.
/// Previewing uses a temporary candidate; history retains only affected object geometry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutPlan {
    /// Deterministic commands with localized exact inverses; no-ops are omitted.
    pub commands: Vec<Command>,
    /// Changed selected origins in typed identity order.
    pub changes: Vec<LayoutChange>,
    /// Selected records whose actual geometry changes, including paths whose origin stays fixed.
    pub geometry_changed: BTreeSet<ObjectRef>,
    /// Selected records whose own geometry is unchanged; touching routes can still change.
    pub unchanged: BTreeSet<ObjectRef>,
    /// Union of connected routes affected by selected anchor movement, each counted once.
    pub affected_routes: BTreeSet<RouteId>,
}

/// Read supported layout origins, enforcing a single view and deduplicating identities.
/// An annotation uses its position, a path its first vertex, and an anchor its position.
pub(crate) fn layout_origins(
    project: &Project,
    selection: impl IntoIterator<Item = ObjectRef>,
) -> Result<(ViewId, BTreeMap<ObjectRef, DrawingPoint>), LayoutError> {
    let mut view = None;
    let mut origins = BTreeMap::new();
    let drawing = project.drawing();
    for target in selection.into_iter().collect::<BTreeSet<_>>() {
        project.require_object(target)?;
        let (current_view, origin) = match target {
            ObjectRef::Label(id) => {
                let item = drawing.label(id).expect("required label");
                (item.view, item.position)
            }
            ObjectRef::Polyline(id) => {
                let item = drawing.polyline(id).expect("required path");
                (item.view, item.points[0])
            }
            ObjectRef::Anchor(id) => {
                let item = drawing.terminal_anchor(id).expect("required anchor");
                (item.view, item.position)
            }
            _ => return Err(LayoutError::UnsupportedTarget(target)),
        };
        if view.is_some_and(|previous| previous != current_view) {
            return Err(LayoutError::MixedViews);
        }
        view = Some(current_view);
        origins.insert(target, origin);
    }
    Ok((view.ok_or(LayoutError::EmptySelection)?, origins))
}

/// Preflight numeric layout or exact orthogonal transforms over a complete mixed selection.
///
/// Origins are explicit geometry, never estimated text extents. No object is silently skipped.
/// Moving connected anchors adjusts each touching route once; translating both route ends by
/// the same offset translates all its bends. Bindings, styles, and other views remain intact.
/// # Errors
/// Rejects empty/unsupported/mixed-view selections, insufficient distribution objects,
/// coordinate overflow, or invalid local routes without changing the source or history.
pub fn layout_plan(
    project: &Project,
    selection: impl IntoIterator<Item = ObjectRef>,
    operation: LayoutOperation,
) -> Result<LayoutPlan, LayoutError> {
    layout_plan_candidate(project, selection, operation).map(|(plan, _)| plan)
}

/// Share the validated candidate with preview projection, avoiding a second routing pass.
pub(crate) fn layout_plan_candidate(
    project: &Project,
    selection: impl IntoIterator<Item = ObjectRef>,
    operation: LayoutOperation,
) -> Result<(LayoutPlan, Project), LayoutError> {
    let (_, origins) = layout_origins(project, selection)?;
    let destinations = layout_destinations(&origins, operation)?;
    let mut plan = LayoutPlan {
        commands: vec![],
        changes: vec![],
        geometry_changed: BTreeSet::new(),
        unchanged: BTreeSet::new(),
        affected_routes: BTreeSet::new(),
    };
    let mut anchors = BTreeMap::new();
    for (target, before) in origins {
        let after = destinations[&target];
        if before != after {
            plan.changes.push(LayoutChange {
                target,
                before,
                after,
            });
        }
        let changed = match target {
            ObjectRef::Anchor(id) => {
                if before != after || matches!(operation, LayoutOperation::Transform(_)) {
                    anchors.insert(id, after);
                }
                before != after
            }
            ObjectRef::Label(id) => {
                if before != after {
                    let mut label = project.drawing().label(id).expect("known label").clone();
                    label.position = after;
                    plan.commands
                        .push(Command::Drawing(DrawingCommand::ReplaceLabel(label)));
                }
                before != after
            }
            ObjectRef::Polyline(id) => {
                let original = project.drawing().polyline(id).expect("known path");
                let line = reposition_polyline(original, before, after, operation)?;
                let changed = &line != original;
                if changed {
                    plan.commands
                        .push(Command::Drawing(DrawingCommand::ReplacePolyline(line)));
                }
                changed
            }
            _ => unreachable!("origins contain only supported targets"),
        };
        if changed {
            plan.geometry_changed.insert(target);
        } else {
            plan.unchanged.insert(target);
        }
    }
    if !anchors.is_empty() {
        plan.affected_routes = project
            .drawing()
            .route_layout(anchors.keys().copied())?
            .routes
            .iter()
            .map(|r| r.id)
            .collect();
        let command = if let LayoutOperation::Transform(transform) = operation {
            RouteCommand::TransformAnchors {
                anchors: anchors.into_keys().collect(),
                transform,
            }
        } else {
            RouteCommand::MoveAnchors(anchors)
        };
        plan.commands.push(Command::Route(command));
    }
    let mut candidate = project.clone();
    let mut commands = Vec::new();
    for command in std::mem::take(&mut plan.commands) {
        if command.apply(&mut candidate)?.is_some() {
            commands.push(command);
        }
    }
    plan.commands = commands;
    plan.affected_routes
        .retain(|id| candidate.drawing().route(*id) != project.drawing().route(*id));
    candidate.validate()?;
    Ok((plan, candidate))
}

/// Calculate exact target origins independently of object replacement and route adjustment.
fn layout_destinations(
    origins: &BTreeMap<ObjectRef, DrawingPoint>,
    operation: LayoutOperation,
) -> Result<BTreeMap<ObjectRef, DrawingPoint>, LayoutError> {
    let mut destinations = origins.clone();
    match operation {
        LayoutOperation::Transform(transform) => {
            for point in destinations.values_mut() {
                *point = transform.point(*point)?;
            }
        }
        LayoutOperation::Translate(delta) => {
            for point in destinations.values_mut() {
                *point = point.translated(delta)?;
            }
        }
        LayoutOperation::Align {
            axis,
            coordinate_um,
        } => {
            for point in destinations.values_mut() {
                axis.set(point, coordinate_um);
            }
        }
        LayoutOperation::Distribute(axis) => {
            if origins.len() < 3 {
                return Err(LayoutError::TooFewObjects);
            }
            let mut order: Vec<_> = origins.keys().copied().collect();
            order.sort_by_key(|target| (axis.coordinate(origins[target]), *target));
            let first = i128::from(axis.coordinate(origins[&order[0]]));
            let last = i128::from(axis.coordinate(origins[order.last().expect("three origins")]));
            let intervals =
                i128::try_from(order.len() - 1).map_err(|_| DrawingError::CoordinateOverflow)?;
            for (index, target) in order.iter().enumerate() {
                let index = i128::try_from(index).map_err(|_| DrawingError::CoordinateOverflow)?;
                let offset = (last - first)
                    .checked_mul(index)
                    .ok_or(DrawingError::CoordinateOverflow)?
                    / intervals;
                let coordinate =
                    i64::try_from(first + offset).map_err(|_| DrawingError::CoordinateOverflow)?;
                axis.set(
                    destinations.get_mut(target).expect("known target"),
                    coordinate,
                );
            }
        }
    }
    Ok(destinations)
}

/// Transform all path vertices even if its origin stays fixed, retaining style and identity.
fn reposition_polyline(
    original: &DrawingPolyline,
    before: DrawingPoint,
    after: DrawingPoint,
    operation: LayoutOperation,
) -> Result<DrawingPolyline, DrawingError> {
    let mut line = original.clone();
    let dx = i128::from(after.x) - i128::from(before.x);
    let dy = i128::from(after.y) - i128::from(before.y);
    for point in &mut line.points {
        *point = if let LayoutOperation::Transform(transform) = operation {
            transform.point(*point)?
        } else {
            DrawingPoint {
                x: i64::try_from(i128::from(point.x) + dx)
                    .map_err(|_| DrawingError::CoordinateOverflow)?,
                y: i64::try_from(i128::from(point.y) + dy)
                    .map_err(|_| DrawingError::CoordinateOverflow)?,
            }
        };
    }
    Ok(line)
}
