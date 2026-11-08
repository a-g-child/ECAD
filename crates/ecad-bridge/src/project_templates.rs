//! Saved project-template catalogue and validated exact-reference settings transport.
use crate::dto::BridgeError;
use ecad_domain::*;
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
    choices: Vec<(String, String)>,
}
/// Exact saved paper dimensions for read-only sheet controls; never float editor values.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct SavedTemplateRow {
    pub key: String,
    pub new_assignments: bool,
    pub label: String,
    pub width: String,
    pub height: String,
    pub left: String,
    pub right: String,
    pub top: String,
    pub bottom: String,
}
pub(crate) fn choices(project: &Project) -> Vec<SavedTemplateRow> {
    project
        .template_revisions()
        .flat_map(|r| r.variants())
        .map(|v| {
            let p = v.layout().paper();
            SavedTemplateRow {
                key: SheetTemplate::Saved(v.reference()).key(),
                new_assignments: v.allows_new_assignments(),
                label: v.name().to_owned(),
                width: p.width.to_string(),
                height: p.height.to_string(),
                left: p.left.to_string(),
                right: p.right.to_string(),
                top: p.top.to_string(),
                bottom: p.bottom.to_string(),
            }
        })
        .collect()
}
impl TemplateCatalog {
    pub(crate) fn new(project: &Project) -> Self {
        let values = project.template_defaults();
        Self {
            defaults: TemplateDefaultsRow {
                schematic: values.schematic.key(),
                cover: values.cover.key(),
                drawing_list: values.drawing_list.key(),
            },
            choices: choices(project)
                .into_iter()
                .filter(|v| v.new_assignments)
                .map(|r| (r.key, r.label))
                .collect(),
        }
    }
}
pub(crate) fn parse(
    project: &Project,
    value: serde_json::Value,
) -> Result<DrawingTemplateDefaults, BridgeError> {
    let r: TemplateDefaultsRow =
        serde_json::from_value(value).map_err(|e| BridgeError::new("template", e))?;
    let template = |s: &str| {
        let Some(SheetTemplate::Saved(pin)) = SheetTemplate::from_key(s) else {
            return Err(BridgeError::new(
                "template",
                "Select a saved template variant",
            ));
        };
        project
            .saved_template(pin)
            .map_err(|e| BridgeError::new("template", e))?;
        if !project
            .saved_template(pin)
            .map_err(|e| BridgeError::new("template", e))?
            .allows_new_assignments()
        {
            return Err(BridgeError::new(
                "template",
                "Repair this retained legacy layout in a new revision before selecting it",
            ));
        }
        Ok(SheetTemplate::Saved(pin))
    };
    Ok(DrawingTemplateDefaults {
        schematic: template(&r.schematic)?,
        cover: template(&r.cover)?,
        drawing_list: template(&r.drawing_list)?,
    })
}
