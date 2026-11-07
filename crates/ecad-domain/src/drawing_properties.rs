//! Per-property resolution and typed drawing edits with preserved override intent.

use super::DrawingDocument;
use crate::*;

fn patch_value(patch: &StylePatch, property: PropertyId) -> Option<PropertyValue> {
    match property {
        PropertyId::FontFamily => patch
            .font
            .as_ref()
            .map(|font| PropertyValue::Text(font.name().to_owned())),
        PropertyId::TextHeight => patch.text_height.map(PropertyValue::Length),
        PropertyId::StrokeWidth => patch.stroke_width.map(PropertyValue::Length),
        PropertyId::DrawingColour => patch.colour.map(PropertyValue::Colour),
        _ => None,
    }
}
fn set_patch(
    patch: &mut StylePatch,
    property: PropertyId,
    value: Option<PropertyValue>,
) -> Result<(), ModelError> {
    let wrong = || {
        ModelError::from(PropertyError::WrongType {
            property,
            expected: property.descriptor().value_kind,
        })
    };
    match (property, value) {
        (PropertyId::FontFamily, None) => patch.font = None,
        (PropertyId::FontFamily, Some(PropertyValue::Text(font))) => {
            patch.font = Some(FontFamily::new(font)?)
        }
        (PropertyId::TextHeight, None) => patch.text_height = None,
        (PropertyId::TextHeight, Some(PropertyValue::Length(length))) => {
            patch.text_height = Some(length)
        }
        (PropertyId::StrokeWidth, None) => patch.stroke_width = None,
        (PropertyId::StrokeWidth, Some(PropertyValue::Length(length))) => {
            patch.stroke_width = Some(length)
        }
        (PropertyId::DrawingColour, None) => patch.colour = None,
        (PropertyId::DrawingColour, Some(PropertyValue::Colour(colour))) => {
            patch.colour = Some(colour)
        }
        _ => return Err(wrong()),
    }
    Ok(())
}
impl DrawingDocument {
    fn appearance(&self, target: ObjectRef) -> Option<(&Appearance, ViewId)> {
        match target {
            ObjectRef::Route(id) => self
                .routes
                .get(&id)
                .map(|route| (&route.appearance, route.view)),
            ObjectRef::Label(id) => self.labels.get(&id).map(|o| (&o.appearance, o.view)),
            ObjectRef::Polyline(id) => self.polylines.get(&id).map(|o| (&o.appearance, o.view)),
            _ => None,
        }
    }
    pub(crate) fn property(
        &self,
        target: ObjectRef,
        property: PropertyId,
    ) -> Result<PropertyObservation, ModelError> {
        if !property.applies_to(target) {
            return Err(PropertyError::NotApplicable { target, property }.into());
        }
        if property.is_inheritable() {
            return self.resolve(target, property);
        }
        let value = match (target, property) {
            (ObjectRef::Anchor(id), PropertyId::AnchorTerminal) => {
                PropertyValue::Object(ObjectRef::Terminal(
                    self.terminal_anchor(id)
                        .ok_or(RouteError::MissingAnchor(id))?
                        .terminal,
                ))
            }
            (ObjectRef::Anchor(id), PropertyId::DrawingPosition) => PropertyValue::Point(
                self.terminal_anchor(id)
                    .ok_or(RouteError::MissingAnchor(id))?
                    .position,
            ),
            (ObjectRef::Anchor(id), PropertyId::DrawingView) => {
                PropertyValue::Object(ObjectRef::View(
                    self.terminal_anchor(id)
                        .ok_or(RouteError::MissingAnchor(id))?
                        .view,
                ))
            }
            (ObjectRef::Route(id), PropertyId::RouteConductor) => {
                PropertyValue::Object(ObjectRef::Conductor(
                    self.route(id)
                        .ok_or(RouteError::MissingRoute(id))?
                        .conductor,
                ))
            }
            (ObjectRef::Route(id), PropertyId::RouteAnchorA | PropertyId::RouteAnchorB) => {
                PropertyValue::OptionalObject(
                    self.route(id).ok_or(RouteError::MissingRoute(id))?.anchors[if property
                        == PropertyId::RouteAnchorA
                    {
                        0
                    } else {
                        1
                    }]
                    .map(ObjectRef::Anchor),
                )
            }
            (ObjectRef::Route(id), PropertyId::RouteMode) => {
                PropertyValue::RouteMode(self.route(id).ok_or(RouteError::MissingRoute(id))?.mode)
            }
            (ObjectRef::Route(id), PropertyId::DrawingVertices) => PropertyValue::Points(
                self.route(id)
                    .ok_or(RouteError::MissingRoute(id))?
                    .points
                    .clone(),
            ),
            (_, PropertyId::Identity) => PropertyValue::Object(target),
            (ObjectRef::View(id), PropertyId::Name) => PropertyValue::Text(
                self.views
                    .get(&id)
                    .ok_or(DrawingError::MissingView(id))?
                    .name
                    .clone(),
            ),
            (ObjectRef::Label(id), PropertyId::DrawingText) => PropertyValue::Text(
                self.labels
                    .get(&id)
                    .ok_or(DrawingError::MissingLabel(id))?
                    .text
                    .clone(),
            ),
            (ObjectRef::Label(id), PropertyId::DrawingPosition) => PropertyValue::Point(
                self.labels
                    .get(&id)
                    .ok_or(DrawingError::MissingLabel(id))?
                    .position,
            ),
            (ObjectRef::Polyline(id), PropertyId::DrawingVertices) => PropertyValue::Points(
                self.polylines
                    .get(&id)
                    .ok_or(DrawingError::MissingPolyline(id))?
                    .points
                    .clone(),
            ),
            (_, PropertyId::DrawingView) => PropertyValue::Object(ObjectRef::View(
                self.appearance(target)
                    .ok_or(PropertyError::NotApplicable { target, property })?
                    .1,
            )),
            (_, PropertyId::DrawingStyle) => PropertyValue::Style(
                self.appearance(target)
                    .ok_or(PropertyError::NotApplicable { target, property })?
                    .0
                    .style,
            ),
            _ => return Err(PropertyError::NotApplicable { target, property }.into()),
        };
        Ok(PropertyObservation {
            value,
            access: if property == PropertyId::Identity {
                PropertyAccess::ReadOnly(PropertyRestriction::Identity)
            } else if !property.descriptor().editable || matches!(target, ObjectRef::Anchor(_)) {
                PropertyAccess::ReadOnly(PropertyRestriction::StructuralOperation)
            } else {
                PropertyAccess::Editable
            },
            origin: None,
        })
    }
    fn resolve(
        &self,
        target: ObjectRef,
        property: PropertyId,
    ) -> Result<PropertyObservation, ModelError> {
        let (appearance, view_id) = self
            .appearance(target)
            .ok_or(PropertyError::NotApplicable { target, property })?;
        let view = self
            .views
            .get(&view_id)
            .ok_or(DrawingError::MissingView(view_id))?;
        let mut candidates = vec![
            (&appearance.overrides, PropertyOrigin::Object),
            (&view.overrides, PropertyOrigin::View(view_id)),
        ];
        if let Some(id) = appearance.style {
            candidates.push((
                self.styles
                    .get(&id)
                    .ok_or(DrawingError::MissingStyle(id))?
                    .patch(),
                PropertyOrigin::NamedStyle(id),
            ));
        }
        for (patch, origin) in candidates {
            if let Some(value) = patch_value(patch, property) {
                return Ok(PropertyObservation {
                    value,
                    access: PropertyAccess::Editable,
                    origin: Some(origin),
                });
            }
        }
        let base = &self.settings.base;
        let value = match property {
            PropertyId::FontFamily => PropertyValue::Text(base.font.name().to_owned()),
            PropertyId::TextHeight => PropertyValue::Length(base.text_height),
            PropertyId::StrokeWidth => PropertyValue::Length(base.stroke_width),
            PropertyId::DrawingColour => PropertyValue::Colour(base.colour),
            _ => return Err(PropertyError::NotInheritable(property).into()),
        };
        Ok(PropertyObservation {
            value,
            access: PropertyAccess::Editable,
            origin: Some(PropertyOrigin::Project),
        })
    }
    pub(crate) fn override_value(
        &self,
        target: ObjectRef,
        property: PropertyId,
    ) -> Option<PropertyValue> {
        self.appearance(target)
            .and_then(|(appearance, _)| patch_value(&appearance.overrides, property))
    }
    pub(crate) fn set_override(
        &mut self,
        target: ObjectRef,
        property: PropertyId,
        value: Option<PropertyValue>,
    ) -> Result<(), ModelError> {
        self.property(target, property)?;
        if !property.is_inheritable() {
            return Err(PropertyError::NotInheritable(property).into());
        }
        match target {
            ObjectRef::Route(id) => {
                let mut object = self
                    .routes
                    .get(&id)
                    .ok_or(RouteError::MissingRoute(id))?
                    .clone();
                set_patch(&mut object.appearance.overrides, property, value)?;
                self.replace_route(object)?;
            }
            ObjectRef::Label(id) => {
                let mut object = self
                    .labels
                    .get(&id)
                    .ok_or(DrawingError::MissingLabel(id))?
                    .clone();
                set_patch(&mut object.appearance.overrides, property, value)?;
                self.replace_label(object)?;
            }
            ObjectRef::Polyline(id) => {
                let mut object = self
                    .polylines
                    .get(&id)
                    .ok_or(DrawingError::MissingPolyline(id))?
                    .clone();
                set_patch(&mut object.appearance.overrides, property, value)?;
                self.replace_polyline(object)?;
            }
            _ => return Err(PropertyError::NotApplicable { target, property }.into()),
        }
        Ok(())
    }
    pub(crate) fn set_property(
        &mut self,
        target: ObjectRef,
        property: PropertyId,
        value: PropertyValue,
    ) -> Result<PropertyValue, ModelError> {
        let previous = self.property(target, property)?;
        if !property.descriptor().accepts_value(&value) {
            return Err(PropertyError::WrongType {
                property,
                expected: property.descriptor().value_kind,
            }
            .into());
        }
        if property.is_inheritable() {
            self.set_override(target, property, Some(value))?;
            return Ok(previous.value);
        }
        if previous.value == value {
            return Ok(previous.value);
        }
        if let PropertyAccess::ReadOnly(reason) = previous.access {
            return Err(PropertyError::ReadOnly {
                target,
                property,
                reason,
            }
            .into());
        }
        match target {
            ObjectRef::Route(id) => {
                let mut object = self
                    .routes
                    .get(&id)
                    .ok_or(RouteError::MissingRoute(id))?
                    .clone();
                match value {
                    PropertyValue::Points(points) => object.points = points,
                    PropertyValue::Object(ObjectRef::View(view)) => object.view = view,
                    PropertyValue::Style(style) => object.appearance.style = style,
                    _ => unreachable!("descriptor validated route field"),
                }
                self.replace_route(object)?;
            }
            ObjectRef::View(id) => {
                let mut object = self
                    .views
                    .get(&id)
                    .ok_or(DrawingError::MissingView(id))?
                    .clone();
                if let PropertyValue::Text(text) = value {
                    object.name = text;
                }
                self.replace_view(object)?;
            }
            ObjectRef::Label(id) => {
                let mut object = self
                    .labels
                    .get(&id)
                    .ok_or(DrawingError::MissingLabel(id))?
                    .clone();
                match value {
                    PropertyValue::Text(text) => object.text = text,
                    PropertyValue::Point(point) => object.position = point,
                    PropertyValue::Object(ObjectRef::View(view)) => object.view = view,
                    PropertyValue::Style(style) => object.appearance.style = style,
                    _ => unreachable!("descriptor validated label field"),
                }
                self.replace_label(object)?;
            }
            ObjectRef::Polyline(id) => {
                let mut object = self
                    .polylines
                    .get(&id)
                    .ok_or(DrawingError::MissingPolyline(id))?
                    .clone();
                match value {
                    PropertyValue::Points(points) => object.points = points,
                    PropertyValue::Object(ObjectRef::View(view)) => object.view = view,
                    PropertyValue::Style(style) => object.appearance.style = style,
                    _ => unreachable!("descriptor validated path field"),
                }
                self.replace_polyline(object)?;
            }
            _ => return Err(PropertyError::NotApplicable { target, property }.into()),
        }
        Ok(previous.value)
    }
}
