//! Strict project-template settings transport; only immutable bundled identities are accepted.
use crate::dto::BridgeError;
use ecad_domain::{DrawingTemplateDefaults, SheetTemplate};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TemplateDefaultsRow {
    schematic: String,
    cover: String,
    drawing_list: String,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct TemplateCatalog {
    defaults: TemplateDefaultsRow,
    choices: Vec<(&'static str, &'static str)>,
}
impl TemplateCatalog {
    pub(crate) fn new(values: &DrawingTemplateDefaults) -> Self {
        Self {
            defaults: TemplateDefaultsRow {
                schematic: values.schematic.key().into(),
                cover: values.cover.key().into(),
                drawing_list: values.drawing_list.key().into(),
            },
            choices: SheetTemplate::ALL
                .into_iter()
                .map(|t| (t.key(), t.label()))
                .collect(),
        }
    }
}
pub(crate) fn parse(value: serde_json::Value) -> Result<DrawingTemplateDefaults, BridgeError> {
    let r: TemplateDefaultsRow =
        serde_json::from_value(value).map_err(|e| BridgeError::new("template", e))?;
    let template = |s: &str| {
        SheetTemplate::from_key(s)
            .ok_or_else(|| BridgeError::new("template", format!("Unknown bundled template: {s}")))
    };
    Ok(DrawingTemplateDefaults {
        schematic: template(&r.schematic)?,
        cover: template(&r.cover)?,
        drawing_list: template(&r.drawing_list)?,
    })
}
