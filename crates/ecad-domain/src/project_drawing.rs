//! Validated drawing aggregate access; ordinary edits use application commands.

use super::Project;
use crate::*;

impl Project {
    /// Inspect project-owned drawing content and settings without mutable storage access.
    pub fn drawing(&self) -> &DrawingDocument {
        &self.drawing
    }
    /// Install a fully validated drawing aggregate during loading/import.
    /// # Errors
    /// Invalid drawing references leave the previous aggregate unchanged.
    pub fn replace_drawing(
        &mut self,
        drawing: DrawingDocument,
    ) -> Result<DrawingDocument, ModelError> {
        drawing.validate()?;
        self.check_drawing_references(&drawing)?;
        self.check_sheet_views(&drawing)?;
        Ok(std::mem::replace(&mut self.drawing, drawing))
    }
    /// Insert one drawing view through aggregate invariants.
    /// # Errors
    /// Returns identity, reference, role, or in-use failures without partial mutation.
    pub fn insert_drawing_view(&mut self, value: DrawingView) -> Result<(), DrawingError> {
        self.drawing.insert_view(value)
    }
    /// Replace one drawing view through aggregate invariants.
    /// # Errors
    /// Returns identity, reference, role, or in-use failures without partial mutation.
    pub fn replace_drawing_view(
        &mut self,
        value: DrawingView,
    ) -> Result<DrawingView, DrawingError> {
        self.drawing.replace_view(value)
    }
    /// Remove one drawing view through aggregate invariants.
    /// # Errors
    /// Returns identity, reference, role, or in-use failures without partial mutation.
    pub fn remove_drawing_view(&mut self, id: ViewId) -> Result<DrawingView, DrawingError> {
        if self
            .sheets()
            .iter()
            .any(|s| s.placement.as_ref().is_some_and(|p| p.view == id))
        {
            return Err(DrawingError::ViewInUse(id));
        }
        self.drawing.remove_view(id)
    }
    /// Insert one drawing style through aggregate invariants.
    /// # Errors
    /// Returns identity, reference, role, or in-use failures without partial mutation.
    pub fn insert_drawing_style(&mut self, value: DrawingStyle) -> Result<(), DrawingError> {
        self.drawing.insert_style(value)
    }
    /// Replace one drawing style through aggregate invariants.
    /// # Errors
    /// Returns identity, reference, role, or in-use failures without partial mutation.
    pub fn replace_drawing_style(
        &mut self,
        value: DrawingStyle,
    ) -> Result<DrawingStyle, DrawingError> {
        self.drawing.replace_style(value)
    }
    /// Remove one drawing style through aggregate invariants.
    /// # Errors
    /// Returns identity, reference, role, or in-use failures without partial mutation.
    pub fn remove_drawing_style(&mut self, id: StyleId) -> Result<DrawingStyle, DrawingError> {
        self.drawing.remove_style(id)
    }
    /// Insert one drawing label through aggregate invariants.
    /// # Errors
    /// Returns identity, reference, role, or in-use failures without partial mutation.
    pub fn insert_drawing_label(&mut self, value: DrawingLabel) -> Result<(), DrawingError> {
        self.drawing.insert_label(value)
    }
    /// Replace one drawing label through aggregate invariants.
    /// # Errors
    /// Returns identity, reference, role, or in-use failures without partial mutation.
    pub fn replace_drawing_label(
        &mut self,
        value: DrawingLabel,
    ) -> Result<DrawingLabel, DrawingError> {
        self.drawing.replace_label(value)
    }
    /// Remove one drawing label through aggregate invariants.
    /// # Errors
    /// Returns identity, reference, role, or in-use failures without partial mutation.
    pub fn remove_drawing_label(&mut self, id: LabelId) -> Result<DrawingLabel, DrawingError> {
        self.drawing.remove_label(id)
    }
    /// Insert one drawing polyline through aggregate invariants.
    /// # Errors
    /// Returns identity, reference, role, or in-use failures without partial mutation.
    pub fn insert_drawing_polyline(&mut self, value: DrawingPolyline) -> Result<(), DrawingError> {
        self.drawing.insert_polyline(value)
    }
    /// Replace one drawing polyline through aggregate invariants.
    /// # Errors
    /// Returns identity, reference, role, or in-use failures without partial mutation.
    pub fn replace_drawing_polyline(
        &mut self,
        value: DrawingPolyline,
    ) -> Result<DrawingPolyline, DrawingError> {
        self.drawing.replace_polyline(value)
    }
    /// Remove one drawing polyline through aggregate invariants.
    /// # Errors
    /// Returns identity, reference, role, or in-use failures without partial mutation.
    pub fn remove_drawing_polyline(
        &mut self,
        id: PolylineId,
    ) -> Result<DrawingPolyline, DrawingError> {
        self.drawing.remove_polyline(id)
    }
    /// Set shared style fallbacks and future-object presets without rewriting objects.
    /// Existing inherited values follow base changes; presets affect future creation only.
    /// # Errors
    /// Rejects invalid creation-preset style references/roles without mutation.
    pub fn set_drawing_settings(
        &mut self,
        settings: DrawingSettings,
    ) -> Result<DrawingSettings, DrawingError> {
        self.drawing.set_settings(settings)
    }
    /// Snapshot exact editable state, distinguishing inherited values from equal overrides.
    /// # Errors
    /// Missing/inapplicable targets fail without mutation.
    pub fn property_state(
        &self,
        target: ObjectRef,
        property: PropertyId,
    ) -> Result<PropertyState, ModelError> {
        let observation = self.property(target, property)?;
        if property.is_inheritable() {
            Ok(PropertyState::Override(
                self.drawing.override_value(target, property),
            ))
        } else {
            Ok(PropertyState::Field(observation.value))
        }
    }
    /// Restore literal state or per-property inheritance, returning its exact previous state.
    /// # Errors
    /// Rejects mismatched state kinds and normal field/reference/role validation failures.
    pub fn set_property_state(
        &mut self,
        target: ObjectRef,
        property: PropertyId,
        state: PropertyState,
    ) -> Result<PropertyState, ModelError> {
        let previous = self.property_state(target, property)?;
        match state {
            PropertyState::Override(value) if property.is_inheritable() => {
                self.drawing.set_override(target, property, value)?
            }
            PropertyState::Field(value) if !property.is_inheritable() => {
                self.set_property(target, property, value)?;
            }
            _ => return Err(PropertyError::NotInheritable(property).into()),
        }
        Ok(previous)
    }
}

impl Project {
    /// Insert a validated graphical picture without changing electrical content.
    /// # Errors
    /// Propagates drawing identity/view/geometry/budget errors without mutation.
    pub fn insert_drawing_picture(&mut self, picture: DrawingPicture) -> Result<(), DrawingError> {
        self.drawing.insert_picture(picture)
    }
    /// Replace one graphical picture with exact prior state returned for undo.
    /// # Errors
    /// Propagates drawing identity/view/geometry/budget errors without mutation.
    pub fn replace_drawing_picture(
        &mut self,
        picture: DrawingPicture,
    ) -> Result<DrawingPicture, DrawingError> {
        self.drawing.replace_picture(picture)
    }
    /// Remove a graphical picture while retaining all other placements.
    /// # Errors
    /// Rejects a missing identity.
    pub fn remove_drawing_picture(
        &mut self,
        id: PictureId,
    ) -> Result<DrawingPicture, DrawingError> {
        self.drawing.remove_picture(id)
    }
}
