//! Exact editor appearance DTOs; optional fields retain inheritance intent.
use crate::dto::BridgeError;
use ecad_domain::{TemplateTextStyle, TemplateTextStyles};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StylesRow {
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
            caption: (&s.caption).into(),
            content: (&s.content).into(),
        }
    }
}
impl TryFrom<StylesRow> for TemplateTextStyles {
    type Error = BridgeError;
    fn try_from(s: StylesRow) -> Result<Self, Self::Error> {
        Ok(Self {
            caption: s.caption.try_into()?,
            content: s.content.try_into()?,
        })
    }
}
