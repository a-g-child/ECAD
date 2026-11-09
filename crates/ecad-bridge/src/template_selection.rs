//! Strict selection DTO and read-only contextual action eligibility projection.
use crate::{controller::DesktopBridge, dto::*};
use ecad_app::*;
use serde::Deserialize;
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum SelectionRow {
    Paper,
    Cells {
        variant: String,
        table: String,
        cells: Vec<String>,
    },
    Artwork {
        variant: String,
        id: String,
    },
}
impl SelectionRow {
    pub(crate) fn selection(self) -> Result<TemplateSelection, BridgeError> {
        let error = |e| BridgeError::new("selection", e);
        Ok(match self {
            Self::Paper => TemplateSelection::Paper,
            Self::Cells {
                variant,
                table,
                cells,
            } => TemplateSelection::Cells {
                variant: variant.parse().map_err(error)?,
                table: table.parse().map_err(error)?,
                cells: cells
                    .iter()
                    .map(|s| s.parse().map_err(error))
                    .collect::<Result<_, _>>()?,
            },
            Self::Artwork { variant, id } => TemplateSelection::Artwork {
                variant: variant.parse().map_err(error)?,
                id: id.parse().map_err(error)?,
            },
        })
    }
}
impl DesktopBridge {
    pub(crate) fn template_actions(
        &self,
        value: serde_json::Value,
    ) -> Result<Response, BridgeError> {
        let context = self.context(value)?;
        let doc = self
            .templates
            .editor
            .as_ref()
            .ok_or_else(|| BridgeError::new("template", "Template editor is closed"))?;
        let actions = ["template.cell.split", "template.cells.merge"]
            .into_iter()
            .map(|id| {
                let state = self.registry.template_state(
                    &ActionId::new(id).expect("static action"),
                    &context,
                    doc,
                );
                (
                    id.to_owned(),
                    state.disabled.is_none()
                        && !self.templates.pending
                        && self.templates.gesture.is_none(),
                )
            })
            .collect();
        Ok(Response {
            template_actions: Some(actions),
            ..Response::default()
        })
    }
}
