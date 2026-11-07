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
    pub revision: Token,
    pub selection: Vec<Target>,
    pub view: Option<String>,
    pub focus: String,
}
/// Strict request envelope. IDs and exact document values cross JSON as strings.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
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
    /// Begin an orthogonal transform with an explicit reference-origin seed pivot.
    BeginTransform {
        /// Captured original selection/view/revision.
        context: serde_json::Value,
        /// Selected reference whose exact origin seeds the visible pivot fields.
        reference: serde_json::Value,
        /// Registered rotate or mirror handler kind.
        kind: String,
    },
    /// Replace a held transform, invalidating prior proposals before validation.
    PreviewTransform {
        /// Opaque tool lifetime.
        session: String,
        /// Exact micrometre pivot x.
        x: String,
        /// Exact micrometre pivot y.
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
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Snapshot {
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
