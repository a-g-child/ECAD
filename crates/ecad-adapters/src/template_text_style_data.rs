//! Versioned template text appearance storage, separate from domain and bridge types.
use ecad_domain::{TemplateCellTextLayout, TemplateTextStyle, TemplateTextStyles};
use serde::{Deserialize, Serialize};
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StylesData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    layout: Option<LayoutData>,
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
            layout: (s.layout != TemplateCellTextLayout::default()).then(|| (&s.layout).into()),
            caption: (&s.caption).into(),
            content: (&s.content).into(),
        }
    }
}
impl From<StylesData> for TemplateTextStyles {
    fn from(s: StylesData) -> Self {
        Self {
            layout: s.layout.map(Into::into).unwrap_or_default(),
            caption: s.caption.into(),
            content: s.content.into(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LayoutData {
    caption_position: Option<String>,
    separator: Option<String>,
    horizontal: Option<String>,
    vertical: Option<String>,
}
impl From<&TemplateCellTextLayout> for LayoutData {
    fn from(v: &TemplateCellTextLayout) -> Self {
        Self {
            caption_position: v.caption_position.clone(),
            separator: v.separator.clone(),
            horizontal: v.horizontal.clone(),
            vertical: v.vertical.clone(),
        }
    }
}
impl From<LayoutData> for TemplateCellTextLayout {
    fn from(v: LayoutData) -> Self {
        Self {
            caption_position: v.caption_position,
            separator: v.separator,
            horizontal: v.horizontal,
            vertical: v.vertical,
        }
    }
}

impl StylesData {
    pub(crate) fn has_layout(&self) -> bool {
        self.layout.is_some()
    }
}
