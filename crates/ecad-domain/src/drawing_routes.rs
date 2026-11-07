//! Indexed whole-route ownership and atomic anchor/layout replacement.

use super::DrawingDocument;
use crate::*;
use std::collections::{BTreeMap, BTreeSet};

impl DrawingDocument {
    /// Iterate placed terminal anchors in stable identity order.
    pub fn terminal_anchors(&self) -> impl ExactSizeIterator<Item = &TerminalAnchor> {
        self.anchors.values()
    }
    /// Inspect a placed terminal anchor.
    pub fn terminal_anchor(&self, id: AnchorId) -> Option<&TerminalAnchor> {
        self.anchors.get(&id)
    }
    /// Iterate whole routes in stable identity order.
    pub fn routes(&self) -> impl ExactSizeIterator<Item = &DrawingRoute> {
        self.routes.values()
    }
    /// Inspect a whole route.
    pub fn route(&self, id: RouteId) -> Option<&DrawingRoute> {
        self.routes.get(&id)
    }
    /// Query only the representations of one conductor, independent of unrelated drawing size.
    pub fn conductor_routes(&self, id: ConductorId) -> impl Iterator<Item = &DrawingRoute> {
        self.conductor_routes
            .get(&id)
            .into_iter()
            .flat_map(|ids| ids.iter())
            .filter_map(|id| self.routes.get(id))
    }
    /// Insert a view-local real-terminal placement. Project insertion also validates its terminal.
    /// # Errors
    /// Rejects duplicate IDs or unavailable views.
    pub fn insert_terminal_anchor(&mut self, anchor: TerminalAnchor) -> Result<(), ModelError> {
        if self.anchors.contains_key(&anchor.id) {
            return Err(RouteError::DuplicateAnchor(anchor.id).into());
        }
        if self.view(anchor.view).is_none() {
            return Err(DrawingError::MissingView(anchor.view).into());
        }
        self.anchors.insert(anchor.id, anchor);
        Ok(())
    }
    /// Remove an unused placed anchor; this never deletes its real electrical terminal.
    /// # Errors
    /// Rejects missing anchors or remaining touching routes.
    pub fn remove_terminal_anchor(&mut self, id: AnchorId) -> Result<TerminalAnchor, ModelError> {
        if self
            .anchor_routes
            .get(&id)
            .is_some_and(|routes| !routes.is_empty())
        {
            return Err(RouteError::AnchorInUse(id).into());
        }
        self.anchors
            .remove(&id)
            .ok_or_else(|| RouteError::MissingAnchor(id).into())
    }
    fn check_route_with_anchor(
        &self,
        route: &DrawingRoute,
        moved: Option<&TerminalAnchor>,
    ) -> Result<(), ModelError> {
        route.validate_geometry()?;
        if self.view(route.view).is_none() {
            return Err(DrawingError::MissingView(route.view).into());
        }
        self.check_appearance(&route.appearance, DrawingRole::Stroke)?;
        if self
            .conductor_routes(route.conductor)
            .any(|old| old.id != route.id && old.view == route.view)
        {
            return Err(RouteError::RepeatedRepresentation(route.conductor).into());
        }
        for (index, id) in route.anchors.iter().enumerate() {
            if let Some(id) = id {
                let anchor = moved
                    .filter(|anchor| anchor.id == *id)
                    .or_else(|| self.anchors.get(id))
                    .ok_or(RouteError::MissingAnchor(*id))?;
                let point = if index == 0 {
                    route.points[0]
                } else {
                    *route.points.last().expect("validated path has endpoints")
                };
                if anchor.view != route.view || point != anchor.position {
                    return Err(RouteError::AttachmentConflict(route.id).into());
                }
            }
        }
        Ok(())
    }
    fn index_route(&mut self, route: &DrawingRoute, insert: bool) {
        fn update<K: Ord + Copy>(
            map: &mut BTreeMap<K, BTreeSet<RouteId>>,
            key: K,
            route: RouteId,
            insert: bool,
        ) {
            if insert {
                map.entry(key).or_default().insert(route);
            } else if let Some(ids) = map.get_mut(&key) {
                ids.remove(&route);
                if ids.is_empty() {
                    map.remove(&key);
                }
            }
        }
        update(
            &mut self.conductor_routes,
            route.conductor,
            route.id,
            insert,
        );
        for anchor in route.anchors.into_iter().flatten() {
            update(&mut self.anchor_routes, anchor, route.id, insert);
        }
    }
    /// Insert one whole representation. Project insertion additionally checks conductor continuity.
    /// # Errors
    /// Rejects duplicates, repeated same-view conductor representations, invalid geometry/anchors/styles.
    pub fn insert_route(&mut self, route: DrawingRoute) -> Result<(), ModelError> {
        if self.routes.contains_key(&route.id) {
            return Err(RouteError::DuplicateRoute(route.id).into());
        }
        self.check_route_with_anchor(&route, None)?;
        self.index_route(&route, true);
        self.routes.insert(route.id, route);
        Ok(())
    }
    /// Replace one whole representation, retaining its identity and returning exact previous state.
    /// # Errors
    /// Rejects missing routes or invalid resulting drawing references/geometry.
    pub fn replace_route(&mut self, route: DrawingRoute) -> Result<DrawingRoute, ModelError> {
        let old = self
            .routes
            .get(&route.id)
            .ok_or(RouteError::MissingRoute(route.id))?
            .clone();
        self.check_route_with_anchor(&route, None)?;
        self.index_route(&old, false);
        self.index_route(&route, true);
        self.routes.insert(route.id, route);
        Ok(old)
    }
    /// Remove one representation without changing its conductor, terminals, or other views.
    /// # Errors
    /// Rejects an unavailable route identity.
    pub fn remove_route(&mut self, id: RouteId) -> Result<DrawingRoute, ModelError> {
        let route = self
            .routes
            .remove(&id)
            .ok_or(RouteError::MissingRoute(id))?;
        self.index_route(&route, false);
        Ok(route)
    }
    /// Snapshot only one anchor and its touching routes, for exact local undo.
    /// # Errors
    /// Rejects an unavailable anchor identity.
    pub fn anchor_layout(&self, id: AnchorId) -> Result<AnchorLayout, ModelError> {
        let anchor = self
            .anchors
            .get(&id)
            .ok_or(RouteError::MissingAnchor(id))?
            .clone();
        let routes = self
            .anchor_routes
            .get(&id)
            .into_iter()
            .flat_map(|ids| ids.iter())
            .map(|id| self.routes[id].clone())
            .collect();
        Ok(AnchorLayout { anchor, routes })
    }
    /// Atomically restore one local layout, retaining all anchor/route electrical bindings and styles.
    /// # Errors
    /// Rejects missing/extra/rebound records or invalid geometry before changing anything.
    pub fn restore_anchor_layout(
        &mut self,
        layout: AnchorLayout,
    ) -> Result<AnchorLayout, ModelError> {
        let old = self.anchor_layout(layout.anchor.id)?;
        let mismatch = || ModelError::from(RouteError::LayoutMismatch(layout.anchor.id));
        if old.anchor.view != layout.anchor.view
            || old.anchor.terminal != layout.anchor.terminal
            || old.routes.len() != layout.routes.len()
        {
            return Err(mismatch());
        }
        let expected: BTreeSet<_> = old.routes.iter().map(|route| route.id).collect();
        let supplied: BTreeSet<_> = layout.routes.iter().map(|route| route.id).collect();
        if expected != supplied || supplied.len() != layout.routes.len() {
            return Err(mismatch());
        }
        for route in &layout.routes {
            let mut previous = self.routes[&route.id].clone();
            previous.points = route.points.clone();
            if previous != *route {
                return Err(mismatch());
            }
            self.check_route_with_anchor(route, Some(&layout.anchor))?;
        }
        self.anchors.insert(layout.anchor.id, layout.anchor);
        for route in layout.routes {
            self.routes.insert(route.id, route);
        }
        Ok(old)
    }
    /// Move an anchor by adjusting touching first legs/bends; preserve remaining paths and exact undo data.
    /// # Errors
    /// Rejects missing anchors or collapsed/invalid resulting paths without partial changes.
    pub fn move_terminal_anchor(
        &mut self,
        id: AnchorId,
        position: DrawingPoint,
    ) -> Result<AnchorLayout, ModelError> {
        let mut layout = self.anchor_layout(id)?;
        layout.anchor.position = position;
        for route in &mut layout.routes {
            for end in 0..2 {
                if route.anchors[end] == Some(id) {
                    crate::route_geometry::move_end(route, end, position);
                }
            }
        }
        self.restore_anchor_layout(layout)
    }
    pub(super) fn validate_routes(&self) -> Result<(), ModelError> {
        for anchor in self.anchors.values() {
            if self.view(anchor.view).is_none() {
                return Err(DrawingError::MissingView(anchor.view).into());
            }
        }
        let mut anchor_routes: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
        let mut conductor_routes: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
        for route in self.routes.values() {
            self.check_route_with_anchor(route, None)?;
            conductor_routes
                .entry(route.conductor)
                .or_default()
                .insert(route.id);
            for anchor in route.anchors.into_iter().flatten() {
                anchor_routes.entry(anchor).or_default().insert(route.id);
            }
        }
        if anchor_routes != self.anchor_routes || conductor_routes != self.conductor_routes {
            return Err(RouteError::InvalidIndexState.into());
        }
        Ok(())
    }
}
