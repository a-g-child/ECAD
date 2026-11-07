//! Validated annotation proposals retain exact text/position and a generated persistent identity.
use crate::{dto::*, projection};
use ecad_app::*;
use ecad_domain::*;
pub(crate) struct Annotation {
    context: ActionContext,
    id: LabelId,
    session: String,
    serial: u64,
    label: Option<DrawingLabel>,
    row: AnnotationRow,
}
impl Annotation {
    pub fn begin(
        session: &Session,
        context: ActionContext,
        serial: u64,
    ) -> Result<Self, BridgeError> {
        let view = context.view.ok_or_else(|| {
            BridgeError::new("annotation", "Choose a drawing view before placing text")
        })?;
        session
            .project()
            .require_object(ObjectRef::View(view))
            .map_err(|e| BridgeError::new("view", e))?;
        let id = ecad_adapters::RandomIds::label().map_err(|e| BridgeError::new("identity", e))?;
        let identity = format!(
            "annotation:{}:{}:{serial}",
            session.revision().epoch().get(),
            session.revision().get()
        );
        Ok(Self {
            context,
            id,
            session: identity.clone(),
            serial: 0,
            label: None,
            row: AnnotationRow {
                placement: MoveRow {
                    session: identity,
                    token: String::new(),
                    x: "0".into(),
                    y: "0".into(),
                    scene: vec![],
                    candidates: 0,
                    chosen: None,
                    snap: None,
                },
                text: String::new(),
                valid: false,
            },
        })
    }
    pub fn update(
        &mut self,
        session: &Session,
        identity: &str,
        value: serde_json::Value,
    ) -> Result<Response, BridgeError> {
        if identity != self.session {
            return Err(BridgeError::new(
                "annotation",
                "Annotation placement was replaced",
            ));
        }
        self.label = None;
        self.row.valid = false;
        self.row.placement.scene.clear();
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or_else(|| BridgeError::new("annotation", "Proposal identity exhausted"))?;
        if self.context.revision != Some(session.revision()) {
            return Err(BridgeError::new(
                "stale",
                "Document changed during annotation placement",
            ));
        }
        let input: AnnotationInput =
            serde_json::from_value(value).map_err(|e| BridgeError::new("placement", e))?;
        if input.text.trim().is_empty() {
            return Err(BridgeError::new("text", "Enter nonblank annotation text"));
        }
        let cursor = crate::placement::resolve(
            session,
            self.context.view.expect("checked at begin"),
            crate::placement::CursorInput {
                x: input.x,
                y: input.y,
                exact: input.exact,
                suppressed: input.suppressed,
                candidate: input.candidate,
                tolerance_um: input.tolerance_um,
            },
        )?;
        let position = cursor.position;
        let view = self.context.view.expect("checked at begin");
        let project = session.project();
        let label = DrawingLabel {
            id: self.id,
            view,
            text: input.text,
            position,
            appearance: project.drawing().settings().text_creation.clone(),
        };
        // The same base revision guards preset/style resolution at the normal T commit.
        let mut candidate = project.clone();
        candidate
            .insert_drawing_label(label.clone())
            .map_err(|e| BridgeError::new("annotation", e))?;
        self.row = AnnotationRow {
            placement: MoveRow {
                session: self.session.clone(),
                token: self.token(),
                x: position.x.to_string(),
                y: position.y.to_string(),
                scene: vec![projection::label(&candidate, &label)],
                candidates: cursor.candidates,
                chosen: cursor.chosen,
                snap: cursor.snap,
            },
            text: label.text.clone(),
            valid: true,
        };
        self.label = Some(label);
        Ok(self.response())
    }
    fn token(&self) -> String {
        format!("{}:{}", self.session, self.serial)
    }
    pub fn take(&mut self, token: &str) -> Result<(ActionContext, ActionParameters), BridgeError> {
        if token != self.token() {
            return Err(BridgeError::new(
                "annotation",
                "Annotation proposal was superseded",
            ));
        }
        let label = self.label.take().ok_or_else(|| {
            BridgeError::new("annotation", "No valid annotation proposal to apply")
        })?;
        Ok((
            self.context.clone(),
            ActionParameters::Label {
                id: label.id,
                text: label.text,
                position: label.position,
            },
        ))
    }
    pub fn response(&self) -> Response {
        Response {
            annotation: Some(self.row.clone()),
            ..Response::default()
        }
    }
}
