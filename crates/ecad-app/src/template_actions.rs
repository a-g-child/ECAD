//! Template actions share the registry/router/input stream while targeting an independent document.
use crate::*;
use ecad_domain::{TemplateRevisionId, TemplateVariantRef};
/// Operations available in the template workspace; property values remain command parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateOperation {
    /// Cancel a host-owned transient table gesture.
    CancelDrag,
    /// Structured table geometry/content review.
    Table(TableOperation),
    /// Explicitly refresh the observed library catalogue without replacing the draft.
    Reload,
    /// Create an unsaved definition.
    New,
    /// Edit a selected saved revision/variant.
    Open,
    /// Close a clean editor.
    Close,
    /// Explicitly discard its unsaved draft.
    Discard,
    /// Import the last successfully saved revision into the project, without adoption.
    Import,
    /// Apply a closed validated property edit.
    Apply,
    /// Revert uncommitted property controls.
    Revert,
}
/// Registered structured table operations; numeric values remain closed command parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableOperation {
    /// Insert a row across a retained table partition.
    InsertRow,
    /// Insert a column across a retained table partition.
    InsertColumn,
    /// Move/resize a complete coherent table.
    Resize,
    /// Split one cell into a row-major grid.
    Split,
    /// Merge a rectangular cell selection with explicit content handling.
    Merge,
    /// Move an aligned interior boundary.
    Boundary,
}
/// Result of a template action; host requests retain their explicit source when present.
pub enum TemplateActionOutcome {
    /// Template draft/history changed or publication completed.
    Changed,
    /// Host workspace operation.
    Request(TemplateOperation),
    /// Open this exact saved variant.
    Open(TemplateVariantRef),
    /// Fit the template paper surface.
    Fit,
    /// Focus the current template properties.
    Properties,
    /// Open registry-generated command search.
    Palette,
}
impl ActionRegistry {
    /// Shared availability revalidated against actual template history and captured revision.
    pub fn template_state(
        &self,
        id: &ActionId,
        context: &ActionContext,
        document: &TemplateDocument,
    ) -> ActionState {
        let mut state = self.state(id, context, None);
        if context.editor != Some(document.revision()) {
            state.disabled = Some(ActionDisabled::StaleContext);
            return state;
        }
        if state.disabled.is_some() {
            return state;
        }
        let handler = self.get(id).expect("registered action").handler;
        state.disabled = match handler {
            ActionHandler::CancelPlacement if !context.tool_active => {
                Some(ActionDisabled::NoActiveTool)
            }
            ActionHandler::OpenProject => Some(ActionDisabled::NoDocument),
            ActionHandler::Undo if document.undo_len() == 0 => Some(ActionDisabled::NothingToUndo),
            ActionHandler::Redo if document.redo_len() == 0 => Some(ActionDisabled::NothingToRedo),
            ActionHandler::Undo | ActionHandler::Redo
                if crate::shortcut::protected_focus(context) =>
            {
                Some(ActionDisabled::TextEditorOwnsUndo)
            }
            ActionHandler::Template(
                TemplateOperation::New
                | TemplateOperation::Open
                | TemplateOperation::Close
                | TemplateOperation::Import,
            ) if document.is_dirty() => Some(ActionDisabled::UnsavedTemplate),
            _ => None,
        };
        state
    }
}
impl ActionDispatcher {
    /// Dispatch a captured template invocation through the same monotonic action boundary.
    /// Project document state is never supplied to this operation.
    /// # Errors
    /// Rejects protected/replayed/stale/invalid requests and preserves original save failures.
    pub fn invoke_template<S: VersionedTemplateLibraryStore>(
        &mut self,
        registry: &ActionRegistry,
        invocation: ActionInvocation,
        document: &mut TemplateDocument,
        publisher: &mut TemplatePublisher<S>,
        save_revision: Option<TemplateRevisionId>,
    ) -> Result<TemplateActionOutcome, TemplateDispatchError<S::Error>> {
        self.record_input(invocation.input_id)
            .map_err(|_| TemplateDispatchError::Replayed)?;
        if matches!(
            invocation.source,
            InvocationSource::Keyboard | InvocationSource::NativeAccelerator
        ) && crate::shortcut::protected_focus(&invocation.context)
        {
            return Err(TemplateDispatchError::Protected);
        }
        let state = registry.template_state(&invocation.action, &invocation.context, document);
        if let Some(reason) = state.disabled {
            return Err(TemplateDispatchError::Disabled(reason));
        }
        let definition = registry
            .get(&invocation.action)
            .expect("registered availability");
        if invocation.repeat && definition.repeat == ActionRepeat::Ignore {
            return Err(TemplateDispatchError::Protected);
        }
        let revision = document.revision();
        use ActionHandler as H;
        use TemplateActionOutcome as O;
        match (definition.handler, invocation.parameters) {
            (H::Undo, ActionParameters::None) => {
                document
                    .undo(revision)
                    .map_err(TemplateDispatchError::Edit)?;
                Ok(O::Changed)
            }
            (H::Redo, ActionParameters::None) => {
                document
                    .redo(revision)
                    .map_err(TemplateDispatchError::Edit)?;
                Ok(O::Changed)
            }
            (H::Save, ActionParameters::None) => {
                let id = save_revision.ok_or(TemplateDispatchError::Parameters)?;
                document
                    .save(revision, id, publisher)
                    .map_err(TemplateDispatchError::Save)?;
                Ok(O::Changed)
            }
            (
                H::Template(TemplateOperation::Apply | TemplateOperation::Table(_)),
                ActionParameters::TemplateEdit {
                    variant,
                    name,
                    layout,
                },
            ) => {
                document
                    .replace_variant(revision, variant, name, *layout)
                    .map_err(TemplateDispatchError::Edit)?;
                Ok(O::Changed)
            }
            (H::Template(TemplateOperation::Open), ActionParameters::TemplateSource(pin)) => {
                Ok(O::Open(pin))
            }
            (H::Template(TemplateOperation::Apply), ActionParameters::None) => {
                Ok(O::Request(TemplateOperation::Apply))
            }
            (H::Template(TemplateOperation::Open), ActionParameters::None) => {
                Ok(O::Request(TemplateOperation::Open))
            }
            (H::Template(op), ActionParameters::None)
                if !matches!(op, TemplateOperation::Open | TemplateOperation::Apply) =>
            {
                Ok(O::Request(op))
            }
            (H::CancelPlacement, ActionParameters::None) => {
                Ok(O::Request(TemplateOperation::CancelDrag))
            }
            (H::FitDrawing, ActionParameters::None) => Ok(O::Fit),
            (H::Properties, ActionParameters::None) => Ok(O::Properties),
            (H::Palette, ActionParameters::None) => Ok(O::Palette),
            (H::TemplateLibrary, ActionParameters::None) => Ok(O::Changed),
            _ => Err(TemplateDispatchError::Parameters),
        }
    }
}
/// Typed template action failure; failed attempts retain document state and consume input IDs.
#[derive(Debug)]
pub enum TemplateDispatchError<E> {
    /// The same/older input was already attempted.
    Replayed,
    /// Text/composition/control/repeat owns this input.
    Protected,
    /// Registry availability rejected the invocation.
    Disabled(ActionDisabled),
    /// Parameters do not match the registered handler.
    Parameters,
    /// Draft/history validation failed.
    Edit(TemplateEditError),
    /// Durable publication failed.
    Save(TemplateDocumentError<E>),
}
impl<E: std::fmt::Display> std::fmt::Display for TemplateDispatchError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Replayed => f.write_str("Input was already handled"),
            Self::Protected => f.write_str("The focused control owns this input"),
            Self::Disabled(e) => write!(f, "Template action unavailable: {e:?}"),
            Self::Parameters => f.write_str("Template action parameters do not match"),
            Self::Edit(e) => e.fmt(f),
            Self::Save(e) => e.fmt(f),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for TemplateDispatchError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Edit(e) => Some(e),
            Self::Save(e) => Some(e),
            _ => None,
        }
    }
}
