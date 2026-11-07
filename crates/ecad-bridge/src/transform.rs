//! Retained orthogonal transform proposals with exact pivot strings and original action context.
use crate::{dto::*, preview_scene};
use ecad_app::*;
use ecad_domain::*;
use std::collections::BTreeMap;
pub(crate) struct Transform {
    context: ActionContext,
    action: ActionId,
    session: String,
    serial: u64,
    pivot: DrawingPoint,
    operation: String,
    preview: Option<LayoutPreview>,
    scene: BTreeMap<ObjectRef, SceneRow>,
}
impl Transform {
    pub fn begin(
        session: &Session,
        context: ActionContext,
        reference: ObjectRef,
        kind: &str,
        serial: u64,
        scene: Vec<SceneRow>,
    ) -> Result<Self, BridgeError> {
        let drawing = session.project().drawing();
        if !context.selection.contains(&reference) {
            return Err(BridgeError::new(
                "transform",
                "Pivot reference must be selected",
            ));
        }
        let pivot = match reference {
            ObjectRef::Anchor(id) => drawing.terminal_anchor(id).map(|a| a.position),
            ObjectRef::Label(id) => drawing.label(id).map(|a| a.position),
            ObjectRef::Polyline(id) => drawing.polyline(id).and_then(|a| a.points.first().copied()),
            _ => None,
        }
        .ok_or_else(|| {
            BridgeError::new(
                "transform",
                "Select labels, graphical paths or terminal anchors",
            )
        })?;
        let (action, operation) = match kind {
            "rotate" => ("drawing.selection.rotate", "rotate_90"),
            "mirror" => ("drawing.selection.mirror", "mirror_vertical"),
            _ => return Err(BridgeError::new("transform", "Unknown transform kind")),
        };
        let mut tool = Self {
            scene: preview_scene::capture(session, &context, scene)?,
            context,
            action: ActionId::new(action).expect("builtin"),
            session: format!(
                "transform:{}:{}:{serial}",
                session.revision().epoch().get(),
                session.revision().get()
            ),
            serial: 0,
            pivot,
            operation: operation.into(),
            preview: None,
        };
        tool.preview = Some(tool.prepare(session, pivot, operation)?);
        Ok(tool)
    }
    fn prepare(
        &self,
        session: &Session,
        pivot: DrawingPoint,
        operation: &str,
    ) -> Result<LayoutPreview, BridgeError> {
        if self.context.revision != Some(session.revision()) {
            return Err(BridgeError::new(
                "stale",
                "Document changed during transformation",
            ));
        }
        let transform = match (self.action.as_str(), operation) {
            ("drawing.selection.rotate", "rotate_90") => DrawingTransform::Rotate {
                pivot,
                turn: QuarterTurn::Positive90,
            },
            ("drawing.selection.rotate", "rotate_180") => DrawingTransform::Rotate {
                pivot,
                turn: QuarterTurn::Half,
            },
            ("drawing.selection.rotate", "rotate_270") => DrawingTransform::Rotate {
                pivot,
                turn: QuarterTurn::Negative90,
            },
            ("drawing.selection.mirror", "mirror_vertical") => DrawingTransform::Mirror {
                pivot,
                axis: MirrorAxis::Vertical,
            },
            ("drawing.selection.mirror", "mirror_horizontal") => DrawingTransform::Mirror {
                pivot,
                axis: MirrorAxis::Horizontal,
            },
            _ => {
                return Err(BridgeError::new(
                    "transform",
                    "Operation does not match the active transform tool",
                ));
            }
        };
        let preview = preview_layout(
            session,
            self.context.selection.iter().copied(),
            LayoutOperation::Transform(transform),
        )
        .map_err(|e| BridgeError::new("transform", e))?;
        if self.context.view != Some(preview.view()) {
            return Err(BridgeError::new(
                "transform",
                "Selection must belong to the active view",
            ));
        }
        Ok(preview)
    }
    pub fn update(
        &mut self,
        session: &Session,
        identity: &str,
        x: &str,
        y: &str,
        operation: &str,
    ) -> Result<Response, BridgeError> {
        if identity != self.session {
            return Err(BridgeError::new("transform", "Transform was replaced"));
        }
        self.preview = None;
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or_else(|| BridgeError::new("transform", "Preview identity exhausted"))?;
        let parse = |s: &str| {
            s.parse::<i64>().map_err(|_| {
                BridgeError::new(
                    "coordinate",
                    "Enter a signed 64-bit integer micrometre pivot",
                )
            })
        };
        let pivot = DrawingPoint {
            x: parse(x)?,
            y: parse(y)?,
        };
        let preview = self.prepare(session, pivot, operation)?;
        self.pivot = pivot;
        self.operation = operation.into();
        self.preview = Some(preview);
        self.response()
    }
    fn token(&self) -> String {
        format!("{}:{}", self.session, self.serial)
    }
    pub fn take(
        &mut self,
        token: &str,
    ) -> Result<(ActionContext, ActionId, LayoutPreview), BridgeError> {
        if token != self.token() {
            return Err(BridgeError::new("transform", "Preview was superseded"));
        }
        let preview = self
            .preview
            .take()
            .ok_or_else(|| BridgeError::new("transform", "No valid transform preview"))?;
        Ok((self.context.clone(), self.action.clone(), preview))
    }
    pub fn response(&self) -> Result<Response, BridgeError> {
        let preview = self
            .preview
            .as_ref()
            .ok_or_else(|| BridgeError::new("transform", "No valid transform preview"))?;
        Ok(Response {
            transform: Some(TransformRow {
                session: self.session.clone(),
                token: self.token(),
                x: self.pivot.x.to_string(),
                y: self.pivot.y.to_string(),
                operation: self.operation.clone(),
                scene: preview_scene::project(preview, &self.scene)?,
            }),
            ..Response::default()
        })
    }
}
