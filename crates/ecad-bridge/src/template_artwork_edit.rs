//! Exact pointer intent for artwork, resolved against the captured immutable draft.
use crate::{dto::BridgeError, template_rows::RunRow};
use ecad_app::{ActionParameters, TemplateDocument};
use ecad_domain::*;
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArtworkEditRow {
    variant: String,
    graphic: String,
    edit: Edit,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Edit {
    // Both absent arms a creation tool without manufacturing an object or undo entry.
    start: Option<[String; 2]>,
    end: Option<[String; 2]>,
    // Typed unresolved runs are allowed only for text creation, never movement or vectors.
    contents: Option<Vec<RunRow>>,
    // Complete clicked vertices; hovering rubber segments never enter this proposal.
    points: Option<Vec<[String; 2]>>,
}
impl ArtworkEditRow {
    pub(crate) fn target(&self) -> Vec<String> {
        vec![self.variant.clone(), self.graphic.clone()]
    }
    pub(crate) fn proposal(
        self,
        doc: &TemplateDocument,
        kind: Option<TemplatePrimitive>,
    ) -> Result<Option<ActionParameters>, BridgeError> {
        let error = |e| BridgeError::new("template", e);
        let variant = self
            .variant
            .parse::<TemplateVariantId>()
            .map_err(|e| error(e.to_string()))?;
        let id = self
            .graphic
            .parse::<TemplateGraphicId>()
            .map_err(|e| error(e.to_string()))?;
        let old = doc
            .variant(variant)
            .ok_or_else(|| error("Template variant unavailable".into()))?;
        let mut graphics = old.layout().graphics().to_vec();
        let index = graphics.iter().position(|g| g.id == id);
        if kind.is_some() && index.is_some() {
            return Err(error("Artwork identity already exists".into()));
        }
        if kind.is_none() && index.is_none() {
            return Err(error("Artwork is unavailable".into()));
        }
        if self.edit.contents.is_some() && kind != Some(TemplatePrimitive::Text) {
            return Err(error("Text contents require the text creation tool".into()));
        }
        if let Some(points) = self.edit.points {
            if kind != Some(TemplatePrimitive::Polyline)
                || self.edit.start.is_some()
                || self.edit.end.is_some()
                || self.edit.contents.is_some()
            {
                return Err(error(
                    "Path vertices require the path tool without other geometry or contents".into(),
                ));
            }
            let points = points
                .into_iter()
                .map(|p| {
                    Ok([
                        p[0].parse::<i64>()
                            .map_err(|_| error("Use signed integer micrometres".into()))?,
                        p[1].parse::<i64>()
                            .map_err(|_| error("Use signed integer micrometres".into()))?,
                    ])
                })
                .collect::<Result<Vec<_>, BridgeError>>()?;
            graphics.push(
                TemplateGraphic::from_vertices(id, &points, old.layout().paper())
                    .map_err(|e| error(e.to_string()))?,
            );
        } else {
            if kind == Some(TemplatePrimitive::Polyline)
                && (self.edit.start.is_some() || self.edit.end.is_some())
            {
                return Err(error("Provide the complete path vertices".into()));
            }
            let (start, end) = match (self.edit.start, self.edit.end) {
                (None, None) if self.edit.contents.is_none() => return Ok(None),
                (None, None) => {
                    return Err(error("Place a text anchor before editing contents".into()));
                }
                (Some(a), Some(b)) => (a, b),
                _ => return Err(error("Both pointer coordinates are required".into())),
            };
            let parse = |p: [String; 2]| -> Result<[i64; 2], BridgeError> {
                Ok([
                    p[0].parse()
                        .map_err(|_| error("Use signed integer micrometres".into()))?,
                    p[1].parse()
                        .map_err(|_| error("Use signed integer micrometres".into()))?,
                ])
            };
            let start = parse(start)?;
            let end = parse(end)?;
            let paper = old.layout().paper();
            if let Some(kind) = kind {
                let mut graphic = TemplateGraphic::between(id, kind, start, end, paper)
                    .map_err(|e| error(e.to_string()))?;
                if let Some(contents) = self.edit.contents {
                    let runs = contents
                        .into_iter()
                        .map(TemplateTextRun::try_from)
                        .collect::<Result<Vec<_>, _>>()?;
                    if !runs.iter().any(|r| match r {
                        TemplateTextRun::Literal(text) => !text.trim().is_empty(),
                        TemplateTextRun::Field(_) => true,
                    }) {
                        return Err(error("Enter text or choose a template variable".into()));
                    }
                    graphic.shape = TemplateGraphicShape::Text(runs);
                }
                graphics.push(graphic);
            } else {
                let index = index.expect("validated existing object");
                graphics[index] = graphics[index]
                    .placed_at(end, paper)
                    .map_err(|e| error(e.to_string()))?;
            }
        }
        let layout = old
            .layout()
            .clone()
            .with_graphics(graphics)
            .map_err(|e| error(e.to_string()))?;
        Ok(Some(ActionParameters::TemplateEdit {
            variant,
            name: old.name().into(),
            layout: Box::new(layout),
        }))
    }
}
