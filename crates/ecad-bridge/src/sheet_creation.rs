//! New typed pages are drafts; a fresh schematic view is inserted only with the reviewed sheet.
use crate::{
    controller::DesktopBridge,
    dto::{BridgeError, Response},
    sheets::SheetRow,
};
use ecad_domain::*;
impl DesktopBridge {
    pub(crate) fn new_sheet_draft(
        &mut self,
        context: serde_json::Value,
        kind: Option<String>,
    ) -> Result<Response, BridgeError> {
        self.sheet_tool.preview = None;
        self.sheet_tool.new_view = None;
        let context = self.context(context)?;
        let typed = kind.is_some();
        let kind = kind
            .map(|key| {
                SheetKind::from_key(&key)
                    .ok_or_else(|| BridgeError::new("sheet", "Unknown sheet type"))
            })
            .transpose()?
            .unwrap_or(SheetKind::Schematic);
        let id = ecad_adapters::RandomIds::sheet().map_err(|e| BridgeError::new("sheet", e))?;
        let project = self.document.session().project();
        if project.sheet(id).is_some() {
            return Err(BridgeError::new(
                "sheet",
                "Generated sheet identity already exists; retry",
            ));
        }
        let owner = project
            .drawing_documents()
            .first()
            .expect("initial document")
            .id;
        let number = (1u64..)
            .map(|n| n.to_string())
            .find(|n| !project.sheets().iter().any(|s| &s.number == n))
            .expect("finite sheets");
        let title = match kind {
            SheetKind::Schematic => "New schematic",
            SheetKind::Cover => "Cover sheet",
            SheetKind::DrawingList => "Drawing list",
        };
        let template = Some(project.template_defaults().for_kind(kind));
        let view = if typed && kind == SheetKind::Schematic {
            let view =
                ecad_adapters::RandomIds::view().map_err(|e| BridgeError::new("sheet", e))?;
            if project.drawing().view(view).is_some() {
                return Err(BridgeError::new(
                    "sheet",
                    "Generated view identity already exists; retry",
                ));
            }
            self.sheet_tool.new_view = Some((
                id,
                DrawingView {
                    id: view,
                    name: title.into(),
                    overrides: StylePatch::default(),
                },
            ));
            Some(view)
        } else if kind == SheetKind::Schematic {
            context.view
        } else {
            None
        };
        let paper = match template {
            Some(SheetTemplate::Saved(pin)) => project
                .saved_template(pin)
                .map_err(|e| BridgeError::new("template", e))?
                .layout()
                .paper()
                .clone(),
            _ => {
                return Err(BridgeError::new(
                    "template",
                    "Project defaults require a saved template",
                ));
            }
        };
        let sheet = DrawingSheet {
            inherited_variant: None,
            id,
            kind,
            template,
            number,
            title: title.into(),
            paper,
            placement: view.map(|view| SheetViewPlacement {
                view,
                paper_origin: DrawingPoint {
                    x: 20_000,
                    y: 60_000,
                },
                view_origin: DrawingPoint::default(),
                numerator: 1,
                denominator: 1,
            }),
        };
        let mut row = SheetRow::from(&sheet);
        row.document = Some(owner.to_string());
        Ok(Response {
            sheet_draft: Some(row),
            ..Response::default()
        })
    }
}
