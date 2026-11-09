//! Exact editor appearance DTOs; optional fields retain inheritance intent.
use crate::dto::BridgeError;
use ecad_domain::{TemplateCellTextLayout, TemplateTextStyle, TemplateTextStyles};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StylesRow {
    #[serde(default)]
    layout: LayoutRow,
    caption: StyleRow,
    content: StyleRow,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StyleRow {
    font: Option<String>,
    height: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    colour: Option<String>,
    horizontal: Option<String>,
    vertical: Option<String>,
}
impl From<&TemplateTextStyle> for StyleRow {
    fn from(s: &TemplateTextStyle) -> Self {
        Self {
            font: s.font.clone(),
            height: s.height.map(|v| v.to_string()),
            bold: s.bold,
            italic: s.italic,
            colour: s
                .colour
                .map(|c| format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])),
            horizontal: s.horizontal.clone(),
            vertical: s.vertical.clone(),
        }
    }
}
impl TryFrom<StyleRow> for TemplateTextStyle {
    type Error = BridgeError;
    fn try_from(s: StyleRow) -> Result<Self, Self::Error> {
        let error = || {
            BridgeError::new(
                "template",
                "Invalid text style: use a positive height, font, colour and supported alignment",
            )
        };
        let height = s
            .height
            .map(|v| v.parse::<u32>().map_err(|_| error()))
            .transpose()?;
        let colour = s
            .colour
            .map(|v| {
                if v.len() != 7
                    || !v.starts_with('#')
                    || !v[1..].bytes().all(|b| b.is_ascii_hexdigit())
                {
                    return Err(error());
                }
                let n = u32::from_str_radix(&v[1..], 16).map_err(|_| error())?;
                Ok([
                    ((n >> 16) & 255) as u8,
                    ((n >> 8) & 255) as u8,
                    (n & 255) as u8,
                ])
            })
            .transpose()?;
        let style = Self {
            font: s.font,
            height,
            bold: s.bold,
            italic: s.italic,
            colour,
            horizontal: s.horizontal,
            vertical: s.vertical,
        };
        style.validate().map_err(|_| error())?;
        Ok(style)
    }
}
impl From<&TemplateTextStyles> for StylesRow {
    fn from(s: &TemplateTextStyles) -> Self {
        Self {
            layout: (&s.layout).into(),
            caption: (&s.caption).into(),
            content: (&s.content).into(),
        }
    }
}
impl TryFrom<StylesRow> for TemplateTextStyles {
    type Error = BridgeError;
    fn try_from(s: StylesRow) -> Result<Self, Self::Error> {
        let layout: TemplateCellTextLayout = s.layout.into();
        layout
            .validate()
            .map_err(|e| BridgeError::new("template", e))?;
        Ok(Self {
            layout,
            caption: s.caption.try_into()?,
            content: s.content.try_into()?,
        })
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LayoutRow {
    caption_position: Option<String>,
    separator: Option<String>,
    horizontal: Option<String>,
    vertical: Option<String>,
}
impl From<&TemplateCellTextLayout> for LayoutRow {
    fn from(v: &TemplateCellTextLayout) -> Self {
        Self {
            caption_position: v.caption_position.clone(),
            separator: v.separator.clone(),
            horizontal: v.horizontal.clone(),
            vertical: v.vertical.clone(),
        }
    }
}
impl From<LayoutRow> for TemplateCellTextLayout {
    fn from(v: LayoutRow) -> Self {
        Self {
            caption_position: v.caption_position,
            separator: v.separator,
            horizontal: v.horizontal,
            vertical: v.vertical,
        }
    }
}
