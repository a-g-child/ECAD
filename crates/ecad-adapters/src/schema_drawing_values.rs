//! Strict persistence values for drawing units, presets, and sparse override intent.

use crate::ProjectFileError;
use ecad_domain::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PatchData {
    font: Option<String>,
    text_height_um: Option<u32>,
    stroke_width_um: Option<u32>,
    colour: Option<[u8; 3]>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppearanceData {
    style: Option<String>,
    overrides: PatchData,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValuesData {
    font: String,
    text_height_um: u32,
    stroke_width_um: u32,
    colour: [u8; 3],
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SettingsData {
    base: ValuesData,
    text_creation: AppearanceData,
    stroke_creation: AppearanceData,
}
fn colour(value: [u8; 3]) -> DrawingColour {
    DrawingColour {
        red: value[0],
        green: value[1],
        blue: value[2],
    }
}
fn channels(value: DrawingColour) -> [u8; 3] {
    [value.red, value.green, value.blue]
}
impl From<&StylePatch> for PatchData {
    fn from(value: &StylePatch) -> Self {
        Self {
            font: value.font.as_ref().map(|font| font.name().to_owned()),
            text_height_um: value.text_height.map(DrawingLength::micrometres),
            stroke_width_um: value.stroke_width.map(DrawingLength::micrometres),
            colour: value.colour.map(channels),
        }
    }
}
impl TryFrom<PatchData> for StylePatch {
    type Error = ProjectFileError;
    fn try_from(value: PatchData) -> Result<Self, Self::Error> {
        Ok(Self {
            font: value.font.map(FontFamily::new).transpose()?,
            text_height: value.text_height_um.map(DrawingLength::new).transpose()?,
            stroke_width: value.stroke_width_um.map(DrawingLength::new).transpose()?,
            colour: value.colour.map(colour),
        })
    }
}
impl From<&Appearance> for AppearanceData {
    fn from(value: &Appearance) -> Self {
        Self {
            style: value.style.map(|id| id.to_string()),
            overrides: (&value.overrides).into(),
        }
    }
}
impl TryFrom<AppearanceData> for Appearance {
    type Error = ProjectFileError;
    fn try_from(value: AppearanceData) -> Result<Self, Self::Error> {
        Ok(Self {
            style: value.style.map(|id| id.parse()).transpose()?,
            overrides: value.overrides.try_into()?,
        })
    }
}
impl From<&DrawingSettings> for SettingsData {
    fn from(value: &DrawingSettings) -> Self {
        Self {
            base: ValuesData {
                font: value.base.font.name().to_owned(),
                text_height_um: value.base.text_height.micrometres(),
                stroke_width_um: value.base.stroke_width.micrometres(),
                colour: channels(value.base.colour),
            },
            text_creation: (&value.text_creation).into(),
            stroke_creation: (&value.stroke_creation).into(),
        }
    }
}
impl TryFrom<SettingsData> for DrawingSettings {
    type Error = ProjectFileError;
    fn try_from(value: SettingsData) -> Result<Self, Self::Error> {
        Ok(Self {
            base: DrawingValues {
                font: FontFamily::new(value.base.font)?,
                text_height: DrawingLength::new(value.base.text_height_um)?,
                stroke_width: DrawingLength::new(value.base.stroke_width_um)?,
                colour: colour(value.base.colour),
            },
            text_creation: value.text_creation.try_into()?,
            stroke_creation: value.stroke_creation.try_into()?,
            route_creation: Appearance::default(),
        })
    }
}
