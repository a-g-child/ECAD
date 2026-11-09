//! Invariant-preserving drawing aggregate, separate from electrical connectivity.

#[path = "drawing_properties.rs"]
mod properties;
#[path = "drawing_route_layout.rs"]
mod route_layout;
#[path = "drawing_routes.rs"]
mod routes;
#[path = "drawing_snapping.rs"]
mod snapping;

use crate::*;
use std::collections::{BTreeMap, BTreeSet};

/// Project-owned drawing views, styles, annotations, and graphical paths.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DrawingDocument {
    pub(crate) pictures: BTreeMap<PictureId, DrawingPicture>,
    settings: DrawingSettings,
    anchors: BTreeMap<AnchorId, TerminalAnchor>,
    routes: BTreeMap<RouteId, DrawingRoute>,
    anchor_routes: BTreeMap<AnchorId, BTreeSet<RouteId>>,
    conductor_routes: BTreeMap<ConductorId, BTreeSet<RouteId>>,
    views: BTreeMap<ViewId, DrawingView>,
    styles: BTreeMap<StyleId, DrawingStyle>,
    labels: BTreeMap<LabelId, DrawingLabel>,
    polylines: BTreeMap<PolylineId, DrawingPolyline>,
}
impl DrawingDocument {
    /// Inspect portable role values and future-object creation presets.
    pub fn settings(&self) -> &DrawingSettings {
        &self.settings
    }
    /// Iterate drawing views in identity order.
    pub fn views(&self) -> impl ExactSizeIterator<Item = &DrawingView> {
        self.views.values()
    }
    /// Iterate named styles in identity order.
    pub fn styles(&self) -> impl ExactSizeIterator<Item = &DrawingStyle> {
        self.styles.values()
    }
    /// Iterate annotations in identity order.
    pub fn labels(&self) -> impl ExactSizeIterator<Item = &DrawingLabel> {
        self.labels.values()
    }
    /// Iterate graphical paths in identity order.
    pub fn polylines(&self) -> impl ExactSizeIterator<Item = &DrawingPolyline> {
        self.polylines.values()
    }
    /// Resolve one drawing view.
    pub fn view(&self, id: ViewId) -> Option<&DrawingView> {
        self.views.get(&id)
    }
    /// Resolve one named style.
    pub fn style(&self, id: StyleId) -> Option<&DrawingStyle> {
        self.styles.get(&id)
    }
    /// Resolve one annotation.
    pub fn label(&self, id: LabelId) -> Option<&DrawingLabel> {
        self.labels.get(&id)
    }
    /// Resolve one graphical path.
    pub fn polyline(&self, id: PolylineId) -> Option<&DrawingPolyline> {
        self.polylines.get(&id)
    }
    /// Replace settings without rewriting objects; return the previous settings.
    /// # Errors
    /// Rejects missing/wrong-role style references and invalid creation patches.
    pub fn set_settings(
        &mut self,
        settings: DrawingSettings,
    ) -> Result<DrawingSettings, DrawingError> {
        self.check_appearance(&settings.text_creation, DrawingRole::Text)?;
        self.check_appearance(&settings.stroke_creation, DrawingRole::Stroke)?;
        self.check_appearance(&settings.route_creation, DrawingRole::Stroke)?;
        Ok(std::mem::replace(&mut self.settings, settings))
    }
    /// Insert one independent drawing scope.
    /// # Errors
    /// Rejects duplicates or a blank name without mutation.
    pub fn insert_view(&mut self, view: DrawingView) -> Result<(), DrawingError> {
        if self.views.contains_key(&view.id) {
            return Err(DrawingError::DuplicateView(view.id));
        }
        if view.name.trim().is_empty() {
            return Err(DrawingError::EmptyName);
        }
        self.views.insert(view.id, view);
        Ok(())
    }
    /// Change view name/overrides without rewriting its objects; return the old view.
    /// # Errors
    /// Rejects missing views or a blank name.
    pub fn replace_view(&mut self, view: DrawingView) -> Result<DrawingView, DrawingError> {
        if !self.views.contains_key(&view.id) {
            return Err(DrawingError::MissingView(view.id));
        }
        if view.name.trim().is_empty() {
            return Err(DrawingError::EmptyName);
        }
        Ok(self.views.insert(view.id, view).expect("view was checked"))
    }
    /// Remove an empty view, preserving objects in other views.
    /// # Errors
    /// Rejects missing views or any remaining annotation/path membership.
    pub fn remove_view(&mut self, id: ViewId) -> Result<DrawingView, DrawingError> {
        if self.pictures.values().any(|o| o.view == id)
            || self.anchors.values().any(|o| o.view == id)
            || self.routes.values().any(|o| o.view == id)
            || self.labels.values().any(|o| o.view == id)
            || self.polylines.values().any(|o| o.view == id)
        {
            return Err(DrawingError::ViewInUse(id));
        }
        self.views.remove(&id).ok_or(DrawingError::MissingView(id))
    }
    /// Insert one role-validated named style.
    /// # Errors
    /// Rejects duplicate identities.
    pub fn insert_style(&mut self, style: DrawingStyle) -> Result<(), DrawingError> {
        if self.styles.contains_key(&style.id()) {
            return Err(DrawingError::DuplicateStyle(style.id()));
        }
        self.styles.insert(style.id(), style);
        Ok(())
    }
    /// Change one style without touching dependent objects; return the previous value.
    /// # Errors
    /// Rejects missing styles or changing the role associated with an existing identity.
    pub fn replace_style(&mut self, style: DrawingStyle) -> Result<DrawingStyle, DrawingError> {
        let old = self
            .styles
            .get(&style.id())
            .ok_or(DrawingError::MissingStyle(style.id()))?;
        if old.role() != style.role() {
            return Err(DrawingError::WrongRole);
        }
        Ok(self
            .styles
            .insert(style.id(), style)
            .expect("style was checked"))
    }
    /// Remove an unused style; explicit presets also keep their links valid.
    /// # Errors
    /// Rejects missing styles or references from any object or creation preset.
    pub fn remove_style(&mut self, id: StyleId) -> Result<DrawingStyle, DrawingError> {
        if self.routes.values().any(|o| o.appearance.style == Some(id))
            || self.labels.values().any(|o| o.appearance.style == Some(id))
            || self
                .polylines
                .values()
                .any(|o| o.appearance.style == Some(id))
            || self.settings.text_creation.style == Some(id)
            || self.settings.stroke_creation.style == Some(id)
            || self.settings.route_creation.style == Some(id)
        {
            return Err(DrawingError::StyleInUse(id));
        }
        self.styles
            .remove(&id)
            .ok_or(DrawingError::MissingStyle(id))
    }
    pub(crate) fn check_appearance(
        &self,
        appearance: &Appearance,
        role: DrawingRole,
    ) -> Result<(), DrawingError> {
        appearance.overrides.validate_role(role)?;
        if let Some(id) = appearance.style {
            let style = self.styles.get(&id).ok_or(DrawingError::MissingStyle(id))?;
            if style.role() != role {
                return Err(DrawingError::WrongRole);
            }
        }
        Ok(())
    }
    /// Insert a fully specified annotation; importing never reapplies creation presets.
    /// # Errors
    /// Rejects duplicate IDs, blank text, missing views/styles, and wrong-role overrides.
    pub fn insert_label(&mut self, label: DrawingLabel) -> Result<(), DrawingError> {
        if self.labels.contains_key(&label.id) {
            return Err(DrawingError::DuplicateLabel(label.id));
        }
        self.check_label(&label)?;
        self.labels.insert(label.id, label);
        Ok(())
    }
    fn check_label(&self, label: &DrawingLabel) -> Result<(), DrawingError> {
        label.validate()?;
        if !self.views.contains_key(&label.view) {
            return Err(DrawingError::MissingView(label.view));
        }
        self.check_appearance(&label.appearance, DrawingRole::Text)
    }
    /// Replace one annotation while preserving identity; return its exact previous value.
    /// # Errors
    /// Rejects unavailable IDs and invalid resulting text/view/style references.
    pub fn replace_label(&mut self, label: DrawingLabel) -> Result<DrawingLabel, DrawingError> {
        if !self.labels.contains_key(&label.id) {
            return Err(DrawingError::MissingLabel(label.id));
        }
        self.check_label(&label)?;
        Ok(self
            .labels
            .insert(label.id, label)
            .expect("label was checked"))
    }
    /// Remove one annotation without changing styles/settings.
    /// # Errors
    /// Rejects an unavailable annotation identity.
    pub fn remove_label(&mut self, id: LabelId) -> Result<DrawingLabel, DrawingError> {
        self.labels
            .remove(&id)
            .ok_or(DrawingError::MissingLabel(id))
    }
    /// Insert a whole retained graphical path.
    /// # Errors
    /// Rejects duplicates, invalid vertices, missing views/styles, and wrong-role overrides.
    pub fn insert_polyline(&mut self, line: DrawingPolyline) -> Result<(), DrawingError> {
        if self.polylines.contains_key(&line.id) {
            return Err(DrawingError::DuplicatePolyline(line.id));
        }
        self.check_polyline(&line)?;
        self.polylines.insert(line.id, line);
        Ok(())
    }
    fn check_polyline(&self, line: &DrawingPolyline) -> Result<(), DrawingError> {
        line.validate()?;
        if !self.views.contains_key(&line.view) {
            return Err(DrawingError::MissingView(line.view));
        }
        self.check_appearance(&line.appearance, DrawingRole::Stroke)
    }
    /// Replace one whole path with validated geometry/view/appearance; return the old value.
    /// # Errors
    /// Rejects missing paths and invalid resulting references/geometry.
    pub fn replace_polyline(
        &mut self,
        line: DrawingPolyline,
    ) -> Result<DrawingPolyline, DrawingError> {
        if !self.polylines.contains_key(&line.id) {
            return Err(DrawingError::MissingPolyline(line.id));
        }
        self.check_polyline(&line)?;
        Ok(self
            .polylines
            .insert(line.id, line)
            .expect("path was checked"))
    }
    /// Remove one whole graphical path.
    /// # Errors
    /// Rejects an unavailable path identity.
    pub fn remove_polyline(&mut self, id: PolylineId) -> Result<DrawingPolyline, DrawingError> {
        self.polylines
            .remove(&id)
            .ok_or(DrawingError::MissingPolyline(id))
    }
    /// Recheck every drawing reference; no renderer, font lookup, or electrical routing runs.
    /// # Errors
    /// Returns the first invalid reference, role, or object value.
    pub fn validate(&self) -> Result<(), ModelError> {
        for picture in self.pictures.values() {
            picture.validate()?;
            if !self.views.contains_key(&picture.view) {
                return Err(DrawingError::MissingView(picture.view).into());
            }
        }
        self.check_appearance(&self.settings.text_creation, DrawingRole::Text)?;
        self.check_appearance(&self.settings.stroke_creation, DrawingRole::Stroke)?;
        self.check_appearance(&self.settings.route_creation, DrawingRole::Stroke)?;
        for label in self.labels.values() {
            self.check_label(label)?;
        }
        for line in self.polylines.values() {
            self.check_polyline(line)?;
        }
        self.validate_routes()?;
        Ok(())
    }
}
