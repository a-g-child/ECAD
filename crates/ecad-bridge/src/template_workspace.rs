//! One independent template editor plus lazy durable library publication; never project undo for library writes.
use crate::{controller::DesktopBridge, dto::*, template_rows::*};
use ecad_adapters::*;
use ecad_app::*;
use ecad_domain::*;
use std::{collections::BTreeMap, path::PathBuf};
#[derive(Default)]
pub(crate) struct TemplateWorkspace {
    pub editor: Option<TemplateDocument>,
    pub selected: Option<TemplateVariantId>,
    pub gesture: Option<crate::template_table_preview::TableGesture>,
    pub pending: bool,
    pub pending_table: Option<TableOperation>,
    pub pending_selection: Option<TemplateSelection>,
    pub path: Option<PathBuf>,
    publisher: Option<TemplatePublisher<TemplateLibraryFile>>,
}
impl TemplateWorkspace {
    pub(crate) fn dirty(&self) -> bool {
        self.gesture.is_some()
            || self.pending
            || self.editor.as_ref().is_some_and(TemplateDocument::is_dirty)
    }
    fn load(&mut self) -> Result<(), BridgeError> {
        if self.publisher.is_some() {
            return Ok(());
        }
        let path = self.path.clone().ok_or_else(|| {
            BridgeError::new("templates", "Template library storage is not configured")
        })?;
        let store = TemplateLibraryFile::new(path).with_parent_creation();
        self.publisher = Some(match TemplatePublisher::open(store.clone()) {
            Ok(p) => p,
            Err(TemplateLibraryFileError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                TemplatePublisher::new(store)
            }
            Err(e) => return Err(BridgeError::new("templates", e)),
        });
        Ok(())
    }
    fn catalog(
        &self,
        project: &Project,
    ) -> Result<BTreeMap<TemplateRevisionRef, TemplateRevision>, BridgeError> {
        let mut bodies: BTreeMap<_, _> = project
            .template_revisions()
            .map(|r| (r.reference(), r.clone()))
            .collect();
        if let Some(p) = &self.publisher {
            for body in p.catalog().revisions() {
                if bodies.get(&body.reference()).is_some_and(|old| old != body) {
                    return Err(BridgeError::new(
                        "template",
                        "Project and library contain conflicting immutable template bodies",
                    ));
                }
                bodies.insert(body.reference(), body.clone());
            }
        }
        Ok(bodies)
    }
    pub(crate) fn view(&self, project: &Project) -> Option<TemplateView> {
        let doc = self.editor.as_ref()?;
        let (catalog, error) = match self.catalog(project) {
            Ok(bodies) => (
                bodies
                    .values()
                    .flat_map(|r| {
                        r.variants().map(|v| {
                            (
                                SheetTemplate::Saved(v.reference()).key(),
                                v.name().to_owned(),
                            )
                        })
                    })
                    .collect(),
                None,
            ),
            Err(e) => (vec![], Some(e.to_string())),
        };
        let mut view = TemplateView::new(
            doc,
            self.selected.expect("active variant"),
            self.pending || self.gesture.is_some(),
            catalog,
        );
        view.catalog_error = error;
        Some(view)
    }
    fn open(&mut self, project: &Project, pin: TemplateVariantRef) -> Result<(), BridgeError> {
        let catalog = self.catalog(project)?;
        let body = catalog
            .get(&pin.revision_ref())
            .filter(|b| b.variant(pin.variant).is_some())
            .ok_or_else(|| BridgeError::new("template", "Saved template is unavailable"))?;
        self.editor = Some(TemplateDocument::open(body));
        self.selected = Some(pin.variant);
        self.pending = false;
        Ok(())
    }
}
impl DesktopBridge {
    /// Configure template library location without IO or project edits.
    /// # Errors
    /// An open editor must be closed before changing its publication destination.
    pub fn configure_template_store(
        &mut self,
        path: impl Into<PathBuf>,
    ) -> Result<(), BridgeError> {
        if self.templates.editor.is_some() {
            return Err(BridgeError::new(
                "template",
                "Close the template editor before changing library storage",
            ));
        }
        self.templates.path = Some(path.into());
        self.templates.publisher = None;
        Ok(())
    }
    pub(crate) fn open_templates(&mut self) -> Result<Response, BridgeError> {
        if self.templates.editor.is_some() {
            return Ok(self.snapshot());
        }
        if self.has_active_tools() {
            return Err(BridgeError::new(
                "template",
                "Finish or cancel the drawing tool before opening templates",
            ));
        }
        self.templates.load()?;
        let catalog = self.templates.catalog(self.document.session().project())?;
        let pin = catalog
            .values()
            .flat_map(|r| r.variants())
            .next()
            .ok_or_else(|| BridgeError::new("template", "No saved template is available"))?
            .reference();
        self.templates
            .open(self.document.session().project(), pin)?;
        self.router.cancel_pending();
        Ok(self.snapshot())
    }
    pub(crate) fn template_invoke(
        &mut self,
        invocation: ActionInvocation,
    ) -> Result<Response, BridgeError> {
        let handler = self
            .registry
            .get(&invocation.action)
            .map(|a| a.handler)
            .ok_or_else(|| BridgeError::new("action", "Unknown template action"))?;
        if self.templates.gesture.is_some() && handler != ActionHandler::CancelPlacement {
            return Err(BridgeError::new(
                "template",
                "Finish or cancel the template gesture first",
            ));
        }
        let table_commit = matches!(handler, ActionHandler::Template(TemplateOperation::Table(op))
            if self.templates.pending_table == Some(op) && matches!(&invocation.parameters, ActionParameters::TemplateEdit { .. }));
        let pending_control = matches!(
            handler,
            ActionHandler::Template(TemplateOperation::Revert | TemplateOperation::Discard)
        ) || (self.templates.pending_table.is_none()
            && handler == ActionHandler::Template(TemplateOperation::Apply));
        if self.templates.pending && !table_commit && !pending_control {
            return Err(BridgeError::new(
                "template",
                "Apply or revert the property fields first",
            ));
        }
        let id = if handler == ActionHandler::Save {
            Some(RandomIds::template_revision().map_err(|e| BridgeError::new("template", e))?)
        } else {
            None
        };
        if let (Some(revision), Some(editor)) = (id, &self.templates.editor)
            && self
                .templates
                .catalog(self.document.session().project())?
                .keys()
                .any(|pin| pin.definition == editor.definition() && pin.revision == revision)
        {
            return Err(BridgeError::new(
                "template",
                "Generated template revision already exists; retry",
            ));
        }
        let outcome = self
            .dispatcher
            .invoke_template(
                &self.registry,
                invocation,
                self.templates
                    .editor
                    .as_mut()
                    .ok_or_else(|| BridgeError::new("template", "Template editor is closed"))?,
                self.templates.publisher.as_mut().expect("loaded publisher"),
                id,
            )
            .map_err(|e| BridgeError::new("template", e))?;
        let message = match outcome {
            TemplateActionOutcome::Request(TemplateOperation::Draw(kind)) => {
                let mut response = self.snapshot();
                response.request = Some(
                    match kind {
                        Some(TemplatePrimitive::Text) => "template_draw_text",
                        Some(TemplatePrimitive::Line) => "template_draw_line",
                        Some(TemplatePrimitive::Polyline) => "template_draw_polyline",
                        Some(TemplatePrimitive::Rectangle) => "template_draw_rectangle",
                        Some(TemplatePrimitive::Ellipse) => "template_draw_ellipse",
                        None => "template_artwork_move",
                    }
                    .into(),
                );
                return Ok(response);
            }
            TemplateActionOutcome::Request(TemplateOperation::Artwork) => {
                let mut response = self.snapshot();
                response.request = Some("template_artwork".into());
                return Ok(response);
            }
            TemplateActionOutcome::Request(TemplateOperation::CancelDrag) => {
                self.templates.gesture = None;
                "Template gesture cancelled"
            }
            TemplateActionOutcome::Changed => {
                self.templates.pending = false;
                self.templates.pending_table = None;
                self.templates.pending_selection = None;
                if handler == ActionHandler::Save {
                    "Template revision saved"
                } else {
                    "Template updated"
                }
            }
            TemplateActionOutcome::Open(pin) => {
                self.templates
                    .open(self.document.session().project(), pin)?;
                "Template opened"
            }
            TemplateActionOutcome::Request(TemplateOperation::Table(op)) => {
                let mut response = self.snapshot();
                response.request = Some(
                    match op {
                        TableOperation::InsertRow => "template_table_insert_row",
                        TableOperation::InsertColumn => "template_table_insert_column",
                        TableOperation::Resize => "template_table_resize",
                        TableOperation::Split => "template_cell_split",
                        TableOperation::Merge => "template_cells_merge",
                        TableOperation::Boundary => "template_table_boundary",
                    }
                    .into(),
                );
                return Ok(response);
            }
            TemplateActionOutcome::Request(TemplateOperation::Apply) => {
                let mut response = self.snapshot();
                response.request = Some("template_apply_properties".into());
                return Ok(response);
            }
            TemplateActionOutcome::Request(TemplateOperation::Reload) => {
                self.templates
                    .publisher
                    .as_mut()
                    .expect("loaded publisher")
                    .reload()
                    .map_err(|e| BridgeError::new("template", e))?;
                "Template library reloaded"
            }
            TemplateActionOutcome::Request(TemplateOperation::Open) => {
                let mut response = self.snapshot();
                response.request = Some("template_library_focus".into());
                return Ok(response);
            }
            TemplateActionOutcome::Request(TemplateOperation::New) => {
                let definition = RandomIds::template_definition()
                    .map_err(|e| BridgeError::new("template", e))?;
                let variant =
                    RandomIds::template_variant().map_err(|e| BridgeError::new("template", e))?;
                if self
                    .templates
                    .catalog(self.document.session().project())?
                    .keys()
                    .any(|p| p.definition == definition)
                {
                    return Err(BridgeError::new(
                        "template",
                        "Generated template identity already exists; retry",
                    ));
                }
                let layout = SheetTemplate::StandardV1
                    .layout(&SheetPaper::default())
                    .map_err(|e| BridgeError::new("template", e))?;
                self.templates.editor = Some(
                    TemplateDocument::new(
                        definition,
                        variant,
                        "New drawing template".into(),
                        layout,
                    )
                    .map_err(|e| BridgeError::new("template", e))?,
                );
                self.templates.selected = Some(variant);
                "New template draft"
            }
            TemplateActionOutcome::Request(
                TemplateOperation::Close | TemplateOperation::Discard,
            ) => {
                self.templates.editor = None;
                self.templates.selected = None;
                self.templates.pending = false;
                self.templates.pending_table = None;
                self.templates.pending_selection = None;
                self.router.cancel_pending();
                "Template editor closed"
            }
            TemplateActionOutcome::Request(TemplateOperation::Revert) => {
                self.templates.pending = false;
                self.templates.pending_table = None;
                self.templates.pending_selection = None;
                "Template fields reverted"
            }
            TemplateActionOutcome::Request(TemplateOperation::Import) => {
                let pin = self
                    .templates
                    .editor
                    .as_ref()
                    .expect("active editor")
                    .published()
                    .ok_or_else(|| BridgeError::new("template", "Save the template first"))?;
                let command = template_library_import(
                    self.templates
                        .publisher
                        .as_ref()
                        .expect("loaded library")
                        .catalog(),
                    pin,
                )
                .or_else(|_| {
                    self.document
                        .session()
                        .project()
                        .template_revision(pin)
                        .map(|r| Command::InsertTemplateRevision(std::sync::Arc::new(r.clone())))
                        .ok_or(TemplateLibraryError::MissingRevision(pin))
                })
                .map_err(|e| BridgeError::new("template", e))?;
                let changed = self
                    .document
                    .execute(self.document.session().revision(), command)
                    .map_err(|e| BridgeError::new("template", e))?;
                if changed {
                    "Saved template added to project; existing sheets retain their revision"
                } else {
                    "Saved template is already available in this project"
                }
            }
            TemplateActionOutcome::Fit => {
                let mut response = self.snapshot();
                response.request = Some("template_fit".into());
                return Ok(response);
            }
            TemplateActionOutcome::Properties => {
                let mut response = self.snapshot();
                response.request = Some("template_properties".into());
                return Ok(response);
            }
            TemplateActionOutcome::ToggleExplorer | TemplateActionOutcome::ToggleProperties => {
                let mut response = self.snapshot();
                response.request = Some(
                    if handler == ActionHandler::ToggleExplorer {
                        "toggle_explorer"
                    } else {
                        "toggle_properties"
                    }
                    .into(),
                );
                return Ok(response);
            }
            TemplateActionOutcome::Palette => {
                let mut response = self.snapshot();
                response.request = Some("palette".into());
                return Ok(response);
            }
        };
        let mut response = self.snapshot();
        response.message = Some(message.into());
        Ok(response)
    }
    pub(crate) fn template_request(&mut self, request: Request) -> Result<Response, BridgeError> {
        if self.templates.editor.is_none() {
            return Err(BridgeError::new("template", "Template editor is closed"));
        }
        match request {
            Request::TemplatePending {
                context,
                table_action,
                ..
            } => {
                let context = self.context(context)?;
                if self.templates.gesture.is_some() {
                    return Err(BridgeError::new(
                        "template",
                        "Finish or cancel the template gesture first",
                    ));
                }
                let operation = table_action
                    .map(|id| {
                        let id = ActionId::new(id).map_err(|e| BridgeError::new("action", e))?;
                        match self.registry.get(&id).map(|a| a.handler) {
                            Some(ActionHandler::Template(TemplateOperation::Table(op))) => Ok(op),
                            _ => Err(BridgeError::new("template", "Unknown table operation")),
                        }
                    })
                    .transpose()?;
                if self.templates.pending && self.templates.pending_table != operation {
                    return Err(BridgeError::new(
                        "template",
                        "Another set of template fields is pending",
                    ));
                }
                if matches!(
                    operation,
                    Some(TableOperation::Split | TableOperation::Merge)
                ) {
                    let valid = context.template_selection.as_ref().is_some_and(|s| {
                        s.allows(
                            self.templates.editor.as_ref().expect("checked editor"),
                            operation.expect("checked operation"),
                        )
                    });
                    if !valid
                        || (self.templates.pending
                            && self.templates.pending_selection != context.template_selection)
                    {
                        return Err(BridgeError::new(
                            "selection",
                            "Keep the original eligible table selection while editing",
                        ));
                    }
                }
                self.templates.pending = true;
                self.templates.pending_selection = context.template_selection;
                self.templates.pending_table = operation;
                Ok(Response::default())
            }
            Request::TemplateOperation {
                context,
                action,
                source,
                values,
                input,
            } => {
                let context = self.context(context)?;
                if self.templates.pending_table.is_some()
                    && self.templates.pending_selection != context.template_selection
                {
                    return Err(BridgeError::new(
                        "selection",
                        "The pending table selection changed",
                    ));
                }
                let params = if let Some(values) = values {
                    if let Some(ActionHandler::Template(TemplateOperation::Table(op))) = self
                        .registry
                        .get(
                            &ActionId::new(action.clone())
                                .map_err(|e| BridgeError::new("action", e))?,
                        )
                        .map(|a| a.handler)
                    {
                        let row: crate::template_table_edit::TableEditRow =
                            serde_json::from_value(values)
                                .map_err(|e| BridgeError::new("template", e))?;
                        row.validate_selection(&context)?;
                        row.proposal(self.templates.editor.as_ref().expect("checked editor"), op)?
                    } else {
                        let row: TemplateEditRow = serde_json::from_value(values)
                            .map_err(|e| BridgeError::new("template", e))?;
                        row.proposal(self.templates.editor.as_ref().ok_or_else(|| {
                            BridgeError::new("template", "Template editor is closed")
                        })?)?
                    }
                } else if let Some(source) = source {
                    let Some(SheetTemplate::Saved(pin)) = SheetTemplate::from_key(&source) else {
                        return Err(BridgeError::new(
                            "template",
                            "Select an exact saved template",
                        ));
                    };
                    ActionParameters::TemplateSource(pin)
                } else {
                    ActionParameters::None
                };
                self.invoke(ActionInvocation {
                    input_id: super::controller::input_id(&input)?,
                    action: ActionId::new(action).map_err(|e| BridgeError::new("action", e))?,
                    source: InvocationSource::Toolbar,
                    repeat: false,
                    context,
                    parameters: params,
                })
            }
            _ => Err(BridgeError::new("template", "Unknown template request")),
        }
    }
}
