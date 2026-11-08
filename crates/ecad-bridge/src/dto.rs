//! Explicit bridge-v1 JSON DTOs, separate from domain structs and project persistence.
use ecad_app::*;
use ecad_domain::*;
use serde::{Deserialize, Serialize};

/// A recoverable host/projection error; rejected requests preserve the live document.
#[derive(Clone, Debug, Serialize)]
pub struct BridgeError {
    /// Stable broad category for host recovery.
    pub code: String,
    /// Context retained for user-visible diagnostics.
    pub message: String,
}
impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for BridgeError {}
impl BridgeError {
    pub(crate) fn new(code: &str, error: impl std::fmt::Display) -> Self {
        Self {
            code: code.into(),
            message: error.to_string(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Token {
    pub epoch: String,
    pub sequence: String,
}
impl From<Revision> for Token {
    fn from(r: Revision) -> Self {
        Self {
            epoch: r.epoch().get().to_string(),
            sequence: r.get().to_string(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Target {
    pub kind: String,
    pub id: String,
}
impl Target {
    pub fn object(&self) -> Result<ObjectRef, BridgeError> {
        macro_rules! parse {
            ($variant:ident) => {
                ObjectRef::$variant(
                    self.id
                        .parse()
                        .map_err(|e| BridgeError::new("invalid_target", e))?,
                )
            };
        }
        Ok(match self.kind.as_str() {
            "device" => parse!(Device),
            "terminal" => parse!(Terminal),
            "subsystem" => parse!(Subsystem),
            "port" => parse!(Port),
            "conductor" => parse!(Conductor),
            "cable" => parse!(Cable),
            "view" => parse!(View),
            "label" => parse!(Label),
            "polyline" => parse!(Polyline),
            "anchor" => parse!(Anchor),
            "route" => parse!(Route),
            _ => return Err(BridgeError::new("invalid_target", "Unknown object kind")),
        })
    }
}
impl From<ObjectRef> for Target {
    fn from(o: ObjectRef) -> Self {
        let (kind, id) = match o {
            ObjectRef::Device(id) => ("device", id.to_string()),
            ObjectRef::Terminal(id) => ("terminal", id.to_string()),
            ObjectRef::Subsystem(id) => ("subsystem", id.to_string()),
            ObjectRef::Port(id) => ("port", id.to_string()),
            ObjectRef::Conductor(id) => ("conductor", id.to_string()),
            ObjectRef::Cable(id) => ("cable", id.to_string()),
            ObjectRef::View(id) => ("view", id.to_string()),
            ObjectRef::Label(id) => ("label", id.to_string()),
            ObjectRef::Polyline(id) => ("polyline", id.to_string()),
            ObjectRef::Anchor(id) => ("anchor", id.to_string()),
            ObjectRef::Route(id) => ("route", id.to_string()),
        };
        Self {
            kind: kind.into(),
            id,
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Context {
    #[serde(default)]
    pub editor: Option<Token>,
    pub revision: Token,
    pub selection: Vec<Target>,
    pub view: Option<String>,
    pub sheet: Option<String>,
    pub focus: String,
}
/// Strict request envelope. IDs and exact document values cross JSON as strings.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    /// Commit or invoke an explicitly targeted template editor operation.
    TemplateOperation {
        /// Captured project and independent editor context.
        context: serde_json::Value,
        /// Registered action identity.
        action: String,
        /// Exact saved source for Open, otherwise absent.
        source: Option<String>,
        /// Closed property values for Apply, otherwise absent.
        values: Option<serde_json::Value>,
        /// Shared monotonic host input identity.
        input: String,
    },
    /// Protect uncommitted template property fields during native close/quit.
    TemplatePending {
        /// Registered table operation owning these transient fields; absent for inspector fields.
        #[serde(default)]
        table_action: Option<String>,
        /// Captured independent editor context.
        context: serde_json::Value,
        /// Shared monotonic host input identity.
        input: String,
    },
    /// Resynchronize a full revisioned read projection.
    Snapshot {},
    /// Inspect a captured selection, retaining every applicable property.
    Inspect {
        /// Captured targets and revision, validated by the controller.
        context: serde_json::Value,
    },
    /// Edit one typed property across its applicable selection as one undo operation.
    Edit {
        /// Captured original selection/revision.
        context: serde_json::Value,
        /// Stable property key from registry metadata.
        property: String,
        /// Literal editable value, never an inferred identity.
        value: String,
    },
    /// Prepare initial values for a new, uncommitted paper sheet.
    NewSheet {
        /// Optional typed creation flow; absent retains the existing sheet-manager draft behavior.
        kind: Option<String>,
        /// Captured document context.
        context: serde_json::Value,
    },
    /// Validate exact paper values and retain a closed sheet-layout proposal.
    PreviewSheet {
        /// Captured document context.
        context: serde_json::Value,
        /// Strict sheet editor strings.
        sheet: serde_json::Value,
        /// Insert rather than replace existing identity.
        create: bool,
    },
    /// Commit exactly the reviewed sheet proposal.
    CommitSheet {
        /// Retained proposal identity.
        token: String,
        /// Monotonic host input identity.
        input: String,
    },
    /// Explicitly remove or reorder one sheet without changing its drawing number or source view.
    ChangeSheet {
        /// Captured document context.
        context: serde_json::Value,
        /// Existing sheet identity.
        id: String,
        /// remove or move.
        operation: String,
        /// Final ordinal for movement.
        index: usize,
        /// Monotonic host input identity.
        input: String,
    },
    /// Cancel the retained sheet proposal without editing the project.
    CancelSheet {},
    /// Discard the retained profile-copy proposal without editing the project.
    CancelProfileImport {},
    /// Reload the host-configured profile file, invalidating old observations/proposals.
    LoadProfiles {},
    /// Save a new or existing company/contact profile outside document history.
    SaveProfile {
        /// Current profile-file observation token.
        token: String,
        /// Existing profile identity; None generates a new identity.
        id: Option<String>,
        /// Nonblank literal display name.
        name: String,
        /// Company/contact keys only, with literal multiline text.
        values: std::collections::BTreeMap<String, String>,
        /// Shared monotonic host input identity.
        input: String,
    },
    /// Prepare a closed project-copy proposal from saved profile values.
    PreviewProfileImport {
        /// Captured document context/revision.
        context: serde_json::Value,
        /// Current profile-file observation token.
        token: String,
        /// Saved profile identity.
        id: String,
        /// Explicit selected field keys; selecting an empty value deliberately clears it.
        fields: Vec<String>,
    },
    /// Apply exactly the reviewed profile copy as one undoable project edit.
    CommitProfileImport {
        /// Retained immutable proposal identity.
        token: String,
        /// Shared monotonic host input identity.
        input: String,
    },
    /// Replace portable project metadata as one registered document edit.
    UpdateMetadata {
        /// Optional reviewed creation-template defaults, applied atomically with the metadata.
        template_defaults: Option<serde_json::Value>,
        /// Original project revision and action context.
        context: serde_json::Value,
        /// Stable catalog keys to literal text; empty values remove entries.
        values: std::collections::BTreeMap<String, String>,
        /// Shared monotonic host input identity.
        input: String,
    },
    /// Replace project fallbacks, retaining every creation preset and named style.
    UpdateDefaults {
        /// Captured document revision and action context.
        context: serde_json::Value,
        /// Strict font/text_height/stroke_width/colour string fields.
        defaults: serde_json::Value,
        /// Shared monotonic host input identity.
        input: String,
    },
    /// Invoke a centrally registered operation through its normal shared dispatcher.
    Action {
        /// Captured original targets/revision.
        context: serde_json::Value,
        /// Stable registry action ID.
        action: String,
        /// One shared monotonic host input identity, encoded losslessly.
        input: String,
    },
    /// Start transient movement with an explicitly selected reference origin.
    BeginMove {
        /// Original view, selection and revision.
        context: serde_json::Value,
        /// Selected label, polyline or terminal anchor.
        reference: serde_json::Value,
    },
    /// Replace the held proposal. Invalid updates invalidate the previous proposal.
    PreviewMove {
        /// Opaque movement lifetime identity.
        session: String,
        /// Exact micrometre coordinate text.
        x: String,
        /// Exact micrometre coordinate text.
        y: String,
        /// Numeric placement bypasses snapping.
        exact: bool,
        /// Temporary pointer snap suppression.
        suppressed: bool,
        /// Explicit ranked candidate, otherwise best.
        candidate: Option<usize>,
        /// Inclusive pointer tolerance in micrometres.
        tolerance_um: u32,
    },
    /// Commit only the retained immutable proposal, through the shared dispatcher.
    CommitMove {
        /// Opaque proposal identity, superseded by every update.
        token: String,
        /// Shared monotonic host input identity.
        input: String,
    },
    /// Begin exact layout with an explicit reference-origin seed pivot/coordinate.
    BeginTransform {
        /// Captured original selection/view/revision.
        context: serde_json::Value,
        /// Selected reference whose exact origin seeds the visible pivot fields.
        reference: serde_json::Value,
        /// Registered rotate, mirror, align or distribute handler kind.
        kind: String,
    },
    /// Replace a held transform, invalidating prior proposals before validation.
    PreviewTransform {
        /// Opaque tool lifetime.
        session: String,
        /// Exact pivot x, or the alignment coordinate for either axis.
        x: String,
        /// Exact pivot y; unused by alignment/distribution.
        y: String,
        /// Explicit quarter-turn or reflection policy.
        operation: String,
    },
    /// Apply only the reviewed retained transform through its original action.
    CommitTransform {
        /// Current proposal token.
        token: String,
        /// Shared host input identity.
        input: String,
    },
    /// Begin a new annotation in the captured view, initially without text.
    BeginAnnotation {
        /// Captured view, revision and action context.
        context: serde_json::Value,
    },
    /// Replace the annotation proposal; invalid updates invalidate its previous token.
    PreviewAnnotation {
        /// Opaque placement lifetime.
        session: String,
        /// Strict text, exact coordinates and snap-choice input envelope.
        placement: serde_json::Value,
    },
    /// Create only the retained annotation proposal through the normal T action.
    CommitAnnotation {
        /// Current valid proposal token.
        token: String,
        /// Shared host input identity.
        input: String,
    },
    /// Begin one retained graphical path in the captured drawing view.
    BeginPath {
        /// Original view and revision, retained for normal P commit.
        context: serde_json::Value,
    },
    /// Preview the cursor without accepting a vertex or mutating the document.
    PreviewPath {
        /// Opaque draft lifetime.
        session: String,
        /// Strict exact/pointer coordinate and snap-choice envelope.
        placement: serde_json::Value,
    },
    /// Accept the current reviewed cursor as one draft vertex, consuming its token.
    AddPathVertex {
        /// Current acknowledged proposal identity.
        token: String,
    },
    /// Remove the last accepted draft vertex without touching document history.
    BackPathVertex {
        /// Current acknowledged proposal identity.
        token: String,
    },
    /// Finish only the accepted vertices as one ordinary P creation operation.
    CommitPath {
        /// Current accepted-path proposal.
        token: String,
        /// Shared host input identity.
        input: String,
    },
    /// Route one logical keyboard event using the core keymap and protected-input policy.
    Key {
        /// Captured targets/focus/revision.
        context: serde_json::Value,
        /// Logical printable character or named key.
        key: String,
        /// Explicit Control modifier.
        control: bool,
        /// Explicit Command/Meta modifier.
        meta: bool,
        /// Explicit Alt modifier.
        alt: bool,
        /// Explicit Shift modifier.
        shift: bool,
        /// Native/web key auto-repeat flag.
        repeat: bool,
        /// IME owns this event.
        composing: bool,
        /// Shared monotonic host input ID.
        input: String,
        /// Legacy caller timing hint; the host uses its own monotonic clock across reloads.
        now_ms: u64,
    },
}
/// One bridge-v1 response. Fields are transport data, never mutable authoritative model access.
#[derive(Clone, Debug, Serialize, Default)]
pub struct Response {
    pub(crate) snapshot: Option<Snapshot>,
    pub(crate) properties: Option<Vec<PropertyRow>>,
    pub(crate) report: Option<Vec<Vec<String>>>,
    pub(crate) request: Option<String>,
    pub(crate) message: Option<String>,
    pub(crate) movement: Option<MoveRow>,
    pub(crate) transform: Option<TransformRow>,
    pub(crate) annotation: Option<AnnotationRow>,
    pub(crate) path: Option<PathRow>,
    pub(crate) defaults: Option<crate::defaults::DefaultsRow>,
    pub(crate) metadata: Option<Vec<crate::metadata::MetadataRow>>,
    pub(crate) project_templates: Option<crate::project_templates::TemplateCatalog>,
    pub(crate) activate_sheet: Option<String>,
    pub(crate) profiles: Option<crate::profiles::ProfileCatalog>,
    pub(crate) profile_import: Option<crate::profiles::ImportRow>,
    pub(crate) sheet_draft: Option<crate::sheets::SheetRow>,
    pub(crate) sheet_preview: Option<crate::sheets::SheetPreviewRow>,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Snapshot {
    pub template_editor: Option<crate::template_rows::TemplateView>,
    pub template_choices: Vec<crate::project_templates::SavedTemplateRow>,
    pub drawing_documents: Vec<crate::document_rows::DrawingDocumentRow>,
    pub sheets: Vec<crate::sheets::SheetRow>,
    pub drawing_lists: Vec<crate::drawing_list_rows::DrawingListRow>,
    pub sheet_frames: Vec<crate::frame_rows::FrameRow>,
    pub bridge_version: u8,
    pub next_input: String,
    pub revision: Token,
    pub name: String,
    pub path: Option<String>,
    pub dirty: bool,
    pub views: Vec<ViewRow>,
    pub objects: Vec<TreeRow>,
    pub scene: Vec<SceneRow>,
    pub actions: Vec<ActionRow>,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ViewRow {
    pub id: String,
    pub name: String,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct TreeRow {
    pub target: Target,
    pub label: String,
    pub parent: Option<Target>,
    pub view: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct SceneRow {
    pub target: Target,
    pub view: String,
    pub label: String,
    pub shape: String,
    pub points: Vec<[f64; 2]>,
    pub colour: String,
    pub width: f64,
    pub text_height: f64,
    pub font: String,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ActionRow {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub reason: Option<String>,
    pub shortcuts: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct PropertyRow {
    pub key: String,
    pub label: String,
    pub kind: String,
    pub display: String,
    pub value: Option<String>,
    pub mixed: bool,
    pub editable: bool,
    pub applicable: usize,
    pub unsupported: usize,
    pub restricted: usize,
    pub origin: String,
    pub reason: String,
}

/// Transient read projection; commit accepts its token rather than client geometry.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct MoveRow {
    pub session: String,
    pub token: String,
    pub x: String,
    pub y: String,
    pub scene: Vec<SceneRow>,
    pub candidates: usize,
    pub chosen: Option<usize>,
    pub snap: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TransformRow {
    pub session: String,
    pub token: String,
    pub x: String,
    pub y: String,
    pub operation: String,
    pub scene: Vec<SceneRow>,
    pub valid: bool,
    pub changed: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AnnotationRow {
    #[serde(flatten)]
    pub placement: MoveRow,
    pub text: String,
    pub valid: bool,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AnnotationInput {
    pub text: String,
    pub x: String,
    pub y: String,
    pub exact: bool,
    pub suppressed: bool,
    pub candidate: Option<usize>,
    pub tolerance_um: u32,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PathRow {
    #[serde(flatten)]
    pub placement: MoveRow,
    pub vertices: Vec<[String; 2]>,
    pub rubber: Option<SceneRow>,
    pub can_add: bool,
    pub can_finish: bool,
}
