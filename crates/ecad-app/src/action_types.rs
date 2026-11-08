//! Stable action identity and captured targeting, independent of widgets and IO.

use crate::{Revision, Session};
use ecad_domain::{DrawingPoint, ObjectRef, SheetId, ViewId};
use std::{collections::BTreeSet, error::Error, fmt};

/// A stable namespaced action ID, never a translated menu label or executable code.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActionId(String);
impl ActionId {
    /// Validate a dotted lowercase ASCII identity such as `selection.properties`.
    /// # Errors
    /// Requires at least two nonempty segments, starting with letters and containing
    /// only lowercase letters, digits, or underscores.
    pub fn new(value: impl Into<String>) -> Result<Self, ActionConfigError> {
        let value = value.into();
        if !value.contains('.')
            || !value.split('.').all(|segment| {
                segment
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_lowercase)
                    && segment
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            })
        {
            return Err(ActionConfigError::InvalidActionId(value));
        }
        Ok(Self(value))
    }
    /// Inspect the non-localized stable ID.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for ActionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Configuration could not be accepted without changing the prior effective setup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionConfigError {
    /// Identity fails the stable dotted-name contract.
    InvalidActionId(String),
    /// An action was registered twice.
    DuplicateAction(ActionId),
    /// Metadata needs nonblank labels/descriptions/search terms.
    EmptyMetadata,
    /// Use the explicit Unbound policy instead of an empty suggested-default list.
    EmptyDefaultBindings,
    /// A key must be a printable non-whitespace character or supported named key.
    InvalidKey,
    /// Ordered sequences must contain at least one chord.
    EmptySequence,
    /// An imported modifier list repeats the same modifier.
    DuplicateModifier,
    /// An imported override repeats an action instead of listing its bindings once.
    DuplicateOverride(ActionId),
}
impl fmt::Display for ActionConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid action configuration: {self:?}")
    }
}
impl Error for ActionConfigError {}

/// Keyboard focus owned by the host. Text fields include numeric editors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActionFocus {
    /// No document surface currently owns focus.
    #[default]
    Workspace,
    /// Drawing canvas.
    Canvas,
    /// Project/object tree.
    ProjectTree,
    /// Inspector controls outside an active text edit.
    Inspector,
    /// Native/web text or numeric editor; document shortcuts are suppressed.
    TextInput,
    /// Modal UI owns keyboard handling.
    Modal,
}
/// Binding scopes with explicit mutually exclusive focus contexts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BindingScope {
    /// Application workspace while a control does not own input.
    Application,
    /// Any document surface: canvas, tree, or inspector.
    Document,
    /// Canvas focus.
    Canvas,
    /// Canvas with an active transient placement tool.
    Tool,
    /// Project tree focus.
    ProjectTree,
    /// Inspector focus outside text editing.
    Inspector,
}
impl BindingScope {
    /// Test whether two contexts can be active simultaneously.
    pub fn overlaps(self, other: Self) -> bool {
        use BindingScope::*;
        self == other
            || matches!(self, Application | Document)
            || matches!(other, Application | Document)
            || matches!((self, other), (Canvas, Tool) | (Tool, Canvas))
    }
    /// Whether this context is currently active. Text/modal protection happens before routing.
    pub fn active(self, context: &ActionContext) -> bool {
        match self {
            Self::Application => true,
            Self::Document => {
                context.revision.is_some()
                    && matches!(
                        context.focus,
                        ActionFocus::Canvas | ActionFocus::ProjectTree | ActionFocus::Inspector
                    )
            }
            Self::Canvas => context.revision.is_some() && context.focus == ActionFocus::Canvas,
            Self::Tool => context.focus == ActionFocus::Canvas && context.tool_active,
            Self::ProjectTree => {
                context.revision.is_some() && context.focus == ActionFocus::ProjectTree
            }
            Self::Inspector => {
                context.revision.is_some() && context.focus == ActionFocus::Inspector
            }
        }
    }
}

/// Capture this before opening a menu/palette/dialog; revalidate before committing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActionContext {
    /// Observed document/session revision, never replaced silently during invocation.
    pub revision: Option<Revision>,
    /// Original target focus, not the transient popup's focus.
    pub focus: ActionFocus,
    /// Original selection; hover fallback is not implied.
    pub selection: BTreeSet<ObjectRef>,
    /// Original drawing view for placement actions.
    pub view: Option<ViewId>,
    /// Optional captured project sheet; supplied by sheet workspaces, absent in definition editors.
    pub sheet: Option<SheetId>,
    /// Original optional document anchor; popup coordinates never replace it.
    pub anchor: Option<DrawingPoint>,
    /// Host-owned transient placement state; it is not document data.
    pub tool_active: bool,
    /// Input-method composition owns keys, including composition outside native text controls.
    pub composing: bool,
    /// A focused/custom control has claimed keyboard handling.
    pub control_owns_key: bool,
}
impl ActionContext {
    /// Capture a document revision with canvas focus and no inferred selection/view/anchor.
    pub fn for_session(session: &Session) -> Self {
        Self {
            revision: Some(session.revision()),
            focus: ActionFocus::Canvas,
            ..Self::default()
        }
    }
    pub(crate) fn same_sequence_target(&self, other: &Self) -> bool {
        self.revision == other.revision
            && self.focus == other.focus
            && self.selection == other.selection
            && self.view == other.view
            && self.sheet == other.sheet
            && self.tool_active == other.tool_active
            && self.composing == other.composing
            && self.control_owns_key == other.control_owns_key
    }
}

/// Typed unavailability, shared by palette, menus, shortcuts, and execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionDisabled {
    /// Selection cannot participate in this drawing layout action.
    Layout(crate::LayoutError),
    /// Action is not registered.
    UnknownAction(ActionId),
    /// A working document is required.
    NoDocument,
    /// Captured context refers to an older/different session revision.
    StaleContext,
    /// Undo history is empty.
    NothingToUndo,
    /// Redo history is empty.
    NothingToRedo,
    /// Local text editing owns undo/redo.
    TextEditorOwnsUndo,
    /// Action needs selected existing objects.
    NoSelection,
    /// A captured selection object no longer exists.
    MissingTarget(ObjectRef),
    /// Placement requires an existing active drawing view.
    NoDrawingView,
    /// Captured sheet is unavailable or its placed view differs from the captured source view.
    InvalidSheetContext(SheetId),
    /// Placement is scoped to the original canvas context.
    CanvasRequired,
    /// There is no active transient tool to cancel.
    NoActiveTool,
}
