//! Strict artwork persistence DTOs, separate from domain and bridge representations.
use crate::TemplateFileError;
use crate::template_json::{RunData, coords, rect};
use ecad_domain::*;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GraphicData {
    id: String,
    bounds_um: [u32; 4],
    shape: ShapeData,
    font: String,
    text_height_um: u32,
    stroke_width_um: u32,
    colour: [u8; 3],
}
#[derive(Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum ShapeData {
    Text(Vec<RunData>),
    Rectangle,
    Ellipse,
    Polyline(Vec<[u32; 2]>),
}
impl From<&TemplateGraphic> for GraphicData {
    fn from(g: &TemplateGraphic) -> Self {
        Self {
            id: g.id.to_string(),
            bounds_um: coords(g.bounds),
            font: g.font.clone(),
            text_height_um: g.text_height,
            stroke_width_um: g.stroke_width,
            colour: g.colour,
            shape: match &g.shape {
                TemplateGraphicShape::Text(runs) => {
                    ShapeData::Text(runs.iter().map(RunData::from).collect())
                }
                TemplateGraphicShape::Rectangle => ShapeData::Rectangle,
                TemplateGraphicShape::Ellipse => ShapeData::Ellipse,
                TemplateGraphicShape::Polyline(p) => ShapeData::Polyline(p.clone()),
            },
        }
    }
}
impl TryFrom<GraphicData> for TemplateGraphic {
    type Error = TemplateFileError;
    fn try_from(g: GraphicData) -> Result<Self, Self::Error> {
        Ok(Self {
            id: g.id.parse()?,
            bounds: rect(g.bounds_um),
            font: g.font,
            text_height: g.text_height_um,
            stroke_width: g.stroke_width_um,
            colour: g.colour,
            shape: match g.shape {
                ShapeData::Text(runs) => TemplateGraphicShape::Text(
                    runs.into_iter()
                        .map(TemplateTextRun::try_from)
                        .collect::<Result<_, _>>()?,
                ),
                ShapeData::Rectangle => TemplateGraphicShape::Rectangle,
                ShapeData::Ellipse => TemplateGraphicShape::Ellipse,
                ShapeData::Polyline(p) => TemplateGraphicShape::Polyline(p),
            },
        })
    }
}
