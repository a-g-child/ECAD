//! Per-property caption/content overrides, independent of font measurement and widgets.
use crate::TemplateError;

/// Optional text properties. `None` inherits; equal-valued explicit overrides stay explicit.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TemplateTextStyle {
    /// Font family, when explicitly overridden.
    pub font: Option<String>,
    /// Positive physical glyph height in micrometres.
    pub height: Option<u32>,
    /// Explicit bold intent, including normal weight.
    pub bold: Option<bool>,
    /// Explicit italic intent, including upright text.
    pub italic: Option<bool>,
    /// RGB text colour.
    pub colour: Option<[u8; 3]>,
    /// Horizontal alignment: left, center or right.
    pub horizontal: Option<String>,
    /// Vertical alignment: top, middle or bottom.
    pub vertical: Option<String>,
}
impl TemplateTextStyle {
    /// Validate explicit values without resolving inherited appearance.
    /// # Errors
    /// Rejects blank fonts, zero heights and unknown alignment values.
    pub fn validate(&self) -> Result<(), TemplateError> {
        if self.font.as_ref().is_some_and(|v| v.trim().is_empty())
            || self.height == Some(0)
            || self
                .horizontal
                .as_deref()
                .is_some_and(|v| !["left", "center", "right"].contains(&v))
            || self
                .vertical
                .as_deref()
                .is_some_and(|v| !["top", "middle", "bottom"].contains(&v))
        {
            return Err(TemplateError::Graphic);
        }
        Ok(())
    }
    /// Resolve each explicit property over the supplied lower-priority defaults.
    pub fn over(&self, base: &Self) -> Self {
        Self {
            font: self.font.clone().or_else(|| base.font.clone()),
            height: self.height.or(base.height),
            bold: self.bold.or(base.bold),
            italic: self.italic.or(base.italic),
            colour: self.colour.or(base.colour),
            horizontal: self.horizontal.clone().or_else(|| base.horizontal.clone()),
            vertical: self.vertical.clone().or_else(|| base.vertical.clone()),
        }
    }
}
/// Separate caption and content style channels, used by both defaults and cells.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TemplateTextStyles {
    /// Caption appearance overrides.
    pub caption: TemplateTextStyle,
    /// Content appearance overrides, shared by literals and resolved variables.
    pub content: TemplateTextStyle,
}
impl TemplateTextStyles {
    /// Validate both style channels atomically.
    /// # Errors
    /// Returns an error if either channel contains invalid explicit properties.
    pub fn validate(&self) -> Result<(), TemplateError> {
        self.caption.validate()?;
        self.content.validate()
    }
}
