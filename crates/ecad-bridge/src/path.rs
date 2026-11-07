//! Host-owned accepted vertices and a distinct tentative leg; finishing never accepts that leg.
use crate::{
    dto::*,
    placement::{self, Cursor, CursorInput},
    projection,
};
use ecad_app::*;
use ecad_domain::*;
pub(crate) struct PathDraft {
    context: ActionContext,
    id: PolylineId,
    session: String,
    serial: u64,
    vertices: Vec<DrawingPoint>,
    cursor: Option<Cursor>,
    proposal: Option<DrawingPolyline>,
    row: PathRow,
}
impl PathDraft {
    pub fn begin(
        session: &Session,
        context: ActionContext,
        serial: u64,
    ) -> Result<Self, BridgeError> {
        let view = context
            .view
            .ok_or_else(|| BridgeError::new("path", "Choose a drawing view first"))?;
        session
            .project()
            .require_object(ObjectRef::View(view))
            .map_err(|e| BridgeError::new("view", e))?;
        let id =
            ecad_adapters::RandomIds::polyline().map_err(|e| BridgeError::new("identity", e))?;
        let identity = format!(
            "path:{}:{}:{serial}",
            session.revision().epoch().get(),
            session.revision().get()
        );
        Ok(Self {
            context,
            id,
            session: identity.clone(),
            serial: 0,
            vertices: vec![],
            cursor: None,
            proposal: None,
            row: PathRow {
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
                vertices: vec![],
                rubber: None,
                can_add: false,
                can_finish: false,
            },
        })
    }
    fn current(&self, session: &Session) -> Result<(), BridgeError> {
        if self.context.revision != Some(session.revision()) {
            return Err(BridgeError::new(
                "stale",
                "Document changed during path placement",
            ));
        }
        Ok(())
    }
    fn token(&self) -> String {
        format!("{}:{}", self.session, self.serial)
    }
    fn check_token(&self, token: &str) -> Result<(), BridgeError> {
        if token != self.token() {
            return Err(BridgeError::new("path", "Path proposal was superseded"));
        }
        Ok(())
    }
    fn invalidate(&mut self) -> Result<(), BridgeError> {
        self.proposal = None;
        self.row.can_add = false;
        self.row.can_finish = false;
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or_else(|| BridgeError::new("path", "Proposal identity exhausted"))?;
        Ok(())
    }
    pub fn update(
        &mut self,
        session: &Session,
        identity: &str,
        value: serde_json::Value,
    ) -> Result<Response, BridgeError> {
        if identity != self.session {
            return Err(BridgeError::new("path", "Path placement was replaced"));
        }
        self.invalidate()?;
        self.current(session)?;
        let input: CursorInput =
            serde_json::from_value(value).map_err(|e| BridgeError::new("placement", e))?;
        self.cursor = Some(placement::resolve(
            session,
            self.context.view.expect("checked"),
            input,
        )?);
        self.rebuild(session, self.vertices.clone())
    }
    pub fn add(&mut self, session: &Session, token: &str) -> Result<Response, BridgeError> {
        self.check_token(token)?;
        self.current(session)?;
        if !self.row.can_add {
            return Err(BridgeError::new(
                "path",
                "Choose a different point before adding a vertex",
            ));
        }
        let mut vertices = self.vertices.clone();
        vertices.push(self.cursor.as_ref().expect("valid cursor").position);
        self.invalidate()?;
        self.rebuild(session, vertices)
    }
    pub fn back(&mut self, session: &Session, token: &str) -> Result<Response, BridgeError> {
        self.check_token(token)?;
        self.current(session)?;
        let mut vertices = self.vertices.clone();
        if vertices.pop().is_none() {
            return Err(BridgeError::new("path", "No draft vertices to remove"));
        }
        self.invalidate()?;
        self.rebuild(session, vertices)
    }
    fn rebuild(
        &mut self,
        session: &Session,
        vertices: Vec<DrawingPoint>,
    ) -> Result<Response, BridgeError> {
        let cursor = self
            .cursor
            .as_ref()
            .expect("a cursor exists before vertices can be added");
        let position = cursor.position;
        let can_add = vertices.last() != Some(&position);
        let mut points = vertices.clone();
        if can_add {
            points.push(position);
        }
        let mut scene = vec![];
        let mut rubber = None;
        let mut proposal = None;
        if points.len() >= 2 {
            let mut path = DrawingPolyline {
                id: self.id,
                view: self.context.view.expect("checked"),
                points,
                appearance: session
                    .project()
                    .drawing()
                    .settings()
                    .stroke_creation
                    .clone(),
            };
            let mut candidate = session.project().clone();
            candidate
                .insert_drawing_polyline(path.clone())
                .map_err(|e| BridgeError::new("path", e))?;
            let mut row = projection::polyline(&candidate, &path);
            if can_add {
                let mut tail = row.clone();
                tail.points = tail.points[tail.points.len() - 2..].to_vec();
                rubber = Some(tail);
            }
            if vertices.len() >= 2 {
                path.points = vertices.clone();
                row.points = vertices
                    .iter()
                    .map(|p| [p.x as f64 / 1000.0, p.y as f64 / 1000.0])
                    .collect();
                scene.push(row);
                proposal = Some(path);
            }
        }
        self.row = PathRow {
            placement: MoveRow {
                session: self.session.clone(),
                token: self.token(),
                x: position.x.to_string(),
                y: position.y.to_string(),
                scene,
                candidates: cursor.candidates,
                chosen: cursor.chosen,
                snap: cursor.snap.clone(),
            },
            vertices: vertices
                .iter()
                .map(|p| [p.x.to_string(), p.y.to_string()])
                .collect(),
            rubber,
            can_add,
            can_finish: proposal.is_some(),
        };
        self.vertices = vertices;
        self.proposal = proposal;
        Ok(self.response())
    }
    pub fn take(&mut self, token: &str) -> Result<(ActionContext, ActionParameters), BridgeError> {
        self.check_token(token)?;
        let path = self.proposal.take().ok_or_else(|| {
            BridgeError::new("path", "Accept at least two points before finishing")
        })?;
        Ok((
            self.context.clone(),
            ActionParameters::Polyline {
                id: path.id,
                points: path.points,
            },
        ))
    }
    pub fn response(&self) -> Response {
        Response {
            path: Some(self.row.clone()),
            ..Response::default()
        }
    }
}
