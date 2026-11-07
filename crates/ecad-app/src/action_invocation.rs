//! Shared invocation data, typed transient requests/projections, and preserved source errors.

use crate::*;
use ecad_domain::{DrawingPoint, DrawingSettings, LabelId, ModelError, PolylineId};
use std::{error::Error, fmt, num::NonZeroU64};

/// Host-issued monotonic input identity, shared by native and web handlers for one event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InputId(NonZeroU64);
impl InputId {
    /// Wrap a nonzero host-issued monotonic ID; zero is invalid.
    pub fn new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }
    /// Inspect the input sequence number, which is not a project/session revision.
    pub fn get(self) -> u64 {
        self.0.get()
    }
}
/// Invocation origin; accelerators receive the same input protection as web shortcuts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvocationSource {
    /// Web/host keyboard router.
    Keyboard,
    /// Native accelerator for the same normalized input stream.
    NativeAccelerator,
    /// Explicit menu selection.
    Menu,
    /// Explicit toolbar activation.
    Toolbar,
    /// Command-palette selection.
    Palette,
    /// Explicit contextual menu selection.
    ContextMenu,
}
/// Concrete parameters; missing placement data becomes a request, not inferred geometry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ActionParameters {
    /// Prepare exact geometry for a layout action without changing the document.
    PreviewLayout(LayoutOperation),
    /// Commit an inspected closed layout proposal for the matching captured action.
    CommitLayout(LayoutPreview),
    /// Prepare a non-mutating move preview using the captured selection and view.
    PreviewMove {
        /// Explicit selected origin, independent of pointer hover or popup position.
        reference: ecad_domain::ObjectRef,
        /// Exact or snapped placement policy.
        placement: MovePlacement,
    },
    /// Commit an already inspected, closed move preview at its original revision.
    CommitMove(MovePreview),
    /// Explicit numeric translation, alignment, or distribution for captured drawing selection.
    Layout(LayoutOperation),
    /// Complete existing-conductor routing input, with view and stroke preset taken from context.
    Route {
        /// Fresh whole-route identity.
        id: ecad_domain::RouteId,
        /// Existing real electrical conductor.
        conductor: ecad_domain::ConductorId,
        /// Explicit same-view A/B placed anchors.
        anchors: [Option<ecad_domain::AnchorId>; 2],
        /// Complete ordered document geometry.
        points: Vec<DrawingPoint>,
        /// Explicit geometry policy.
        mode: ecad_domain::RouteMode,
    },
    /// Invoke an immediate action or request an input/placement editor.
    #[default]
    None,
    /// Complete annotation data supplied by placement/numeric input.
    Label {
        /// Fresh annotation identity.
        id: LabelId,
        /// Nonblank literal text.
        text: String,
        /// Explicit document position, never a popup screen location.
        position: DrawingPoint,
    },
    /// Complete whole graphical-path data.
    Polyline {
        /// Fresh graphical-path identity.
        id: PolylineId,
        /// Ordered explicit document vertices.
        points: Vec<DrawingPoint>,
    },
    /// Reviewed complete portable drawing settings.
    DrawingSettings(DrawingSettings),
}
/// A single request shared by all action surfaces, with original targets and input identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionInvocation {
    /// Host input ID; each retry/reply gets a new ID.
    pub input_id: InputId,
    /// Stable registered action reference.
    pub action: ActionId,
    /// Source surface, independent of handler implementation.
    pub source: InvocationSource,
    /// Host auto-repeat flag, also checked for native accelerators bypassing key lookup.
    pub repeat: bool,
    /// Captured original document targets; revalidated against the current session.
    pub context: ActionContext,
    /// Concrete parameters or a request for them.
    pub parameters: ActionParameters,
}
/// A typed UI interaction request; actual native controls/tools remain host responsibilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionRequestKind {
    /// Native host should choose and open a project, protecting dirty state.
    OpenProject,
    /// Active drawing surface should fit its viewport without a document edit.
    FitDrawing,
    /// Request pivot and quarter-turn for the captured drawing selection.
    RotateSelection,
    /// Request pivot and reflection axis for the captured drawing selection.
    MirrorSelection,
    /// Request a signed drawing offset for the captured selection.
    MoveSelection,
    /// Request axis and exact numeric coordinate for origin alignment.
    AlignSelection,
    /// Request axis for origin distribution between selected extrema.
    DistributeSelection,
    /// Request an existing conductor, placed anchors, and whole route geometry.
    RoutePlacement,
    /// Request annotation text and/or explicit placement.
    LabelPlacement,
    /// Request whole-path geometry.
    PolylinePlacement,
    /// Open portable drawing settings for review/edit.
    DrawingSettings,
    /// Open the registry-generated searchable palette.
    Palette,
    /// Cancel host-owned transient placement without a document edit.
    CancelPlacement,
}
/// Input/placement context preserved across transient UI; replies revalidate it.
pub struct ActionRequest {
    /// Original action identity for a subsequent parameterized invocation.
    pub action: ActionId,
    /// Requested interaction, with no hidden business logic in the host surface.
    pub kind: ActionRequestKind,
    /// Original selection/view/optional anchor and observed revision.
    pub context: ActionContext,
    /// Current settings only for the settings editor request.
    pub settings: Option<DrawingSettings>,
}
/// Observable action outcome; presentation requests are separate from document edits.
pub enum ActionOutcome {
    /// Read-only current electrical report; presentation/export does not mutate the document.
    ConnectionReport(Box<ConnectionReport>),
    /// Exact transient geometry for alignment, distribution, rotation, mirroring, or movement.
    LayoutPreview(Box<LayoutPreview>),
    /// Exact transient geometry and ranked snap targets, with no document mutation.
    MovePreview(Box<MovePreview>),
    /// A command/history operation completed, indicating whether document state changed.
    Edited(bool),
    /// Guarded persistence completed; project history is unchanged.
    Saved,
    /// Read-only full union inspector projection.
    Properties(PropertySelection),
    /// Host must collect parameters or perform transient presentation interaction.
    Request(Box<ActionRequest>),
}
/// Invocation failed without bypassing normal model/history/save guarantees.
#[derive(Debug)]
pub enum ActionDispatchError<E> {
    /// Move reference, candidate selection, or original geometry preflight failure.
    MovePreview(MovePreviewError),
    /// Original layout preflight failure, with geometry/model context retained.
    Layout(LayoutError),
    /// The same or an older host input ID was already attempted.
    ReplayedInput(InputId),
    /// Text, IME, modal UI, or a control owns this keyboard input.
    ProtectedInput,
    /// This action does not permit auto-repeat, including native accelerator repeats.
    RepeatedInput,
    /// Shared availability rules rejected invocation.
    Disabled(ActionDisabled),
    /// Parameter variant does not match the handler.
    WrongParameters,
    /// Normal document command/history failure.
    Edit(EditError),
    /// Read-only projection/model validation failure.
    Model(ModelError),
    /// Normal guarded document-save failure with original source preserved.
    Store(E),
}
impl<E: fmt::Display> fmt::Display for ActionDispatchError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReplayedInput(id) => write!(
                f,
                "input {} was already attempted or is out of order",
                id.get()
            ),
            Self::RepeatedInput => f.write_str("this action ignores repeated key-downs"),
            Self::ProtectedInput => f.write_str("the focused control owns keyboard input"),
            Self::Disabled(reason) => write!(f, "action is unavailable: {reason:?}"),
            Self::WrongParameters => f.write_str("parameters do not match the action"),
            Self::MovePreview(error) => error.fmt(f),
            Self::Layout(error) => error.fmt(f),
            Self::Edit(error) => error.fmt(f),
            Self::Model(error) => error.fmt(f),
            Self::Store(error) => error.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for ActionDispatchError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MovePreview(error) => Some(error),
            Self::Layout(error) => Some(error),
            Self::Edit(error) => Some(error),
            Self::Model(error) => Some(error),
            Self::Store(error) => Some(error),
            _ => None,
        }
    }
}
