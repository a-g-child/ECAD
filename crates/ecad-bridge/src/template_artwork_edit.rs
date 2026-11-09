//! Exact pointer intent for artwork, resolved against the captured immutable draft.
use crate::dto::BridgeError;
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
        let (start, end) = match (self.edit.start, self.edit.end) {
            (None, None) => return Ok(None),
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
            graphics.push(
                TemplateGraphic::between(id, kind, start, end, paper)
                    .map_err(|e| error(e.to_string()))?,
            );
        } else {
            let index = index.expect("validated existing object");
            graphics[index] = graphics[index]
                .placed_at(end, paper)
                .map_err(|e| error(e.to_string()))?;
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
