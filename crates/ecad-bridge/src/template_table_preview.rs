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
                let Some(ActionHandler::Template(TemplateOperation::Table(op))) =
                    self.registry.get(&action).map(|a| a.handler)
                else {
                    return Err(BridgeError::new(
                        "template",
                        "Choose a registered table action",
                    ));
                };
                let state = self.registry.template_state(&action, &context, doc);
                if let Some(reason) = state.disabled {
                    return Err(BridgeError::new(
                        "template",
                        format!("Table action unavailable: {reason:?}"),
                    ));
                }
                let row: TableEditRow =
                    serde_json::from_value(values).map_err(|e| BridgeError::new("template", e))?;
                let target = row.target();
                let revision = doc.revision();
                if let Some(g) = &self.templates.gesture
                    && (g.revision != revision || g.action != action || g.target != target)
                {
                    return Err(BridgeError::new(
                        "template",
                        "The table gesture target changed; cancel before retargeting",
                    ));
                }
                self.templates.gesture = Some(TableGesture {
                    revision,
                    action,
                    target,
                    token: input.clone(),
                    parameters: None,
                });
                let parameters = row.proposal(doc, op)?;
                let ActionParameters::TemplateEdit { layout, .. } = &parameters else {
                    unreachable!("table proposal")
                };
                let preview = TablePreviewRow {
                    token: input,
                    tables: layout.tables().iter().map(TableRow::new).collect(),
                };
                self.templates
                    .gesture
                    .as_mut()
                    .expect("started gesture")
                    .parameters = Some(parameters);
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
                        BridgeError::new("template", "No table gesture is active")
                    })?;
                if g.token != preview || context.editor != Some(g.revision) {
                    return Err(BridgeError::new(
                        "stale",
                        "Table preview was superseded or belongs to another editor",
                    ));
                }
                let parameters = g.parameters.clone().ok_or_else(|| {
                    BridgeError::new("template", "The latest table proposal is invalid")
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
                "Unknown table gesture request",
            )),
        }
    }
}
