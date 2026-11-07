//! Transient-tool transport orchestration, kept inside the document controller's privacy boundary.
use super::*;
impl DesktopBridge {
    pub(super) fn tool(&mut self, request: Request) -> Result<Response, BridgeError> {
        match request {
            Request::BeginMove { context, reference } => {
                let context = self.context(context)?;
                let reference: Target = serde_json::from_value(reference)
                    .map_err(|e| BridgeError::new("reference", e))?;
                self.move_serial = self
                    .move_serial
                    .checked_add(1)
                    .ok_or_else(|| BridgeError::new("move", "Movement identity exhausted"))?;
                self.movement = None;
                self.transform = None;
                self.annotation = None;
                self.path_draft = None;
                let scene = self.snapshot().snapshot.expect("snapshot").scene;
                let movement = Movement::begin(
                    self.document.session(),
                    context,
                    reference.object()?,
                    self.move_serial,
                    scene,
                )?;
                let reply = movement.response()?;
                self.movement = Some(movement);
                Ok(reply)
            }
            Request::PreviewMove {
                session,
                x,
                y,
                exact,
                suppressed,
                candidate,
                tolerance_um,
            } => {
                let movement = self
                    .movement
                    .as_mut()
                    .ok_or_else(|| BridgeError::new("move", "No active movement"))?;
                movement.update(
                    self.document.session(),
                    &session,
                    &x,
                    &y,
                    exact,
                    suppressed,
                    candidate,
                    tolerance_um,
                )
            }
            Request::CommitMove { token, input } => {
                let movement = self
                    .movement
                    .as_mut()
                    .ok_or_else(|| BridgeError::new("move", "No active movement"))?;
                let (context, preview) = movement.take(&token)?;
                let result = self.invoke(ActionInvocation {
                    input_id: input_id(&input)?,
                    action: ActionId::new("drawing.selection.move").expect("builtin"),
                    source: InvocationSource::Toolbar,
                    repeat: false,
                    context,
                    parameters: ActionParameters::CommitMove(preview),
                });
                self.movement = None;
                result
            }
            Request::BeginTransform {
                context,
                reference,
                kind,
            } => {
                let context = self.context(context)?;
                let reference: Target = serde_json::from_value(reference)
                    .map_err(|e| BridgeError::new("reference", e))?;
                self.move_serial = self
                    .move_serial
                    .checked_add(1)
                    .ok_or_else(|| BridgeError::new("transform", "Tool identity exhausted"))?;
                self.movement = None;
                self.transform = None;
                self.annotation = None;
                self.path_draft = None;
                let scene = self.snapshot().snapshot.expect("snapshot").scene;
                let transform = Transform::begin(
                    self.document.session(),
                    context,
                    reference.object()?,
                    &kind,
                    self.move_serial,
                    scene,
                )?;
                let response = transform.response()?;
                self.transform = Some(transform);
                Ok(response)
            }
            Request::PreviewTransform {
                session,
                x,
                y,
                operation,
            } => self
                .transform
                .as_mut()
                .ok_or_else(|| BridgeError::new("transform", "No active transform"))?
                .update(self.document.session(), &session, &x, &y, &operation),
            Request::CommitTransform { token, input } => {
                let transform = self
                    .transform
                    .as_mut()
                    .ok_or_else(|| BridgeError::new("transform", "No active transform"))?;
                let (context, action, preview) = transform.take(&token)?;
                let result = self.invoke(ActionInvocation {
                    input_id: input_id(&input)?,
                    action,
                    source: InvocationSource::Toolbar,
                    repeat: false,
                    context,
                    parameters: ActionParameters::CommitLayout(preview),
                });
                self.transform = None;
                result
            }
            Request::BeginAnnotation { context } => {
                let context = self.context(context)?;
                self.move_serial = self
                    .move_serial
                    .checked_add(1)
                    .ok_or_else(|| BridgeError::new("annotation", "Tool identity exhausted"))?;
                self.movement = None;
                self.transform = None;
                self.annotation = None;
                self.path_draft = None;
                let annotation =
                    Annotation::begin(self.document.session(), context, self.move_serial)?;
                let reply = annotation.response();
                self.annotation = Some(annotation);
                Ok(reply)
            }
            Request::PreviewAnnotation { session, placement } => {
                let tool = self
                    .annotation
                    .as_mut()
                    .ok_or_else(|| BridgeError::new("annotation", "No active annotation"))?;
                tool.update(self.document.session(), &session, placement)
            }
            Request::CommitAnnotation { token, input } => {
                let tool = self
                    .annotation
                    .as_mut()
                    .ok_or_else(|| BridgeError::new("annotation", "No active annotation"))?;
                let (context, parameters) = tool.take(&token)?;
                let result = self.invoke(ActionInvocation {
                    input_id: input_id(&input)?,
                    action: ActionId::new("drawing.label.create").expect("builtin"),
                    source: InvocationSource::Toolbar,
                    repeat: false,
                    context,
                    parameters,
                });
                self.annotation = None;
                result
            }
            Request::BeginPath { context } => {
                let context = self.context(context)?;
                self.move_serial = self
                    .move_serial
                    .checked_add(1)
                    .ok_or_else(|| BridgeError::new("path", "Tool identity exhausted"))?;
                self.movement = None;
                self.transform = None;
                self.annotation = None;
                self.path_draft = None;
                let draft = PathDraft::begin(self.document.session(), context, self.move_serial)?;
                let reply = draft.response();
                self.path_draft = Some(draft);
                Ok(reply)
            }
            Request::PreviewPath { session, placement } => self
                .path_draft
                .as_mut()
                .ok_or_else(|| BridgeError::new("path", "No active path"))?
                .update(self.document.session(), &session, placement),
            Request::AddPathVertex { token } => self
                .path_draft
                .as_mut()
                .ok_or_else(|| BridgeError::new("path", "No active path"))?
                .add(self.document.session(), &token),
            Request::BackPathVertex { token } => self
                .path_draft
                .as_mut()
                .ok_or_else(|| BridgeError::new("path", "No active path"))?
                .back(self.document.session(), &token),
            Request::CommitPath { token, input } => {
                let draft = self
                    .path_draft
                    .as_mut()
                    .ok_or_else(|| BridgeError::new("path", "No active path"))?;
                let (context, parameters) = draft.take(&token)?;
                let result = self.invoke(ActionInvocation {
                    input_id: input_id(&input)?,
                    action: ActionId::new("drawing.polyline.create").expect("builtin"),
                    source: InvocationSource::Toolbar,
                    repeat: false,
                    context,
                    parameters,
                });
                self.path_draft = None;
                result
            }
            _ => Err(BridgeError::new("tool", "Unsupported tool request")),
        }
    }
}
