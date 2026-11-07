//! Host-retained closed proposals. Updating invalidates old tokens before validation.
use crate::dto::*;
use ecad_app::*;
use ecad_domain::*;

pub(crate) struct Movement {
    context: ActionContext,
    reference: ObjectRef,
    session: String,
    serial: u64,
    preview: Option<MovePreview>,
    scene: std::collections::BTreeMap<ObjectRef, SceneRow>,
}
impl Movement {
    pub fn begin(
        session: &Session,
        context: ActionContext,
        reference: ObjectRef,
        serial: u64,
        scene: Vec<SceneRow>,
    ) -> Result<Self, BridgeError> {
        let drawing = session.project().drawing();
        let position = match reference {
            ObjectRef::Anchor(id) => drawing.terminal_anchor(id).map(|a| a.position),
            ObjectRef::Label(id) => drawing.label(id).map(|a| a.position),
            ObjectRef::Polyline(id) => drawing.polyline(id).and_then(|a| a.points.first().copied()),
            _ => None,
        }.ok_or_else(|| BridgeError::new("move", "Select labels, graphical paths or terminal anchors; move routes through their anchors"))?;
        let preview = preview_move(
            session,
            context.selection.iter().copied(),
            reference,
            MovePlacement::Exact(position),
        )
        .map_err(|e| BridgeError::new("move", e))?;
        if context.view != Some(preview.layout().view()) {
            return Err(BridgeError::new(
                "move",
                "Selection must belong to the active drawing view",
            ));
        }
        let scene = crate::preview_scene::capture(session, &context, scene)?;
        Ok(Self {
            context,
            reference,
            session: format!(
                "{}:{}:{serial}",
                session.revision().epoch().get(),
                session.revision().get()
            ),
            serial: 0,
            preview: Some(preview),
            scene,
        })
    }
    #[allow(clippy::too_many_arguments)] // Strict transport fields remain separate from core placement types.
    pub fn update(
        &mut self,
        session: &Session,
        identity: &str,
        x: &str,
        y: &str,
        exact: bool,
        suppressed: bool,
        candidate: Option<usize>,
        tolerance_um: u32,
    ) -> Result<Response, BridgeError> {
        if identity != self.session {
            return Err(BridgeError::new("move", "Movement was replaced"));
        }
        self.preview = None;
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or_else(|| BridgeError::new("move", "Preview identity exhausted"))?;
        if self.context.revision != Some(session.revision()) {
            return Err(BridgeError::new(
                "stale",
                "Document changed during movement",
            ));
        }
        let parse = |text: &str| {
            text.parse::<i64>().map_err(|_| {
                BridgeError::new(
                    "coordinate",
                    "Enter an integer micrometre coordinate within the signed 64-bit range",
                )
            })
        };
        let position = DrawingPoint {
            x: parse(x)?,
            y: parse(y)?,
        };
        let placement = if exact {
            MovePlacement::Exact(position)
        } else {
            MovePlacement::Pointer {
                position,
                settings: SnapSettings {
                    tolerance_um,
                    terminals: true,
                    endpoints: true,
                    midpoints: true,
                    grid: Some(DrawingGrid {
                        origin: DrawingPoint { x: 0, y: 0 },
                        spacing: DrawingLength::new(5000).expect("constant"),
                    }),
                },
                choice: if suppressed {
                    SnapChoice::Suppressed
                } else {
                    candidate.map_or(SnapChoice::Best, SnapChoice::Candidate)
                },
            }
        };
        self.preview = Some(
            preview_move(
                session,
                self.context.selection.iter().copied(),
                self.reference,
                placement,
            )
            .map_err(|e| BridgeError::new("move", e))?,
        );
        self.response()
    }
    fn token(&self) -> String {
        format!("{}:{}", self.session, self.serial)
    }
    pub fn take(&mut self, token: &str) -> Result<(ActionContext, MovePreview), BridgeError> {
        if token != self.token() {
            return Err(BridgeError::new(
                "move",
                "Preview was superseded; review the current placement",
            ));
        }
        let preview = self
            .preview
            .take()
            .ok_or_else(|| BridgeError::new("move", "No valid movement preview to apply"))?;
        Ok((self.context.clone(), preview))
    }
    pub fn response(&self) -> Result<Response, BridgeError> {
        let preview = self
            .preview
            .as_ref()
            .ok_or_else(|| BridgeError::new("move", "No valid preview"))?;
        let scene = crate::preview_scene::project(preview.layout(), &self.scene)?;
        let chosen = preview
            .chosen()
            .and_then(|c| preview.candidates().iter().position(|v| v == c));
        let snap = preview.chosen().map(|c| {
            match c.target {
                SnapTarget::Terminal(_) => "Terminal",
                SnapTarget::RouteEndpoint { .. } | SnapTarget::PolylineEndpoint { .. } => {
                    "Endpoint"
                }
                SnapTarget::RouteMidpoint { .. } | SnapTarget::PolylineMidpoint { .. } => {
                    "Midpoint"
                }
                SnapTarget::Grid => "Grid",
            }
            .to_string()
        });
        Ok(Response {
            movement: Some(MoveRow {
                session: self.session.clone(),
                token: self.token(),
                x: preview.destination().x.to_string(),
                y: preview.destination().y.to_string(),
                scene,
                candidates: preview.candidates().len(),
                chosen,
                snap,
            }),
            ..Response::default()
        })
    }
}
