//! Deterministic bounded-distance snap queries; linear scanning is explicit in this first slice.

use super::DrawingDocument;
use crate::*;

fn midpoint(a: DrawingPoint, b: DrawingPoint) -> DrawingPoint {
    // The average of two i64 values remains representable; Euclidean division rounds down.
    DrawingPoint {
        x: i64::try_from((i128::from(a.x) + i128::from(b.x)).div_euclid(2))
            .expect("bounded average"),
        y: i64::try_from((i128::from(a.y) + i128::from(b.y)).div_euclid(2))
            .expect("bounded average"),
    }
}
fn grid_coordinate(point: i64, origin: i64, spacing: u32) -> i64 {
    let spacing = i128::from(spacing);
    let lower =
        i128::from(origin) + (i128::from(point) - i128::from(origin)).div_euclid(spacing) * spacing;
    let upper = lower + spacing;
    let (preferred, other) = if i128::from(point) - lower <= upper - i128::from(point) {
        (lower, upper)
    } else {
        (upper, lower)
    };
    i64::try_from(preferred)
        .or_else(|_| i64::try_from(other))
        .expect("at least one neighboring grid line is representable")
}
impl DrawingDocument {
    /// Query all enabled candidates in one view without changing geometry or connectivity.
    ///
    /// Rank terminal anchors, endpoints, midpoints, then grid; within each class use
    /// exact squared distance then typed target identity. Distances are filtered per
    /// axis before squaring, avoiding overflow even across the full coordinate span.
    /// Coincident targets remain visible for explicit cycling. This implementation
    /// scans the view's objects and vertices; no spatial index or hysteresis is implied.
    /// # Errors
    /// Rejects an unavailable view. Exclusions may contain absent identities harmlessly.
    pub fn snap_candidates(&self, query: &SnapQuery) -> Result<Vec<SnapCandidate>, DrawingError> {
        if self.view(query.view).is_none() {
            return Err(DrawingError::MissingView(query.view));
        }
        let mut candidates = Vec::new();
        let tolerance = u128::from(query.settings.tolerance_um);
        let mut offer = |target, position: DrawingPoint| {
            let dx = (i128::from(query.position.x) - i128::from(position.x)).unsigned_abs();
            let dy = (i128::from(query.position.y) - i128::from(position.y)).unsigned_abs();
            if dx > tolerance || dy > tolerance {
                return;
            }
            let distance = dx * dx + dy * dy;
            if distance <= tolerance * tolerance {
                candidates.push(SnapCandidate {
                    target,
                    position,
                    distance_squared_um: distance,
                });
            }
        };
        if query.settings.terminals {
            for anchor in self.terminal_anchors().filter(|a| {
                a.view == query.view && !query.excluded.contains(&ObjectRef::Anchor(a.id))
            }) {
                offer(SnapTarget::Terminal(anchor.id), anchor.position);
            }
        }
        for route in self
            .routes()
            .filter(|r| r.view == query.view && !query.excluded.contains(&ObjectRef::Route(r.id)))
        {
            if query.settings.endpoints {
                for (index, end) in [SnapEnd::First, SnapEnd::Last].into_iter().enumerate() {
                    if route.anchors[index]
                        .is_some_and(|id| query.excluded.contains(&ObjectRef::Anchor(id)))
                    {
                        continue;
                    }
                    let position = if index == 0 {
                        route.points[0]
                    } else {
                        *route.points.last().expect("validated route")
                    };
                    offer(
                        SnapTarget::RouteEndpoint {
                            route: route.id,
                            end,
                        },
                        position,
                    );
                }
            }
            if query.settings.midpoints {
                for (segment, pair) in route.points.windows(2).enumerate() {
                    offer(
                        SnapTarget::RouteMidpoint {
                            route: route.id,
                            segment,
                        },
                        midpoint(pair[0], pair[1]),
                    );
                }
            }
        }
        for line in self.polylines().filter(|p| {
            p.view == query.view && !query.excluded.contains(&ObjectRef::Polyline(p.id))
        }) {
            if query.settings.endpoints {
                offer(
                    SnapTarget::PolylineEndpoint {
                        polyline: line.id,
                        end: SnapEnd::First,
                    },
                    line.points[0],
                );
                offer(
                    SnapTarget::PolylineEndpoint {
                        polyline: line.id,
                        end: SnapEnd::Last,
                    },
                    *line.points.last().expect("validated path"),
                );
            }
            if query.settings.midpoints {
                for (segment, pair) in line.points.windows(2).enumerate() {
                    offer(
                        SnapTarget::PolylineMidpoint {
                            polyline: line.id,
                            segment,
                        },
                        midpoint(pair[0], pair[1]),
                    );
                }
            }
        }
        if let Some(grid) = query.settings.grid {
            offer(
                SnapTarget::Grid,
                DrawingPoint {
                    x: grid_coordinate(query.position.x, grid.origin.x, grid.spacing.micrometres()),
                    y: grid_coordinate(query.position.y, grid.origin.y, grid.spacing.micrometres()),
                },
            );
        }
        candidates.sort_by_key(|c| (c.target.priority(), c.distance_squared_um, c.target));
        Ok(candidates)
    }
}
