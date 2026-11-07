//! Project fallback editor transport; unrelated creation/style intent is retained verbatim.
use crate::dto::BridgeError;
use ecad_domain::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DefaultsRow {
    font: String,
    text_height: String,
    stroke_width: String,
    colour: String,
}
pub(crate) fn project(settings: &DrawingSettings) -> DefaultsRow {
    let base = &settings.base;
    DefaultsRow {
        font: base.font.name().into(),
        text_height: base.text_height.micrometres().to_string(),
        stroke_width: base.stroke_width.micrometres().to_string(),
        colour: format!(
            "#{:02x}{:02x}{:02x}",
            base.colour.red, base.colour.green, base.colour.blue
        ),
    }
}
pub(crate) fn replace(
    current: &DrawingSettings,
    value: serde_json::Value,
) -> Result<DrawingSettings, BridgeError> {
    let row: DefaultsRow =
        serde_json::from_value(value).map_err(|e| BridgeError::new("defaults", e))?;
    let length = |text: &str| {
        let value = text.parse().map_err(|_| {
            BridgeError::new(
                "defaults",
                "Use positive integer micrometres within the supported range",
            )
        })?;
        DrawingLength::new(value).map_err(|e| BridgeError::new("defaults", e))
    };
    let invalid_colour = || BridgeError::new("defaults", "Use #rrggbb for output colour");
    if row.colour.len() != 7 || !row.colour.starts_with('#') || !row.colour.is_ascii() {
        return Err(invalid_colour());
    }
    let value = u32::from_str_radix(&row.colour[1..], 16).map_err(|_| invalid_colour())?;
    let [_, red, green, blue] = value.to_be_bytes();
    let mut settings = current.clone();
    settings.base = DrawingValues {
        font: FontFamily::new(row.font).map_err(|e| BridgeError::new("defaults", e))?,
        text_height: length(&row.text_height)?,
        stroke_width: length(&row.stroke_width)?,
        colour: DrawingColour { red, green, blue },
    };
    Ok(settings)
}
