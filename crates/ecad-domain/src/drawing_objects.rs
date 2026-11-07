//! Minimal real drawing carriers for styles; no electrical routing is implied.

use crate::{
    Appearance, DrawingError, DrawingPoint, DrawingRole, LabelId, PolylineId, StylePatch, ViewId,
};

/// A drawing scope, independent of electrical subsystem containment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingView {
    /// Stable view identity.
    pub id: ViewId,
    /// Display name; document insertion validates nonblank values.
    pub name: String,
    /// Per-property view overrides, shared across applicable graphical roles.
    pub overrides: StylePatch,
}
/// A free-standing text annotation. Device-linked/live labels are later work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingLabel {
    /// Stable annotation identity.
    pub id: LabelId,
    /// Existing drawing view.
    pub view: ViewId,
    /// Annotation text; insertion requires a nonblank value.
    pub text: String,
    /// Document location in micrometres.
    pub position: DrawingPoint,
    /// Style link and explicit per-property intent.
    pub appearance: Appearance,
}
/// One retained graphical path; it does not establish electrical connectivity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingPolyline {
    /// Stable graphical path identity.
    pub id: PolylineId,
    /// Existing drawing view.
    pub view: ViewId,
    /// Ordered whole-object geometry in document units.
    pub points: Vec<DrawingPoint>,
    /// Stroke style link and explicit per-property intent.
    pub appearance: Appearance,
}
impl DrawingLabel {
    pub(crate) fn validate(&self) -> Result<(), DrawingError> {
        if self.text.trim().is_empty() {
            return Err(DrawingError::EmptyName);
        }
        self.appearance.overrides.validate_role(DrawingRole::Text)
    }
}
impl DrawingPolyline {
    pub(crate) fn validate(&self) -> Result<(), DrawingError> {
        if self.points.len() < 2 || self.points.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(DrawingError::InvalidPath);
        }
        self.appearance.overrides.validate_role(DrawingRole::Stroke)
    }
}
