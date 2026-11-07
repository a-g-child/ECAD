//! Strict project-v6 drawing DTOs; old v5 drawings remain independently decodable.

use crate::{
    ProjectFileError,
    schema_drawing::{DrawingData, LabelData, PolylineData, StyleData, ViewData},
    schema_drawing_values::{AppearanceData, SettingsData},
};
use ecad_domain::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AnchorData {
    id: String,
    view: String,
    terminal: String,
    position_um: [i64; 2],
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ModeData {
    Orthogonal,
    Free,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteData {
    id: String,
    view: String,
    conductor: String,
    anchors: [Option<String>; 2],
    points_um: Vec<[i64; 2]>,
    mode: ModeData,
    appearance: AppearanceData,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoutedDrawingData {
    settings: SettingsData,
    route_creation: AppearanceData,
    styles: Vec<StyleData>,
    views: Vec<ViewData>,
    labels: Vec<LabelData>,
    polylines: Vec<PolylineData>,
    anchors: Vec<AnchorData>,
    routes: Vec<RouteData>,
}
impl From<&DrawingDocument> for RoutedDrawingData {
    fn from(drawing: &DrawingDocument) -> Self {
        let base = DrawingData::from(drawing);
        Self {
            settings: base.settings,
            route_creation: (&drawing.settings().route_creation).into(),
            styles: base.styles,
            views: base.views,
            labels: base.labels,
            polylines: base.polylines,
            anchors: drawing
                .terminal_anchors()
                .map(|anchor| AnchorData {
                    id: anchor.id.to_string(),
                    view: anchor.view.to_string(),
                    terminal: anchor.terminal.to_string(),
                    position_um: [anchor.position.x, anchor.position.y],
                })
                .collect(),
            routes: drawing
                .routes()
                .map(|route| RouteData {
                    id: route.id.to_string(),
                    view: route.view.to_string(),
                    conductor: route.conductor.to_string(),
                    anchors: route.anchors.map(|id| id.map(|id| id.to_string())),
                    points_um: route
                        .points
                        .iter()
                        .map(|point| [point.x, point.y])
                        .collect(),
                    mode: match route.mode {
                        RouteMode::Orthogonal => ModeData::Orthogonal,
                        RouteMode::Free => ModeData::Free,
                    },
                    appearance: (&route.appearance).into(),
                })
                .collect(),
        }
    }
}
impl TryFrom<RoutedDrawingData> for DrawingDocument {
    type Error = ProjectFileError;
    fn try_from(data: RoutedDrawingData) -> Result<Self, Self::Error> {
        let mut drawing = Self::try_from(DrawingData {
            settings: data.settings,
            styles: data.styles,
            views: data.views,
            labels: data.labels,
            polylines: data.polylines,
        })?;
        let mut settings = drawing.settings().clone();
        settings.route_creation = data.route_creation.try_into()?;
        drawing.set_settings(settings)?;
        for anchor in data.anchors {
            drawing.insert_terminal_anchor(TerminalAnchor {
                id: anchor.id.parse()?,
                view: anchor.view.parse()?,
                terminal: anchor.terminal.parse()?,
                position: DrawingPoint {
                    x: anchor.position_um[0],
                    y: anchor.position_um[1],
                },
            })?;
        }
        for route in data.routes {
            drawing.insert_route(DrawingRoute {
                id: route.id.parse()?,
                view: route.view.parse()?,
                conductor: route.conductor.parse()?,
                anchors: [
                    route.anchors[0].as_ref().map(|id| id.parse()).transpose()?,
                    route.anchors[1].as_ref().map(|id| id.parse()).transpose()?,
                ],
                points: route
                    .points_um
                    .into_iter()
                    .map(|point| DrawingPoint {
                        x: point[0],
                        y: point[1],
                    })
                    .collect(),
                mode: match route.mode {
                    ModeData::Orthogonal => RouteMode::Orthogonal,
                    ModeData::Free => RouteMode::Free,
                },
                appearance: route.appearance.try_into()?,
            })?;
        }
        drawing.validate()?;
        Ok(drawing)
    }
}
