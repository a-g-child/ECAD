//! Guarded document ownership and one shared action/key routing path for desktop requests.
use crate::{
    annotation::Annotation, defaults, dto::*, layout_tool::LayoutTool, metadata,
    movement::Movement, path::PathDraft, projection, properties,
};
use ecad_adapters::JsonFile;
use ecad_app::*;

use std::{
    collections::BTreeSet,
    num::NonZeroU64,
    path::{Path, PathBuf},
};
#[path = "settings_requests.rs"]
mod settings_requests;
#[path = "tools.rs"]
mod tools;
/// One desktop document controller, independent of Tauri/windowing and testable locally.
pub struct DesktopBridge {
    pub(crate) document: Document<JsonFile>,
    path: Option<PathBuf>,
    registry: ActionRegistry,
    keymap: EffectiveKeymap,
    router: ActionInputRouter,
    dispatcher: ActionDispatcher,
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
                projection::demo()?,
                JsonFile::new(std::env::temp_dir().join("ecad-unsaved-never-written.json")),
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
        })
    }
    /// Configure per-user profile storage without performing IO or modifying the document.
    pub fn configure_profile_store(&mut self, path: impl Into<PathBuf>) {
        self.profiles.configure(path.into());
    }
    /// Opaque current session token for guarding asynchronous native file choosers.
    pub fn revision_key(&self) -> String {
        let r = self.document.session().revision();
        format!("{}:{}", r.epoch().get(), r.get())
    }
    /// Whether the native host must protect unsaved edits before replacing/closing a document.
    pub fn is_dirty(&self) -> bool {
        self.document.session().is_dirty()
            && (!self.demo
                || self.document.session().undo_len() > 0
                || self.document.session().redo_len() > 0)
    }
    /// Whether a native save-as chooser is required before a save can complete.
    pub fn needs_save_as(&self) -> bool {
        self.path.is_none()
    }
    /// Open an explicitly chosen local file. Dirty edits are never discarded implicitly.
    /// # Errors
    /// Rejects dirty replacement or invalid/missing files, retaining the current model/history.
    pub fn open(&mut self, path: &Path) -> Result<Response, BridgeError> {
        if self.is_dirty() {
            return Err(BridgeError::new(
                "dirty",
                "Save the current document before opening another project",
            ));
        }
        let document =
            Document::open(JsonFile::new(path)).map_err(|e| BridgeError::new("open", e))?;
        self.document = document;
        self.profiles.invalidate();
        self.sheet_tool.preview = None;
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
        bridge.document =
            Document::open(JsonFile::new(path)).map_err(|e| BridgeError::new("open", e))?;
        bridge.path = Some(path.into());
        bridge.demo = false;
        Ok(bridge)
    }
    /// Save as an explicitly chosen destination using the guarded document store contract.
    /// # Errors
    /// Existing unobserved destinations and IO failures preserve the document checkpoint/path.
    pub fn save_as(&mut self, path: &Path) -> Result<Response, BridgeError> {
        self.document
            .save_as(JsonFile::new(path))
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
        snapshot.dirty = self.is_dirty();
        snapshot.next_input = self.last_input.saturating_add(1).to_string();
        Response {
            snapshot: Some(snapshot),
            ..Response::default()
        }
    }
    pub(crate) fn context(&self, value: serde_json::Value) -> Result<ActionContext, BridgeError> {
        let ctx: Context =
            serde_json::from_value(value).map_err(|e| BridgeError::new("context", e))?;
        if ctx.revision != Token::from(self.document.session().revision()) {
            return Err(BridgeError::new(
                "stale",
                "The document changed. Refresh before applying this operation",
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
            focus,
            tool_active: self.movement.is_some()
                || self.layout.is_some()
                || self.annotation.is_some()
                || self.path_draft.is_some(),
            ..ActionContext::default()
        })
    }
    fn outcome(&mut self, outcome: ActionOutcome) -> Result<Response, BridgeError> {
        let mut response = self.snapshot();
        match outcome {
            ActionOutcome::Edited(_) => {}
            ActionOutcome::Saved => response.message = Some("Project saved".into()),
            ActionOutcome::Properties(p) => {
                response.properties = Some(properties::inspect(
                    self.document.session().project(),
                    p.objects,
                )?);
            }
            ActionOutcome::Request(r) => {
                response.request = Some(
                    match r.kind {
                        ActionRequestKind::Sheets => "sheets",
                        ActionRequestKind::CompanyProfiles => {
                            response.profiles = self.profiles.load()?.profiles;
                            "profiles"
                        }
                        ActionRequestKind::ProjectMetadata => {
                            response.metadata = Some(metadata::project(
                                self.document.session().project().metadata(),
                            ));
                            "metadata"
                        }
                        ActionRequestKind::DrawingSettings => {
                            response.defaults = Some(defaults::project(
                                r.settings.as_ref().expect("settings request"),
                            ));
                            "defaults"
                        }
                        ActionRequestKind::Palette => "palette",
                        ActionRequestKind::OpenProject => "open",
                        ActionRequestKind::FitDrawing => "fit",
                        ActionRequestKind::MoveSelection => "move",
                        ActionRequestKind::LabelPlacement => "annotation",
                        ActionRequestKind::PolylinePlacement => "path",
                        ActionRequestKind::RotateSelection => "rotate",
                        ActionRequestKind::MirrorSelection => "mirror",
                        ActionRequestKind::AlignSelection => "align",
                        ActionRequestKind::DistributeSelection => "distribute",
                        ActionRequestKind::CancelPlacement => {
                            let message = if self.path_draft.take().is_some() {
                                "Path cancelled"
                            } else if self.annotation.take().is_some() {
                                "Annotation cancelled"
                            } else if self.layout.take().is_some() {
                                "Transform cancelled"
                            } else {
                                "Move cancelled"
                            };
                            self.movement = None;
                            response.message = Some(message.into());
                            "cancel_move"
                        }
                        _ => "unsupported",
                    }
                    .into(),
                )
            }
            ActionOutcome::ConnectionReport(report) => {
                let mut rows = vec![vec![
                    "Conductor".into(),
                    "End A".into(),
                    "End B".into(),
                    "Cable".into(),
                ]];
                for w in report.conductors() {
                    let ends = w.endpoints.map(|id| {
                        id.map_or_else(
                            || "Free end".into(),
                            |id| {
                                let t = report.terminal(id).expect("report terminal");
                                format!(
                                    "{} : {}",
                                    report.device(t.device).expect("owner").designation,
                                    t.number
                                )
                            },
                        )
                    });
                    rows.push(vec![
                        w.id.to_string(),
                        ends[0].clone(),
                        ends[1].clone(),
                        w.cable.map_or_else(
                            || "—".into(),
                            |id| report.cable(id).expect("cable").designation.clone(),
                        ),
                    ]);
                }
                response.report = Some(rows);
            }
            _ => {
                return Err(BridgeError::new(
                    "unsupported",
                    "This tool's UI is not available in the first desktop slice",
                ));
            }
        }
        Ok(response)
    }
    pub(crate) fn invoke(&mut self, invocation: ActionInvocation) -> Result<Response, BridgeError> {
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
        if let Request::Action { input, .. }
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
        match request {
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
