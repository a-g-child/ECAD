//! Sheet editor DTOs and retained exact proposals; paper scene coordinates are presentation-only.
use crate::{
    controller::{DesktopBridge, input_id},
    dto::*,
};
use ecad_app::*;
use ecad_domain::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SheetRow {
    pub id: String,
    #[serde(default = "schematic_kind")]
    pub kind: String,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub document: Option<String>,
    pub number: String,
    pub title: String,
    pub width: String,
    pub height: String,
    pub left: String,
    pub right: String,
    pub top: String,
    pub bottom: String,
    pub view: Option<String>,
    pub paper_x: String,
    pub paper_y: String,
    pub source_x: String,
    pub source_y: String,
    pub numerator: String,
    pub denominator: String,
}
impl From<&DrawingSheet> for SheetRow {
    fn from(s: &DrawingSheet) -> Self {
        let p = s.placement.as_ref();
        Self {
            id: s.id.to_string(),
            kind: s.kind.key().into(),
            template: s.template.map(|t| t.key().into()),
            document: None,
            number: s.number.clone(),
            title: s.title.clone(),
            width: s.paper.width.to_string(),
            height: s.paper.height.to_string(),
            left: s.paper.left.to_string(),
            right: s.paper.right.to_string(),
            top: s.paper.top.to_string(),
            bottom: s.paper.bottom.to_string(),
            view: p.map(|p| p.view.to_string()),
            paper_x: p.map_or(20_000, |p| p.paper_origin.x).to_string(),
            paper_y: p.map_or(60_000, |p| p.paper_origin.y).to_string(),
            source_x: p.map_or(0, |p| p.view_origin.x).to_string(),
            source_y: p.map_or(0, |p| p.view_origin.y).to_string(),
            numerator: p.map_or(1, |p| p.numerator).to_string(),
            denominator: p.map_or(1, |p| p.denominator).to_string(),
        }
    }
}
fn schematic_kind() -> String {
    "schematic".into()
}
impl SheetRow {
    pub(crate) fn project(project: &Project, sheet: &DrawingSheet) -> Self {
        let mut row = Self::from(sheet);
        row.document = project.sheet_document(sheet.id).map(|id| id.to_string());
        row
    }
    fn sheet(self) -> Result<DrawingSheet, BridgeError> {
        fn parse<T: std::str::FromStr>(s: &str) -> Result<T, BridgeError> {
            s.parse().map_err(|_|BridgeError::new("sheet","Use integer micrometres and positive integer scale terms within the supported range"))
        }
        let placement = self
            .view
            .map(|id| {
                Ok::<_, BridgeError>(SheetViewPlacement {
                    view: id.parse().map_err(|e| BridgeError::new("sheet", e))?,
                    paper_origin: DrawingPoint {
                        x: parse(&self.paper_x)?,
                        y: parse(&self.paper_y)?,
                    },
                    view_origin: DrawingPoint {
                        x: parse(&self.source_x)?,
                        y: parse(&self.source_y)?,
                    },
                    numerator: parse(&self.numerator)?,
                    denominator: parse(&self.denominator)?,
                })
            })
            .transpose()?;
        Ok(DrawingSheet {
            kind: SheetKind::from_key(&self.kind)
                .ok_or_else(|| BridgeError::new("sheet", "Unknown sheet type"))?,
            template: self
                .template
                .map(|s| {
                    SheetTemplate::from_key(&s)
                        .ok_or_else(|| BridgeError::new("sheet", "Unknown sheet template"))
                })
                .transpose()?,
            id: self.id.parse().map_err(|e| BridgeError::new("sheet", e))?,
            number: self.number,
            title: self.title,
            paper: SheetPaper {
                width: parse(&self.width)?,
                height: parse(&self.height)?,
                left: parse(&self.left)?,
                right: parse(&self.right)?,
                top: parse(&self.top)?,
                bottom: parse(&self.bottom)?,
            },
            placement,
        })
    }
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct SheetPreviewRow {
    pub token: String,
    pub sheet: SheetRow,
    pub scene: Vec<SceneRow>,
    pub frame: crate::frame_rows::FrameRow,
    pub list: Option<crate::drawing_list_rows::DrawingListRow>,
}
#[derive(Default)]
pub(crate) struct SheetTool {
    pub preview: Option<(String, SheetPreview)>,
    pub new_view: Option<(SheetId, DrawingView)>,
}
impl DesktopBridge {
    pub(super) fn sheet_request(&mut self, request: Request) -> Result<Response, BridgeError> {
        match request {
            Request::NewSheet { context, kind } => self.new_sheet_draft(context, kind),
            Request::PreviewSheet {
                context,
                sheet,
                create,
            } => {
                self.sheet_tool.preview = None;
                self.context(context)?;
                let row: SheetRow =
                    serde_json::from_value(sheet).map_err(|e| BridgeError::new("sheet", e))?;
                let owner = row
                    .document
                    .as_deref()
                    .map(str::parse::<DrawingDocumentId>)
                    .transpose()
                    .map_err(|e| BridgeError::new("sheet", e))?;
                let sheet = row.sheet()?;
                let new_view = self
                    .sheet_tool
                    .new_view
                    .as_ref()
                    .filter(|(id, view)| {
                        *id == sheet.id && sheet.placement.as_ref().map(|p| p.view) == Some(view.id)
                    })
                    .map(|(_, view)| DrawingView {
                        name: sheet.title.clone(),
                        ..view.clone()
                    });
                let preview = if let Some(owner) = owner {
                    preview_sheet_in_document(
                        self.document.session(),
                        sheet,
                        create,
                        owner,
                        new_view,
                    )
                } else {
                    preview_sheet(self.document.session(), sheet, create)
                }
                .map_err(|e| BridgeError::new("sheet", e))?;
                let token = ecad_adapters::RandomIds::sheet()
                    .map_err(|e| BridgeError::new("sheet", e))?
                    .to_string();
                let view = preview
                    .sheet()
                    .placement
                    .as_ref()
                    .map(|p| p.view.to_string());
                let scene = self
                    .snapshot()
                    .snapshot
                    .expect("snapshot")
                    .scene
                    .into_iter()
                    .filter(|row| Some(&row.view) == view.as_ref())
                    .collect();
                let mut sheet_row = SheetRow::from(preview.sheet());
                sheet_row.document = Some(preview.document().to_string());
                let row = SheetPreviewRow {
                    token: token.clone(),
                    sheet: sheet_row,
                    list: preview.list().map(Into::into),
                    frame: crate::frame_rows::FrameRow::new(preview.sheet().id, preview.frame()),
                    scene,
                };
                self.sheet_tool.preview = Some((token, preview));
                Ok(Response {
                    sheet_preview: Some(row),
                    ..Response::default()
                })
            }
            Request::CommitSheet { token, input } => {
                if self.sheet_tool.preview.as_ref().map(|p| p.0.as_str()) != Some(&token) {
                    return Err(BridgeError::new(
                        "sheet",
                        "Sheet preview was replaced or invalidated",
                    ));
                }
                let preview = self.sheet_tool.preview.take().expect("checked preview").1;
                let mut context = ActionContext::for_session(self.document.session());
                context.revision = Some(preview.revision());
                let activate = preview
                    .is_creation()
                    .then(|| preview.sheet().id.to_string());
                let mut response =
                    self.invoke_sheet(context, ActionParameters::SheetPreview(preview), &input)?;
                response.activate_sheet = activate;
                self.sheet_tool.new_view = None;
                Ok(response)
            }
            Request::ChangeSheet {
                context,
                id,
                operation,
                index,
                input,
            } => {
                self.sheet_tool.preview = None;
                let context = self.context(context)?;
                let id = id.parse().map_err(|e| BridgeError::new("sheet", e))?;
                let command = match operation.as_str() {
                    "remove" => SheetCommand::Remove(id),
                    "move" => SheetCommand::Move { id, index },
                    _ => return Err(BridgeError::new("sheet", "Unknown sheet operation")),
                };
                self.invoke_sheet(context, ActionParameters::Sheet(command), &input)
            }
            Request::CancelSheet {} => {
                self.sheet_tool.preview = None;
                self.sheet_tool.new_view = None;
                Ok(Response::default())
            }
            _ => unreachable!("sheet routing"),
        }
    }
    fn invoke_sheet(
        &mut self,
        context: ActionContext,
        parameters: ActionParameters,
        input: &str,
    ) -> Result<Response, BridgeError> {
        let revision = self.document.session().revision();
        let invocation = ActionInvocation {
            input_id: input_id(input)?,
            action: ActionId::new("drawing.sheets.edit").expect("builtin"),
            source: InvocationSource::Toolbar,
            repeat: false,
            context,
            parameters,
        };
        let mut response = self.invoke(invocation)?;
        response.message = Some(
            if revision == self.document.session().revision() {
                "Sheet layout already matches · no document changes"
            } else {
                "Sheet layout updated · one undo operation"
            }
            .into(),
        );
        Ok(response)
    }
}
