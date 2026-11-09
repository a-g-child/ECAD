//! Guarded document ownership and one shared action/key routing path for desktop requests.
use crate::{
    annotation::Annotation, defaults, dto::*, layout_tool::LayoutTool, metadata,
    movement::Movement, path::PathDraft, projection, properties,
};
use ecad_adapters::JsonFile;
use ecad_app::*;

use std::{
    num::NonZeroU64,
    path::{Path, PathBuf},
};
#[path = "outcomes.rs"]
mod outcomes;
#[path = "settings_requests.rs"]
mod settings_requests;
#[path = "tools.rs"]
mod tools;
/// One desktop document controller, independent of Tauri/windowing and testable locally.
pub struct DesktopBridge {
    pub(crate) document: Document<JsonFile>,
    path: Option<PathBuf>,
    pub(crate) registry: ActionRegistry,
    keymap: EffectiveKeymap,
    pub(crate) router: ActionInputRouter,
    pub(crate) dispatcher: ActionDispatcher,
    demo: bool,
    last_input: u64,
    clock: std::time::Instant,
    movement: Option<Movement>,
    layout: Option<LayoutTool>,
    annotation: Option<Annotation>,
    path_draft: Option<PathDraft>,
    move_serial: u64,
    pub(crate) sheet_tool: crate::sheets::SheetTool,
    profiles: crate::profiles::ProfilePanel,
    pub(crate) templates: crate::template_workspace::TemplateWorkspace,
}
impl DesktopBridge {
    /// Start an explicitly unsaved demonstration backed by the real electrical/drawing model.
    /// # Errors
    /// Returns a sample/registry initialization failure without exposing a partial controller.
    pub fn demo() -> Result<Self, BridgeError> {
        let registry = ActionRegistry::builtins();
        let platform = if cfg!(target_os = "macos") {
            ShortcutPlatform::MacOS
        } else if cfg!(target_os = "windows") {
            ShortcutPlatform::Windows
        } else {
            ShortcutPlatform::Linux
        };
        let keymap = compile_keymap(&registry, &KeymapOverrides::default(), platform)
            .map_err(|e| BridgeError::new("keymap", format!("{e:?}")))?;
        let router =
            ActionInputRouter::new(keymap.clone(), NonZeroU64::new(1200).expect("constant"));
        Ok(Self {
            document: Document::new(
                ecad_adapters::migrate_template_assignments(&projection::demo()?)
                    .map_err(|e| BridgeError::new("templates", e))?,
                JsonFile::new(std::env::temp_dir().join("ecad-unsaved-never-written.json"))
                    .with_saved_templates(),
            ),
            path: None,
            registry,
            keymap,
            router,
            dispatcher: ActionDispatcher::default(),
            demo: true,
            last_input: 0,
            clock: std::time::Instant::now(),
            movement: None,
            layout: None,
            annotation: None,
            path_draft: None,
            move_serial: 0,
            sheet_tool: crate::sheets::SheetTool::default(),
            profiles: crate::profiles::ProfilePanel::default(),
            templates: crate::template_workspace::TemplateWorkspace::default(),
        })
    }
    /// Configure per-user profile storage without performing IO or modifying the document.
    pub fn configure_profile_store(&mut self, path: impl Into<PathBuf>) {
        self.profiles.configure(path.into());
    }
    /// Opaque current session token for guarding asynchronous native file choosers.
    pub fn revision_key(&self) -> String {
        let r = self.document.session().revision();
        format!(
            "{}:{}:{:?}",
            r.epoch().get(),
            r.get(),
            self.templates
                .editor
                .as_ref()
                .map(TemplateDocument::revision)
        )
    }
    /// Whether the native host must protect unsaved edits before replacing/closing a document.
    pub fn is_dirty(&self) -> bool {
        self.project_dirty() || self.templates.dirty()
    }
    fn project_dirty(&self) -> bool {
        self.document.session().is_dirty()
            && (!self.demo
                || self.document.session().undo_len() > 0
                || self.document.session().redo_len() > 0)
    }
    /// Whether a native save-as chooser is required before a save can complete.
    pub fn needs_save_as(&self) -> bool {
        self.templates.editor.is_none() && self.path.is_none()
    }
    /// Open an explicitly chosen local file. Dirty edits are never discarded implicitly.
    /// # Errors
    /// Rejects dirty replacement or invalid/missing files, retaining the current model/history.
    pub fn open(&mut self, path: &Path) -> Result<Response, BridgeError> {
        if self.templates.editor.is_some() {
            return Err(BridgeError::new(
                "template",
                "Close the template editor before opening a project",
            ));
        }
        if self.is_dirty() {
            return Err(BridgeError::new(
                "dirty",
                "Save the current document before opening another project",
            ));
        }
        let document = Document::open(JsonFile::new(path).with_saved_templates())
            .map_err(|e| BridgeError::new("open", e))?;
        self.document = document;
        self.profiles.invalidate();
        self.sheet_tool.preview = None;
        self.sheet_tool.new_view = None;
        self.movement = None;
        self.layout = None;
        self.annotation = None;
        self.path_draft = None;
        self.demo = false;
        self.path = Some(path.into());
        self.router.cancel_pending();
        Ok(self.snapshot())
    }
    /// Load a chosen file as the initial document before UI exposure, including browser tests.
    /// # Errors
    /// Returns a validation/IO error without replacing a live user document.
    pub fn from_path(path: &Path) -> Result<Self, BridgeError> {
        let mut bridge = Self::demo()?;
        bridge.document = Document::open(JsonFile::new(path).with_saved_templates())
            .map_err(|e| BridgeError::new("open", e))?;
        bridge.path = Some(path.into());
        bridge.demo = false;
        Ok(bridge)
    }
    /// Save as an explicitly chosen destination using the guarded document store contract.
    /// # Errors
    /// Existing unobserved destinations and IO failures preserve the document checkpoint/path.
    pub fn save_as(&mut self, path: &Path) -> Result<Response, BridgeError> {
        if self.templates.editor.is_some() {
            return Err(BridgeError::new(
                "template",
                "Use Save template revision in the template editor",
            ));
        }
        self.document
            .save_as(JsonFile::new(path).with_saved_templates())
            .map_err(|e| BridgeError::new("save", e))?;
        self.path = Some(path.into());
        self.demo = false;
        Ok(self.snapshot())
    }
    /// Return a fresh read projection for resynchronization without mutating the session.
    pub fn snapshot(&self) -> Response {
        let mut snapshot = projection::snapshot(
            &self.document,
            self.path.as_deref(),
            &self.registry,
            &self.keymap,
        );
        snapshot.dirty = self.project_dirty();
        snapshot.template_editor = self.templates.view(self.document.session().project());
        if let Some(editor) = &self.templates.editor {
            let context = ActionContext {
                editor: Some(editor.revision()),
                focus: ActionFocus::Canvas,
                tool_active: self.templates.gesture.is_some(),
                ..ActionContext::default()
            };
            for row in &mut snapshot.actions {
                let id = ActionId::new(row.id.clone()).expect("registered ID");
                let state = self.registry.template_state(&id, &context, editor);
                row.enabled = state.disabled.is_none();
                row.reason = state.disabled.map(|e| format!("{e:?}"));
                if id.as_str() == "document.save" {
                    row.label = "Save template revision".into();
                }
            }
        }
        snapshot.next_input = self.last_input.saturating_add(1).to_string();
        Response {
            snapshot: Some(snapshot),
            ..Response::default()
        }
    }
    pub(crate) fn has_active_tools(&self) -> bool {
        self.movement.is_some()
            || self.layout.is_some()
            || self.annotation.is_some()
            || self.path_draft.is_some()
    }
    pub(crate) fn invoke(&mut self, invocation: ActionInvocation) -> Result<Response, BridgeError> {
        if invocation.context.editor.is_some() {
            return self.template_invoke(invocation);
        }
        if invocation.action.as_str() == "document.save" && self.path.is_none() {
            return Ok(Response {
                request: Some("save_as".into()),
                ..Response::default()
            });
        }
        let outcome = self
            .dispatcher
            .invoke(&self.registry, invocation, Some(&mut self.document))
            .map_err(|e| BridgeError::new("action", e))?;
        self.outcome(outcome)
    }
    /// Validate and execute one strict bridge request through normal core contracts.
    /// # Errors
    /// Invalid/stale input, model validation, replay/protected input, and IO errors preserve
    /// the document/history. Invalid move updates deliberately invalidate older proposals.
    pub fn handle(&mut self, request: Request) -> Result<Response, BridgeError> {
        if let Request::TemplateTablePreview { input, .. }
        | Request::TemplateTableCommit { input, .. }
        | Request::TemplateOperation { input, .. }
        | Request::TemplatePending { input, .. }
        | Request::EditPicture { input, .. }
        | Request::Action { input, .. }
        | Request::Key { input, .. }
        | Request::CommitMove { input, .. }
        | Request::CommitTransform { input, .. }
        | Request::CommitAnnotation { input, .. }
        | Request::CommitSheet { input, .. }
        | Request::ChangeSheet { input, .. }
        | Request::SaveProfile { input, .. }
        | Request::CommitProfileImport { input, .. }
        | Request::UpdateMetadata { input, .. }
        | Request::UpdateDefaults { input, .. }
        | Request::CommitPath { input, .. } = &request
        {
            let id = input_id(input)?.get();
            if id <= self.last_input {
                return Err(BridgeError::new("replayed", "Input was already handled"));
            }
            self.last_input = id;
        }
        if self.templates.editor.is_some()
            && !matches!(
                &request,
                Request::Action { .. }
                    | Request::Key { .. }
                    | Request::Snapshot {}
                    | Request::ImportPicture { .. }
                    | Request::TemplateOperation { .. }
                    | Request::TemplatePending { .. }
                    | Request::TemplateActions { .. }
                    | Request::TemplateTablePreview { .. }
                    | Request::TemplateTableCommit { .. }
            )
        {
            return Err(BridgeError::new(
                "template",
                "Return to the project drawing before using project tools",
            ));
        }
        match request {
            Request::ImportPicture { data } => {
                let asset = ecad_adapters::picture_from_base64(&data)
                    .map_err(|e| BridgeError::new("picture", e))?;
                Ok(Response {
                    imported_picture: Some(crate::picture_rows::ImportedPictureRow {
                        pixels: [asset.width().to_string(), asset.height().to_string()],
                        picture: crate::picture_rows::PictureRow {
                            data: ecad_adapters::picture_base64(&asset),
                            mime: asset.format().mime().into(),
                            fit: "contain".into(),
                        },
                    }),
                    ..Response::default()
                })
            }
            Request::EditPicture {
                context,
                id,
                values,
                input,
            } => {
                let context = self.context(context)?;
                let target = id
                    .map(|s| {
                        s.parse::<ecad_domain::PictureId>()
                            .map_err(|e| BridgeError::new("picture", e))
                    })
                    .transpose()?;
                let value = values
                    .map(|v| {
                        let row: crate::picture_rows::PlacementRow = serde_json::from_value(v)
                            .map_err(|e| BridgeError::new("picture", e))?;
                        let id = match target {
                            Some(id) => id,
                            None => ecad_adapters::RandomIds::picture()
                                .map_err(|e| BridgeError::new("picture", e))?,
                        };
                        row.drawing(
                            id,
                            context.view.ok_or_else(|| {
                                BridgeError::new("picture", "Select a schematic drawing view")
                            })?,
                        )
                        .map(Box::new)
                    })
                    .transpose()?;
                self.invoke(ActionInvocation {
                    input_id: input_id(&input)?,
                    action: ActionId::new("drawing.picture").expect("registered"),
                    source: InvocationSource::Toolbar,
                    repeat: false,
                    context,
                    parameters: ActionParameters::Picture { target, value },
                })
            }
            Request::TemplateActions { context } => self.template_actions(context),
            request @ (Request::TemplateTablePreview { .. }
            | Request::TemplateTableCommit { .. }) => self.template_table_request(request),
            request @ (Request::TemplateOperation { .. } | Request::TemplatePending { .. }) => {
                self.template_request(request)
            }
            request @ (Request::BeginMove { .. }
            | Request::PreviewMove { .. }
            | Request::CommitMove { .. }
            | Request::BeginTransform { .. }
            | Request::PreviewTransform { .. }
            | Request::CommitTransform { .. }
            | Request::BeginAnnotation { .. }
            | Request::PreviewAnnotation { .. }
            | Request::CommitAnnotation { .. }
            | Request::BeginPath { .. }
            | Request::PreviewPath { .. }
            | Request::AddPathVertex { .. }
            | Request::BackPathVertex { .. }
            | Request::CommitPath { .. }) => self.tool(request),
            request @ (Request::NewSheet { .. }
            | Request::PreviewSheet { .. }
            | Request::CommitSheet { .. }
            | Request::ChangeSheet { .. }
            | Request::CancelSheet {}) => self.sheet_request(request),
            Request::Snapshot {} => Ok(self.snapshot()),
            request @ (Request::UpdateMetadata { .. }
            | Request::UpdateDefaults { .. }
            | Request::CancelProfileImport {}
            | Request::LoadProfiles {}
            | Request::SaveProfile { .. }
            | Request::PreviewProfileImport { .. }
            | Request::CommitProfileImport { .. }) => self.settings_request(request),
            Request::Inspect { context } => {
                let context = self.context(context)?;
                Ok(Response {
                    properties: Some(properties::inspect(
                        self.document.session().project(),
                        context.selection,
                    )?),
                    ..Response::default()
                })
            }
            Request::Edit {
                context,
                property,
                value,
            } => {
                let context = self.context(context)?;
                let commands = properties::edit(
                    self.document.session().project(),
                    context.selection,
                    &property,
                    &value,
                )?;
                self.document
                    .execute_batch(context.revision.expect("validated"), commands)
                    .map_err(|e| BridgeError::new("edit", e))?;
                Ok(self.snapshot())
            }
            Request::EditAppearance { context, changes } => {
                let context = self.context(context)?;
                let commands = properties::appearance(
                    self.document.session().project(),
                    context.selection,
                    changes,
                )?;
                self.document
                    .execute_batch(context.revision.expect("validated"), commands)
                    .map_err(|e| BridgeError::new("edit", e))?;
                Ok(self.snapshot())
            }
            Request::Action {
                context,
                action,
                input,
            } => {
                let context = self.context(context)?;
                let invocation = ActionInvocation {
                    input_id: input_id(&input)?,
                    action: ActionId::new(action).map_err(|e| BridgeError::new("action", e))?,
                    source: InvocationSource::Toolbar,
                    repeat: false,
                    context,
                    parameters: ActionParameters::None,
                };
                self.invoke(invocation)
            }
            Request::Key {
                context,
                key,
                control,
                meta,
                alt,
                shift,
                repeat,
                composing,
                input,
                now_ms: _,
            } => {
                let mut context = self.context(context)?;
                context.composing = composing;
                let key = match key.as_str() {
                    "Escape" => ShortcutKey::named(NamedKey::Escape),
                    _ => {
                        let mut chars = key.chars();
                        let character = chars
                            .next()
                            .ok_or_else(|| BridgeError::new("key", "Empty key"))?;
                        if chars.next().is_some() {
                            return Ok(Response::default());
                        }
                        ShortcutKey::character(character).map_err(|e| BridgeError::new("key", e))?
                    }
                };
                let event = ActionKeyEvent {
                    id: input_id(&input)?,
                    chord: ResolvedChord {
                        key,
                        modifiers: PhysicalModifiers {
                            control,
                            meta,
                            alt,
                            shift,
                        },
                    },
                    repeat,
                };
                let now_ms = u64::try_from(self.clock.elapsed().as_millis()).unwrap_or(u64::MAX);
                match self.router.route(&self.registry, event, &context, now_ms) {
                    KeyRoute::Invoke(invocation) => self.invoke(*invocation),
                    KeyRoute::Pending { .. } => Ok(Response {
                        message: Some("Waiting for the next shortcut key…".into()),
                        request: Some("key_prefix".into()),
                        ..Response::default()
                    }),
                    _ => Ok(Response::default()),
                }
            }
        }
    }
}
pub(crate) fn input_id(text: &str) -> Result<InputId, BridgeError> {
    text.parse().ok().and_then(InputId::new).ok_or_else(|| {
        BridgeError::new("input", "Input identity must be a nonzero unsigned integer")
    })
}
