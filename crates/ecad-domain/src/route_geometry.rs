//! Local deterministic stubs and edits; no global rerouting, snapping, or topology inference.

use crate::{DrawingPoint, DrawingRoute, RouteEdit, RouteError, RouteMode};

fn append(points: &mut Vec<DrawingPoint>, point: DrawingPoint) {
    if points.last() != Some(&point) {
        points.push(point);
    }
}
fn elbow(
    from: DrawingPoint,
    to: DrawingPoint,
    old_from: DrawingPoint,
    old_to: DrawingPoint,
) -> Option<DrawingPoint> {
    if from.x == to.x || from.y == to.y {
        return None;
    }
    Some(if old_from.y == old_to.y {
        DrawingPoint { x: to.x, y: from.y }
    } else {
        DrawingPoint { x: from.x, y: to.y }
    })
}
/// Replace an endpoint and its first straight leg/bend locally, preserving the remaining path.
pub(crate) fn move_end(route: &mut DrawingRoute, end: usize, point: DrawingPoint) {
    if end == 1 {
        route.points.reverse();
    }
    let old = route.points[0];
    let neighbor = route.points[1];
    let mut points = vec![point];
    if route.mode == RouteMode::Free {
        for point in &route.points[1..] {
            append(&mut points, *point);
        }
    } else {
        // Treat the terminal's complete straight leg as the affected region, including
        // explicit collinear waypoints. Move its first bend, preserving the rest of the
        // path. This avoids accumulating a fresh elbow on every up/down anchor move.
        let horizontal = old.y == neighbor.y;
        let mut bend = 1;
        while bend + 1 < route.points.len() {
            let a = route.points[bend];
            let b = route.points[bend + 1];
            if (horizontal && a.y != b.y) || (!horizontal && a.x != b.x) {
                break;
            }
            bend += 1;
        }
        let shifted = |mut p: DrawingPoint| {
            if horizontal {
                p.y = point.y;
            } else {
                p.x = point.x;
            }
            p
        };
        for p in &route.points[1..bend] {
            append(&mut points, shifted(*p));
        }
        if bend + 1 == route.points.len() {
            let fixed = route.points[bend];
            if let Some(elbow) = elbow(point, fixed, old, neighbor) {
                append(&mut points, elbow);
            }
            append(&mut points, fixed);
        } else {
            append(&mut points, shifted(route.points[bend]));
            for p in &route.points[bend + 1..] {
                append(&mut points, *p);
            }
        }
    }
    route.points = points;
    if end == 1 {
        route.points.reverse();
    }
}
impl DrawingRoute {
    /// Prepare a geometry-only edit without changing this route or any electrical data.
    /// Anchor/view/conductor/appearance identities are retained; commit validates attachment positions.
    /// # Errors
    /// Rejects invalid indices, attached endpoint moves, unsupported segment modes, or invalid geometry.
    pub fn edited(&self, edit: &RouteEdit) -> Result<Self, RouteError> {
        let mut result = self.clone();
        match *edit {
            RouteEdit::MoveVertex { index, position } => {
                if index >= self.points.len() {
                    return Err(RouteError::InvalidIndex(self.id));
                }
                if ((index == 0 && self.anchors[0].is_some())
                    || (index + 1 == self.points.len() && self.anchors[1].is_some()))
                    && self.points[index] != position
                {
                    return Err(RouteError::AttachedEndpoint(self.id));
                }
                result.points[index] = position;
            }
            RouteEdit::InsertVertex { index, position } => {
                if index == 0 || index >= self.points.len() {
                    return Err(RouteError::InvalidIndex(self.id));
                }
                result.points.insert(index, position);
            }
            RouteEdit::RemoveVertex { index } => {
                if index == 0 || index >= self.points.len().saturating_sub(1) {
                    return Err(RouteError::InvalidIndex(self.id));
                }
                result.points.remove(index);
            }
            RouteEdit::MoveSegment {
                index,
                coordinate_um,
            } => {
                if self.mode != RouteMode::Orthogonal {
                    return Err(RouteError::OrthogonalRequired(self.id));
                }
                if index >= self.points.len().saturating_sub(1) {
                    return Err(RouteError::InvalidIndex(self.id));
                }
                let old_a = self.points[index];
                let old_b = self.points[index + 1];
                let (mut a, mut b) = (old_a, old_b);
                if old_a.y == old_b.y {
                    a.y = coordinate_um;
                    b.y = coordinate_um;
                } else {
                    a.x = coordinate_um;
                    b.x = coordinate_um;
                }
                let mut points = self.points[..index].to_vec();
                if index > 0 {
                    let previous = self.points[index - 1];
                    if let Some(bend) = elbow(previous, a, previous, old_a) {
                        append(&mut points, bend);
                    }
                } else if self.anchors[0].is_some() {
                    append(&mut points, old_a);
                }
                append(&mut points, a);
                append(&mut points, b);
                if index + 2 < self.points.len() {
                    let next = self.points[index + 2];
                    if let Some(bend) = elbow(b, next, old_b, next) {
                        append(&mut points, bend);
                    }
                } else if self.anchors[1].is_some() {
                    append(&mut points, old_b);
                }
                for point in &self.points[index + 2..] {
                    append(&mut points, *point);
                }
                result.points = points;
            }
        }
        result.validate_geometry()?;
        Ok(result)
    }
}
