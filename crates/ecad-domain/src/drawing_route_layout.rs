//! Simultaneous anchor movement with one localized route union and exact restoration.

use super::DrawingDocument;
use crate::*;
use std::collections::{BTreeMap, BTreeSet};

impl DrawingDocument {
    /// Snapshot selected anchors and the union of their touching routes, in stable order.
    /// # Errors
    /// Rejects an unavailable anchor before producing a partial layout.
    pub fn route_layout(
        &self,
        anchors: impl IntoIterator<Item = AnchorId>,
    ) -> Result<RouteLayout, ModelError> {
        let mut routes = BTreeSet::new();
        let mut placements = Vec::new();
        for id in anchors.into_iter().collect::<BTreeSet<_>>() {
            placements.push(
                self.anchors
                    .get(&id)
                    .ok_or(RouteError::MissingAnchor(id))?
                    .clone(),
            );
            if let Some(touching) = self.anchor_routes.get(&id) {
                routes.extend(touching);
            }
        }
        Ok(RouteLayout {
            anchors: placements,
            routes: routes
                .into_iter()
                .map(|id| self.routes[&id].clone())
                .collect(),
        })
    }

    /// Restore complete affected geometry without rebinding anchors, routes, views, or styles.
    /// # Errors
    /// Rejects duplicate/incomplete/rebound records and invalid geometry before any mutation.
    pub fn restore_route_layout(&mut self, layout: RouteLayout) -> Result<RouteLayout, ModelError> {
        let old = self.route_layout(layout.anchors.iter().map(|anchor| anchor.id))?;
        let moved: BTreeMap<_, _> = layout.anchors.iter().map(|a| (a.id, a.clone())).collect();
        if moved.len() != layout.anchors.len() {
            return Err(RouteError::LayoutMismatch(layout.anchors[0].id).into());
        }
        // No anchor means no affected routes. Nonempty unexpected routes still fail below.
        for anchor in &old.anchors {
            let new = &moved[&anchor.id];
            if anchor.view != new.view || anchor.terminal != new.terminal {
                return Err(RouteError::LayoutMismatch(anchor.id).into());
            }
        }
        let expected: BTreeSet<_> = old.routes.iter().map(|r| r.id).collect();
        let supplied: BTreeSet<_> = layout.routes.iter().map(|r| r.id).collect();
        if expected != supplied || supplied.len() != layout.routes.len() {
            return Err(RouteError::LayoutRouteSetMismatch.into());
        }
        for route in &layout.routes {
            let mut previous = self.routes[&route.id].clone();
            previous.points = route.points.clone();
            if previous != *route {
                return Err(RouteError::LayoutMismatch(layout.anchors[0].id).into());
            }
            self.check_route_with_anchor(route, Some(&moved))?;
        }
        for anchor in layout.anchors {
            self.anchors.insert(anchor.id, anchor);
        }
        for route in layout.routes {
            self.routes.insert(route.id, route);
        }
        Ok(old)
    }

    /// Move anchors simultaneously and adjust each touching route once.
    ///
    /// When both ends move by the same offset, translate the entire route, preserving
    /// every bend. Other moves adjust endpoint legs in A/B order. No electrical data
    /// changes. The returned layout is the exact inverse, independent of routing rules.
    /// # Errors
    /// Rejects unavailable anchors, overflow, or collapsed/invalid local geometry atomically.
    /// Differential moves that collapse the intermediate path require explicit route replacement.
    pub fn move_terminal_anchors(
        &mut self,
        positions: BTreeMap<AnchorId, DrawingPoint>,
    ) -> Result<RouteLayout, ModelError> {
        let mut layout = self.route_layout(positions.keys().copied())?;
        let offsets: BTreeMap<_, _> = layout
            .anchors
            .iter()
            .map(|anchor| {
                let to = positions[&anchor.id];
                (
                    anchor.id,
                    (
                        i128::from(to.x) - i128::from(anchor.position.x),
                        i128::from(to.y) - i128::from(anchor.position.y),
                    ),
                )
            })
            .collect();
        for anchor in &mut layout.anchors {
            anchor.position = positions[&anchor.id];
        }
        for route in &mut layout.routes {
            let deltas = route
                .anchors
                .map(|id| id.and_then(|id| offsets.get(&id).copied()));
            if let [Some(a), Some(b)] = deltas
                && a == b
            {
                for point in &mut route.points {
                    point.x = i64::try_from(i128::from(point.x) + a.0)
                        .map_err(|_| DrawingError::CoordinateOverflow)?;
                    point.y = i64::try_from(i128::from(point.y) + a.1)
                        .map_err(|_| DrawingError::CoordinateOverflow)?;
                }
            } else {
                for end in 0..2 {
                    if let Some(position) = route.anchors[end].and_then(|id| positions.get(&id)) {
                        route.validate_geometry()?;
                        crate::route_geometry::move_end(route, end, *position);
                    }
                }
            }
        }
        self.restore_route_layout(layout)
    }
    /// Transform selected anchors and each touching route once, without electrical mutation.
    /// Routes with both anchors selected transform completely, including self-loops at the
    /// pivot. Other routes retain their unaffected geometry and adjust only the moved end.
    /// # Errors
    /// Rejects missing anchors, overflow, or invalid local geometry before any mutation.
    pub fn transform_terminal_anchors(
        &mut self,
        anchors: BTreeSet<AnchorId>,
        transform: DrawingTransform,
    ) -> Result<RouteLayout, ModelError> {
        let mut layout = self.route_layout(anchors.iter().copied())?;
        let mut positions = BTreeMap::new();
        for anchor in &mut layout.anchors {
            anchor.position = transform.point(anchor.position)?;
            positions.insert(anchor.id, anchor.position);
        }
        for route in &mut layout.routes {
            if route
                .anchors
                .iter()
                .all(|id| id.is_some_and(|id| anchors.contains(&id)))
            {
                for point in &mut route.points {
                    *point = transform.point(*point)?;
                }
            } else {
                for end in 0..2 {
                    if let Some(position) = route.anchors[end].and_then(|id| positions.get(&id)) {
                        route.validate_geometry()?;
                        crate::route_geometry::move_end(route, end, *position);
                    }
                }
            }
        }
        self.restore_route_layout(layout)
    }
}
