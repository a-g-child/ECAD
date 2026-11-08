//! Exact editable artwork DTOs; neither persistence structs nor authoritative geometry.
use crate::{dto::BridgeError, template_rows::RunRow};
use ecad_domain::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GraphicRow {
    id: String,
    kind: String,
    bounds: [String; 4],
    contents: Vec<RunRow>,
    points: Vec<[String; 2]>,
    font: String,
    text_height: String,
    stroke_width: String,
    colour: String,
}
impl From<&TemplateGraphic> for GraphicRow {
    fn from(g: &TemplateGraphic) -> Self {
        let (kind, contents, points) = match &g.shape {
            TemplateGraphicShape::Text(r) => ("text", r.iter().map(RunRow::from).collect(), vec![]),
            TemplateGraphicShape::Polyline(p) => (
                "polyline",
                vec![],
                p.iter().map(|p| p.map(|v| v.to_string())).collect(),
            ),
            TemplateGraphicShape::Rectangle => ("rectangle", vec![], vec![]),
            TemplateGraphicShape::Ellipse => ("ellipse", vec![], vec![]),
        };
        Self {
            id: g.id.to_string(),
            kind: kind.into(),
            bounds: [g.bounds.x, g.bounds.y, g.bounds.width, g.bounds.height]
                .map(|v| v.to_string()),
            contents,
            points,
            font: g.font.clone(),
            text_height: g.text_height.to_string(),
            stroke_width: g.stroke_width.to_string(),
            colour: format!("#{:02x}{:02x}{:02x}", g.colour[0], g.colour[1], g.colour[2]),
        }
    }
}
impl TryFrom<GraphicRow> for TemplateGraphic {
    type Error = BridgeError;
    fn try_from(g: GraphicRow) -> Result<Self, Self::Error> {
        let error = || {
            BridgeError::new(
                "template",
                "Invalid artwork: use integer micrometres, a known shape and #RRGGBB colour",
            )
        };
        let parse = |s: &str| s.parse::<u32>().map_err(|_| error());
        let b = g
            .bounds
            .iter()
            .map(|s| parse(s))
            .collect::<Result<Vec<_>, _>>()?;
        if g.colour.len() != 7
            || !g.colour.starts_with('#')
            || !g.colour[1..].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(error());
        }
        let rgb = u32::from_str_radix(&g.colour[1..], 16)
            .map_err(|_| error())?
            .to_be_bytes();
        if (g.kind != "text" && !g.contents.is_empty())
            || (g.kind != "polyline" && !g.points.is_empty())
        {
            return Err(error());
        }
        let shape = match g.kind.as_str() {
            "text" => TemplateGraphicShape::Text(
                g.contents
                    .into_iter()
                    .map(TemplateTextRun::try_from)
                    .collect::<Result<_, _>>()?,
            ),
            "rectangle" => TemplateGraphicShape::Rectangle,
            "ellipse" => TemplateGraphicShape::Ellipse,
            "polyline" => TemplateGraphicShape::Polyline(
                g.points
                    .iter()
                    .map(|p| Ok([parse(&p[0])?, parse(&p[1])?]))
                    .collect::<Result<_, BridgeError>>()?,
            ),
            _ => return Err(error()),
        };
        Ok(Self {
            id: g.id.parse().map_err(|_| error())?,
            bounds: FrameRect {
                x: b[0],
                y: b[1],
                width: b[2],
                height: b[3],
            },
            shape,
            font: g.font,
            text_height: parse(&g.text_height)?,
            stroke_width: parse(&g.stroke_width)?,
            colour: [rgb[1], rgb[2], rgb[3]],
        })
    }
}
impl From<&TemplateTextRun> for RunRow {
    fn from(r: &TemplateTextRun) -> Self {
        match r {
            TemplateTextRun::Literal(s) => Self {
                kind: "literal".into(),
                value: s.clone(),
            },
            TemplateTextRun::Field(f) => Self {
                kind: "field".into(),
                value: f.key().into(),
            },
        }
    }
}
impl TryFrom<RunRow> for TemplateTextRun {
    type Error = BridgeError;
    fn try_from(r: RunRow) -> Result<Self, Self::Error> {
        match r.kind.as_str() {
            "literal" => Ok(Self::Literal(r.value)),
            "field" => FrameField::from_key(&r.value)
                .map(Self::Field)
                .ok_or_else(|| BridgeError::new("template", "Unknown template variable")),
            _ => Err(BridgeError::new("template", "Unknown text content type")),
        }
    }
}
