//! Retained validated pointer proposals; commit never reruns geometry or reallocates cell IDs.
use crate::{
    controller::{DesktopBridge, input_id},
    dto::*,
    template_rows::TableRow,
    template_table_edit::TableEditRow,
};
use ecad_app::*;
use serde::Serialize;

pub(crate) struct TableGesture {
    revision: Revision,
    action: ActionId,
    target: Vec<String>,
    token: String,
    parameters: Option<ActionParameters>,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct TablePreviewRow {
    pub token: String,
    pub tables: Vec<TableRow>,
    pub snapped: Vec<String>,
    pub graphics: Vec<crate::template_graphic_rows::GraphicRow>,
}
impl DesktopBridge {
    pub(crate) fn template_table_request(
        &mut self,
        request: Request,
    ) -> Result<Response, BridgeError> {
        match request {
            Request::TemplateTablePreview {
                context,
                action,
                values,
                input,
            } => {
                let context = self.context(context)?;
                if self.templates.pending {
                    return Err(BridgeError::new(
                        "template",
                        "Apply or revert the property fields first",
                    ));
                }
                let doc = self
                    .templates
                    .editor
                    .as_ref()
                    .ok_or_else(|| BridgeError::new("template", "Template editor is closed"))?;
                // Every newer attempt invalidates the old successful proposal, including malformed input.
                if let Some(g) = &mut self.templates.gesture {
                    g.parameters = None;
                }
                let action = ActionId::new(action).map_err(|e| BridgeError::new("action", e))?;
                let operation = match self.registry.get(&action).map(|a| a.handler) {
                    Some(ActionHandler::Template(
                        op @ (TemplateOperation::Table(_) | TemplateOperation::Draw(_)),
                    )) => op,
                    _ => {
                        return Err(BridgeError::new(
                            "template",
                            "Choose a registered template geometry action",
                        ));
                    }
                };
                let state = self.registry.template_state(&action, &context, doc);
                if let Some(reason) = state.disabled {
                    return Err(BridgeError::new(
                        "template",
                        format!("Template action unavailable: {reason:?}"),
                    ));
                }
                let (target, proposal) = match operation {
                    TemplateOperation::Table(op) => {
                        let row: TableEditRow = serde_json::from_value(values)
                            .map_err(|e| BridgeError::new("template", e))?;
                        row.validate_selection(&context)?;
                        (
                            row.target(),
                            row.proposal_with_snap(doc, op).map(|(p, s)| (Some(p), s)),
                        )
                    }
                    TemplateOperation::Draw(kind) => {
                        let row: crate::template_artwork_edit::ArtworkEditRow =
                            serde_json::from_value(values)
                                .map_err(|e| BridgeError::new("template", e))?;
                        (row.target(), row.proposal(doc, kind).map(|p| (p, vec![])))
                    }
                    _ => unreachable!("validated geometry operation"),
                };
                let revision = doc.revision();
                if let Some(g) = &self.templates.gesture
                    && (g.revision != revision || g.action != action || g.target != target)
                {
                    return Err(BridgeError::new(
                        "template",
                        "The template gesture target changed; cancel before retargeting",
                    ));
                }
                self.templates.gesture = Some(TableGesture {
                    revision,
                    action,
                    target,
                    token: input.clone(),
                    parameters: None,
                });
                let (parameters, snapped) = proposal?;
                let layout = match &parameters {
                    Some(ActionParameters::TemplateEdit { layout, .. }) => Some(layout),
                    None => None,
                    _ => unreachable!("geometry proposal"),
                };
                let preview = TablePreviewRow {
                    token: if parameters.is_some() {
                        input
                    } else {
                        String::new()
                    },
                    tables: layout
                        .map(|l| l.tables().iter().map(TableRow::new).collect())
                        .unwrap_or_default(),
                    graphics: layout
                        .map(|l| l.graphics().iter().map(Into::into).collect())
                        .unwrap_or_default(),
                    snapped,
                };
                self.templates
                    .gesture
                    .as_mut()
                    .expect("started gesture")
                    .parameters = parameters;
                Ok(Response {
                    table_preview: Some(preview),
                    ..Response::default()
                })
            }
            Request::TemplateTableCommit {
                context,
                preview,
                input,
            } => {
                let context = self.context(context)?;
                let g =
                    self.templates.gesture.as_ref().ok_or_else(|| {
                        BridgeError::new("template", "No template gesture is active")
                    })?;
                if g.token != preview || context.editor != Some(g.revision) {
                    return Err(BridgeError::new(
                        "stale",
                        "Template preview was superseded or belongs to another editor",
                    ));
                }
                let parameters = g.parameters.clone().ok_or_else(|| {
                    BridgeError::new("template", "The latest template proposal is invalid")
                })?;
                let invocation = ActionInvocation {
                    input_id: input_id(&input)?,
                    action: g.action.clone(),
                    source: InvocationSource::Toolbar,
                    repeat: false,
                    context,
                    parameters,
                };
                let retained = self.templates.gesture.take();
                match self.invoke(invocation) {
                    Ok(reply) => Ok(reply),
                    Err(e) => {
                        self.templates.gesture = retained;
                        Err(e)
                    }
                }
            }
            _ => Err(BridgeError::new(
                "template",
                "Unknown template gesture request",
            )),
        }
    }
}
