//! Per-property text-block layout defaults and cell overrides; renderer measurement stays separate.
use crate::TemplateError;
/// Optional cell text layout properties. `None` uses the template/factory default.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TemplateCellTextLayout {
    /// Caption placement: `above` or `inline`.
    pub caption_position: Option<String>,
    /// Exact inline separator, including spaces; defaults to `: `.
    pub separator: Option<String>,
    /// Whole-block horizontal justification: left, center or right.
    pub horizontal: Option<String>,
    /// Whole-block vertical justification: top, middle or bottom.
    pub vertical: Option<String>,
}
impl TemplateCellTextLayout {
    /// Validate explicit layout values without resolving defaults.
    /// # Errors
    /// Rejects unknown positions/justification and multiline/control/overlong separators.
    pub fn validate(&self) -> Result<(), TemplateError> {
        if self
            .caption_position
            .as_deref()
            .is_some_and(|v| !["above", "inline"].contains(&v))
            || self
                .horizontal
                .as_deref()
                .is_some_and(|v| !["left", "center", "right"].contains(&v))
            || self
                .vertical
                .as_deref()
                .is_some_and(|v| !["top", "middle", "bottom"].contains(&v))
            || self
                .separator
                .as_ref()
                .is_some_and(|v| v.chars().count() > 64 || v.chars().any(char::is_control))
        {
            return Err(TemplateError::TextLayout);
        }
        Ok(())
    }
    /// Resolve each explicit cell value over its template default, retaining override intent.
    pub fn over(&self, base: &Self) -> Self {
        Self {
            caption_position: self
                .caption_position
                .clone()
                .or_else(|| base.caption_position.clone()),
            separator: self.separator.clone().or_else(|| base.separator.clone()),
            horizontal: self.horizontal.clone().or_else(|| base.horizontal.clone()),
            vertical: self.vertical.clone().or_else(|| base.vertical.clone()),
        }
    }
}
