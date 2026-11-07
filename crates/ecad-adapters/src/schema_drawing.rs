//! Strict drawing content DTOs; validate all identity/reference/role constraints on load.

use crate::{
    ProjectFileError,
    schema_drawing_values::{AppearanceData, PatchData, SettingsData},
};
use ecad_domain::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RoleData {
    Text,
    Stroke,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StyleData {
    id: String,
    name: String,
    role: RoleData,
    values: PatchData,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewData {
    id: String,
    name: String,
    overrides: PatchData,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LabelData {
    id: String,
    view: String,
    text: String,
    position_um: [i64; 2],
    appearance: AppearanceData,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolylineData {
    id: String,
    view: String,
    points_um: Vec<[i64; 2]>,
    appearance: AppearanceData,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DrawingData {
    settings: SettingsData,
    styles: Vec<StyleData>,
    views: Vec<ViewData>,
    labels: Vec<LabelData>,
    polylines: Vec<PolylineData>,
}
impl From<&DrawingDocument> for DrawingData {
    fn from(drawing: &DrawingDocument) -> Self {
        Self {
            settings: drawing.settings().into(),
            styles: drawing
                .styles()
                .map(|style| StyleData {
                    id: style.id().to_string(),
                    name: style.name().to_owned(),
                    role: match style.role() {
                        DrawingRole::Text => RoleData::Text,
                        DrawingRole::Stroke => RoleData::Stroke,
                    },
                    values: style.patch().into(),
                })
                .collect(),
            views: drawing
                .views()
                .map(|view| ViewData {
                    id: view.id.to_string(),
                    name: view.name.clone(),
                    overrides: (&view.overrides).into(),
                })
                .collect(),
            labels: drawing
                .labels()
                .map(|label| LabelData {
                    id: label.id.to_string(),
                    view: label.view.to_string(),
                    text: label.text.clone(),
                    position_um: [label.position.x, label.position.y],
                    appearance: (&label.appearance).into(),
                })
                .collect(),
            polylines: drawing
                .polylines()
                .map(|line| PolylineData {
                    id: line.id.to_string(),
                    view: line.view.to_string(),
                    points_um: line.points.iter().map(|point| [point.x, point.y]).collect(),
                    appearance: (&line.appearance).into(),
                })
                .collect(),
        }
    }
}
impl TryFrom<DrawingData> for DrawingDocument {
    type Error = ProjectFileError;
    fn try_from(data: DrawingData) -> Result<Self, Self::Error> {
        let mut drawing = Self::default();
        for style in data.styles {
            drawing.insert_style(DrawingStyle::new(
                style.id.parse()?,
                style.name,
                match style.role {
                    RoleData::Text => DrawingRole::Text,
                    RoleData::Stroke => DrawingRole::Stroke,
                },
                style.values.try_into()?,
            )?)?;
        }
        for view in data.views {
            drawing.insert_view(DrawingView {
                id: view.id.parse()?,
                name: view.name,
                overrides: view.overrides.try_into()?,
            })?;
        }
        for label in data.labels {
            drawing.insert_label(DrawingLabel {
                id: label.id.parse()?,
                view: label.view.parse()?,
                text: label.text,
                position: DrawingPoint {
                    x: label.position_um[0],
                    y: label.position_um[1],
                },
                appearance: label.appearance.try_into()?,
            })?;
        }
        for line in data.polylines {
            drawing.insert_polyline(DrawingPolyline {
                id: line.id.parse()?,
                view: line.view.parse()?,
                points: line
                    .points_um
                    .into_iter()
                    .map(|point| DrawingPoint {
                        x: point[0],
                        y: point[1],
                    })
                    .collect(),
                appearance: line.appearance.try_into()?,
            })?;
        }
        drawing.set_settings(data.settings.try_into()?)?;
        drawing.validate()?;
        Ok(drawing)
    }
}
