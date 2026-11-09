//! Versioned template text appearance storage, separate from domain and bridge types.
use ecad_domain::{TemplateTextStyle, TemplateTextStyles};
use serde::{Deserialize, Serialize};
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StylesData {
    caption: StyleData,
    content: StyleData,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StyleData {
    font: Option<String>,
    height: Option<u32>,
    bold: Option<bool>,
    italic: Option<bool>,
    colour: Option<[u8; 3]>,
    horizontal: Option<String>,
    vertical: Option<String>,
}
impl From<&TemplateTextStyle> for StyleData {
    fn from(s: &TemplateTextStyle) -> Self {
        Self {
            font: s.font.clone(),
            height: s.height,
            bold: s.bold,
            italic: s.italic,
            colour: s.colour,
            horizontal: s.horizontal.clone(),
            vertical: s.vertical.clone(),
        }
    }
}
impl From<StyleData> for TemplateTextStyle {
    fn from(s: StyleData) -> Self {
        Self {
            font: s.font,
            height: s.height,
            bold: s.bold,
            italic: s.italic,
            colour: s.colour,
            horizontal: s.horizontal,
            vertical: s.vertical,
        }
    }
}
impl From<&TemplateTextStyles> for StylesData {
    fn from(s: &TemplateTextStyles) -> Self {
        Self {
            caption: (&s.caption).into(),
            content: (&s.content).into(),
        }
    }
}
impl From<StylesData> for TemplateTextStyles {
    fn from(s: StylesData) -> Self {
        Self {
            caption: s.caption.into(),
            content: s.content.into(),
        }
    }
}
