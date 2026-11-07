//! Per-property style inheritance, explicit appearance intent, and creation presets.

use crate::{DrawingColour, DrawingError, DrawingLength, FontFamily, StyleId};

/// Graphical roles determine which properties a named style may define.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawingRole {
    /// Annotation font, text height, and colour.
    Text,
    /// Graphical stroke width and colour.
    Stroke,
}
/// Sparse overrides; absence means inherit independently for that property.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StylePatch {
    /// Font override for text.
    pub font: Option<FontFamily>,
    /// Text height override in document units.
    pub text_height: Option<DrawingLength>,
    /// Graphical stroke width, unrelated to conductor cross-section.
    pub stroke_width: Option<DrawingLength>,
    /// Output colour override.
    pub colour: Option<DrawingColour>,
}
impl StylePatch {
    /// Validate role applicability; view overrides may deliberately cover both roles.
    /// # Errors
    /// Rejects a text style carrying stroke width or a stroke style carrying font/height.
    pub fn validate_role(&self, role: DrawingRole) -> Result<(), DrawingError> {
        if (role == DrawingRole::Text && self.stroke_width.is_some())
            || (role == DrawingRole::Stroke && (self.font.is_some() || self.text_height.is_some()))
        {
            return Err(DrawingError::WrongRole);
        }
        Ok(())
    }
}
/// One object's style link and per-property overrides; copied from creation presets.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Appearance {
    /// Optional project-owned named style, resolved below view overrides.
    pub style: Option<StyleId>,
    /// Explicit values retain intent even if equal to inherited values.
    pub overrides: StylePatch,
}
/// Complete project role fallbacks. Updating these is a shared-style operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingValues {
    /// Portable font request for text without a more specific value.
    pub font: FontFamily,
    /// Default inherited text height in document units.
    pub text_height: DrawingLength,
    /// Default inherited graphical stroke width.
    pub stroke_width: DrawingLength,
    /// Default inherited output colour.
    pub colour: DrawingColour,
}
impl Default for DrawingValues {
    fn default() -> Self {
        Self {
            font: FontFamily::new("sans-serif").expect("nonblank constant"),
            text_height: DrawingLength::new(2_500).expect("positive constant"),
            stroke_width: DrawingLength::new(250).expect("positive constant"),
            colour: DrawingColour::default(),
        }
    }
}
/// Portable project settings: linked style fallbacks and separate future-object presets.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DrawingSettings {
    /// Shared inherited project values; existing unoverridden objects follow changes.
    pub base: DrawingValues,
    /// Initial style link/overrides copied into newly created annotations only.
    pub text_creation: Appearance,
    /// Initial style link/overrides copied into newly created polylines only.
    pub stroke_creation: Appearance,
    /// Initial appearance for conductor routes, independent of graphical polyline presets.
    pub route_creation: Appearance,
}
/// An immutable-value proposal for one project-owned named style.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingStyle {
    id: StyleId,
    name: String,
    role: DrawingRole,
    patch: StylePatch,
}
impl DrawingStyle {
    /// Construct a named sparse style for one role.
    /// # Errors
    /// Rejects blank names and properties belonging to the other role.
    pub fn new(
        id: StyleId,
        name: impl Into<String>,
        role: DrawingRole,
        patch: StylePatch,
    ) -> Result<Self, DrawingError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DrawingError::EmptyName);
        }
        patch.validate_role(role)?;
        Ok(Self {
            id,
            name,
            role,
            patch,
        })
    }
    /// Stable style identity, independent of its display name.
    pub fn id(&self) -> StyleId {
        self.id
    }
    /// Display name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Graphical applicability.
    pub fn role(&self) -> DrawingRole {
        self.role
    }
    /// Sparse values resolved independently per property.
    pub fn patch(&self) -> &StylePatch {
        &self.patch
    }
}
