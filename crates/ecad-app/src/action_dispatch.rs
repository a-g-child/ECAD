//! One invocation boundary for every UI surface; commands retain model/history authority.

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
            Self::Edit(error) => error.fmt(f),
            Self::Model(error) => error.fmt(f),
            Self::Store(error) => error.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for ActionDispatchError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Model(error) => Some(error),
            Self::Store(error) => Some(error),
            _ => None,
        }
    }
}
/// Serial action boundary; share one instance across native/web/menu surfaces.
/// A host must provide one monotonic ID stream, reusing an ID for duplicate deliveries.
/// Failed attempts also consume IDs so delayed callbacks cannot replay after state changes.
#[derive(Default)]
pub struct ActionDispatcher {
    last_input: Option<InputId>,
}
impl ActionDispatcher {
    /// Revalidate and execute one invocation using actual commands and document lifecycle.
    /// # Errors
    /// Reports replay/protected/disabled/parameter failures or original model/save errors.
    pub fn invoke<S: VersionedProjectStore>(
        &mut self,
        registry: &ActionRegistry,
        invocation: ActionInvocation,
        mut document: Option<&mut Document<S>>,
    ) -> Result<ActionOutcome, ActionDispatchError<S::Error>> {
        if self
            .last_input
            .is_some_and(|previous| invocation.input_id <= previous)
        {
            return Err(ActionDispatchError::ReplayedInput(invocation.input_id));
        }
        self.last_input = Some(invocation.input_id);
        if matches!(
            invocation.source,
            InvocationSource::Keyboard | InvocationSource::NativeAccelerator
        ) && crate::shortcut::protected_focus(&invocation.context)
        {
            return Err(ActionDispatchError::ProtectedInput);
        }
        let state = registry.state(
            &invocation.action,
            &invocation.context,
            document.as_ref().map(|document| document.session()),
        );
        if let Some(reason) = state.disabled {
            return Err(ActionDispatchError::Disabled(reason));
        }
        let definition = registry
            .get(&invocation.action)
            .expect("availability resolved registered action");
        if invocation.repeat && definition.repeat == ActionRepeat::Ignore {
            return Err(ActionDispatchError::RepeatedInput);
        }
        let handler = definition.handler;
        let request = |kind, settings| {
            ActionOutcome::Request(Box::new(ActionRequest {
                action: invocation.action.clone(),
                kind,
                context: invocation.context.clone(),
                settings,
            }))
        };
        match (handler, &invocation.parameters) {
            (ActionHandler::Palette, ActionParameters::None) => {
                return Ok(request(ActionRequestKind::Palette, None));
            }
            (ActionHandler::CancelPlacement, ActionParameters::None) => {
                return Ok(request(ActionRequestKind::CancelPlacement, None));
            }
            (ActionHandler::Palette | ActionHandler::CancelPlacement, _) => {
                return Err(ActionDispatchError::WrongParameters);
            }
            _ => {}
        }
        let document = document
            .as_mut()
            .ok_or(ActionDispatchError::Disabled(ActionDisabled::NoDocument))?;
        let revision = invocation
            .context
            .revision
            .expect("document availability checked revision");
        let outcome = match (handler, invocation.parameters) {
            (ActionHandler::Undo, ActionParameters::None) => {
                ActionOutcome::Edited(document.undo(revision).map_err(ActionDispatchError::Edit)?)
            }
            (ActionHandler::Redo, ActionParameters::None) => {
                ActionOutcome::Edited(document.redo(revision).map_err(ActionDispatchError::Edit)?)
            }
            (ActionHandler::Save, ActionParameters::None) => {
                document.save().map_err(ActionDispatchError::Store)?;
                ActionOutcome::Saved
            }
            (ActionHandler::Properties, ActionParameters::None) => ActionOutcome::Properties(
                inspect_properties(
                    document.session().project(),
                    invocation.context.selection.iter().copied(),
                )
                .map_err(ActionDispatchError::Model)?,
            ),
            (ActionHandler::CreateLabel, ActionParameters::None) => {
                request(ActionRequestKind::LabelPlacement, None)
            }
            (ActionHandler::CreatePolyline, ActionParameters::None) => {
                request(ActionRequestKind::PolylinePlacement, None)
            }
            (ActionHandler::DrawingSettings, ActionParameters::None) => request(
                ActionRequestKind::DrawingSettings,
                Some(document.session().project().drawing().settings().clone()),
            ),
            (ActionHandler::CreateLabel, ActionParameters::Label { id, text, position }) => {
                ActionOutcome::Edited(
                    document
                        .execute(
                            revision,
                            Command::Drawing(DrawingCommand::CreateLabel {
                                id,
                                text,
                                position,
                                view: invocation
                                    .context
                                    .view
                                    .expect("placement availability checked view"),
                            }),
                        )
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            (ActionHandler::CreatePolyline, ActionParameters::Polyline { id, points }) => {
                ActionOutcome::Edited(
                    document
                        .execute(
                            revision,
                            Command::Drawing(DrawingCommand::CreatePolyline {
                                id,
                                points,
                                view: invocation
                                    .context
                                    .view
                                    .expect("placement availability checked view"),
                            }),
                        )
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            (ActionHandler::DrawingSettings, ActionParameters::DrawingSettings(settings)) => {
                ActionOutcome::Edited(
                    document
                        .execute(
                            revision,
                            Command::Drawing(DrawingCommand::Settings(settings)),
                        )
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            _ => return Err(ActionDispatchError::WrongParameters),
        };
        Ok(outcome)
    }
}
