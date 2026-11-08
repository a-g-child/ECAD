//! Captured project/template targets and protected focus, independent of request execution.
use crate::{controller::DesktopBridge, dto::*};
use ecad_app::*;
use std::collections::BTreeSet;
impl DesktopBridge {
    pub(crate) fn context(&self, value: serde_json::Value) -> Result<ActionContext, BridgeError> {
        let ctx: Context =
            serde_json::from_value(value).map_err(|e| BridgeError::new("context", e))?;
        if ctx.revision != Token::from(self.document.session().revision()) {
            return Err(BridgeError::new(
                "stale",
                "The document changed. Refresh before applying this operation",
            ));
        }
        if let Some(token) = ctx.editor {
            let editor = self
                .templates
                .editor
                .as_ref()
                .ok_or_else(|| BridgeError::new("stale", "Template editor is closed"))?;
            if token != Token::from(editor.revision()) {
                return Err(BridgeError::new(
                    "stale",
                    "The template editor changed; refresh before applying",
                ));
            }
            if !ctx.selection.is_empty() || ctx.view.is_some() || ctx.sheet.is_some() {
                return Err(BridgeError::new(
                    "template",
                    "Template context cannot address electrical drawing targets",
                ));
            }
            let focus = match ctx.focus.as_str() {
                "canvas" => ActionFocus::Canvas,
                "tree" => ActionFocus::ProjectTree,
                "inspector" => ActionFocus::Inspector,
                "text" => ActionFocus::TextInput,
                "modal" => ActionFocus::Modal,
                "workspace" => ActionFocus::Workspace,
                _ => return Err(BridgeError::new("focus", "Unknown input focus")),
            };
            return Ok(ActionContext {
                editor: Some(editor.revision()),
                tool_active: self.templates.gesture.is_some(),
                focus,
                ..ActionContext::default()
            });
        }
        if self.templates.editor.is_some() {
            return Err(BridgeError::new(
                "stale",
                "The project drawing is not the active editor",
            ));
        }
        let selection = ctx
            .selection
            .iter()
            .map(Target::object)
            .collect::<Result<BTreeSet<_>, _>>()?;
        for target in &selection {
            self.document
                .session()
                .project()
                .require_object(*target)
                .map_err(|e| BridgeError::new("target", e))?;
        }
        let view = ctx
            .view
            .map(|id| id.parse())
            .transpose()
            .map_err(|e| BridgeError::new("view", e))?;
        let sheet = ctx
            .sheet
            .map(|id| id.parse::<ecad_domain::SheetId>())
            .transpose()
            .map_err(|e| BridgeError::new("sheet", e))?;
        if let Some(id) = sheet
            && self
                .document
                .session()
                .project()
                .sheet(id)
                .is_none_or(|s| s.placement.as_ref().map(|p| p.view) != view)
        {
            return Err(BridgeError::new(
                "sheet",
                "The captured sheet is missing or has a different drawing view",
            ));
        }
        let focus = match ctx.focus.as_str() {
            "canvas" => ActionFocus::Canvas,
            "tree" => ActionFocus::ProjectTree,
            "text" => ActionFocus::TextInput,
            "inspector" => ActionFocus::Inspector,
            "modal" => ActionFocus::Modal,
            "workspace" => ActionFocus::Workspace,
            _ => return Err(BridgeError::new("focus", "Unknown input focus")),
        };
        Ok(ActionContext {
            revision: Some(self.document.session().revision()),
            selection,
            view,
            sheet,
            focus,
            tool_active: self.has_active_tools(),
            ..ActionContext::default()
        })
    }
}
