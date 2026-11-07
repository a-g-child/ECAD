//! Cross-aggregate electrical continuity guards for view-local route geometry.

use super::Project;
use crate::*;

impl Project {
    pub(super) fn check_drawing_references(
        &self,
        drawing: &DrawingDocument,
    ) -> Result<(), ModelError> {
        for anchor in drawing.terminal_anchors() {
            self.check_terminal(Some(anchor.terminal))?;
        }
        for route in drawing.routes() {
            let conductor = self
                .conductor(route.conductor)
                .ok_or(ModelError::MissingConductor(route.conductor))?;
            Self::check_route_wire(drawing, route, conductor)?;
        }
        Ok(())
    }
    fn check_route_wire(
        drawing: &DrawingDocument,
        route: &DrawingRoute,
        conductor: &Conductor,
    ) -> Result<(), ModelError> {
        for (index, end) in [End::A, End::B].into_iter().enumerate() {
            let terminal = route.anchors[index]
                .map(|id| {
                    drawing
                        .terminal_anchor(id)
                        .map(|anchor| anchor.terminal)
                        .ok_or(RouteError::MissingAnchor(id))
                })
                .transpose()?;
            if terminal != conductor.endpoint(end) {
                return Err(RouteError::AttachmentConflict(route.id).into());
            }
        }
        Ok(())
    }
    pub(super) fn check_conductor_routes(&self, conductor: &Conductor) -> Result<(), ModelError> {
        for route in self.drawing.conductor_routes(conductor.id()) {
            Self::check_route_wire(&self.drawing, route, conductor)?;
        }
        Ok(())
    }
    /// Place an existing electrical terminal in one view, independent of symbol geometry.
    /// # Errors
    /// Rejects missing terminals, views, or duplicate placed-anchor identities.
    pub fn insert_terminal_anchor(&mut self, anchor: TerminalAnchor) -> Result<(), ModelError> {
        self.check_terminal(Some(anchor.terminal))?;
        self.drawing.insert_terminal_anchor(anchor)
    }
    /// Remove an unused terminal placement without deleting its real terminal.
    /// # Errors
    /// Rejects missing anchors or remaining touching routes.
    pub fn remove_terminal_anchor(&mut self, id: AnchorId) -> Result<TerminalAnchor, ModelError> {
        self.drawing.remove_terminal_anchor(id)
    }
    /// Insert one conductor representation with exact A/B attachment continuity.
    /// # Errors
    /// Rejects invalid references/geometry, duplicate IDs, or inconsistent electrical attachments.
    pub fn insert_drawing_route(&mut self, route: DrawingRoute) -> Result<(), ModelError> {
        let wire = self
            .conductor(route.conductor)
            .ok_or(ModelError::MissingConductor(route.conductor))?;
        Self::check_route_wire(&self.drawing, &route, wire)?;
        self.drawing.insert_route(route)
    }
    /// Replace a whole representation explicitly, retaining its stable route identity.
    /// # Errors
    /// Rejects missing/invalid routes or electrical/drawing attachment conflicts atomically.
    pub fn replace_drawing_route(
        &mut self,
        route: DrawingRoute,
    ) -> Result<DrawingRoute, ModelError> {
        let wire = self
            .conductor(route.conductor)
            .ok_or(ModelError::MissingConductor(route.conductor))?;
        Self::check_route_wire(&self.drawing, &route, wire)?;
        self.drawing.replace_route(route)
    }
    /// Remove geometry only, preserving its electrical conductor and other views.
    /// # Errors
    /// Rejects an unavailable route identity.
    pub fn remove_drawing_route(&mut self, id: RouteId) -> Result<DrawingRoute, ModelError> {
        self.drawing.remove_route(id)
    }
    /// Move one anchor and locally adjust all touching routes, returning exact previous geometry.
    /// # Errors
    /// Rejects missing anchors or invalid/collapsed resulting geometry without partial mutation.
    pub fn move_terminal_anchor(
        &mut self,
        id: AnchorId,
        position: DrawingPoint,
    ) -> Result<AnchorLayout, ModelError> {
        self.drawing.move_terminal_anchor(id, position)
    }
    /// Restore an exact local anchor/route layout; bindings and style intent cannot be changed.
    /// # Errors
    /// Rejects incomplete/rebound snapshots or invalid geometry atomically.
    pub fn restore_anchor_layout(
        &mut self,
        layout: AnchorLayout,
    ) -> Result<AnchorLayout, ModelError> {
        self.drawing.restore_anchor_layout(layout)
    }
}

impl Project {
    /// Move selected drawing terminal placements simultaneously without electrical mutation.
    /// # Errors
    /// Missing anchors, coordinate overflow, or invalid local route geometry leave state intact.
    pub fn move_terminal_anchors(
        &mut self,
        positions: std::collections::BTreeMap<AnchorId, DrawingPoint>,
    ) -> Result<RouteLayout, ModelError> {
        self.drawing.move_terminal_anchors(positions)
    }
    /// Restore exact affected route geometry, preserving every electrical and style binding.
    /// # Errors
    /// Rejects incomplete/rebound layouts or invalid geometry without mutation.
    pub fn restore_route_layout(&mut self, layout: RouteLayout) -> Result<RouteLayout, ModelError> {
        self.drawing.restore_route_layout(layout)
    }
}
