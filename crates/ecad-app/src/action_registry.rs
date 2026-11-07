//! One registry for metadata, availability, palette queries, and typed feature handlers.

use crate::*;
use std::collections::BTreeMap;

/// Implemented action handler families. Feature business logic remains in commands/services.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionHandler {
    /// Undo one document transaction.
    Undo,
    /// Redo one document transaction.
    Redo,
    /// Guarded save through the bound document controller.
    Save,
    /// Inspect the current existing selection.
    Properties,
    /// Request annotation input/placement or create from supplied parameters.
    CreateLabel,
    /// Request graphical polyline input/placement or create from supplied parameters.
    CreatePolyline,
    /// Inspect/edit portable drawing settings.
    DrawingSettings,
    /// Request the registry-generated command palette.
    Palette,
    /// Request cancellation of host-owned transient placement.
    CancelPlacement,
}
/// Declared input needs; missing creation input produces a typed request, not an invented point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionInputRequirement {
    /// No additional parameters.
    None,
    /// Existing selected objects are required.
    Selection,
    /// Annotation text/identity and a definite position are required to commit.
    LabelPlacement,
    /// Whole graphical-path identity/vertices are required to commit.
    PolylinePlacement,
    /// Settings can be supplied or requested through an editor.
    DrawingSettings,
}
/// Host key-repeat policy, separate from command undo grouping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionRepeat {
    /// One dispatch per deliberate key press.
    Ignore,
    /// Each repeated event may invoke one operation (used for undo/redo).
    Allow,
}
/// Explicit shipped binding policy. Unbound actions remain assignable and searchable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefaultBindings {
    /// Shipped portable candidates; user overrides replace all candidates for this action.
    Suggested(Vec<ScopedShortcut>),
    /// Deliberately unbound by default, with palette/menu access still available.
    Unbound,
}
/// One registered operation, independent of UI surface and translated key labels.
#[derive(Clone, Debug)]
pub struct ActionDefinition {
    /// Stable namespaced ID.
    pub id: ActionId,
    /// Display name.
    pub label: String,
    /// Display category.
    pub category: String,
    /// Purpose and relevant behavior.
    pub description: String,
    /// Additional discoverability terms.
    pub search_terms: Vec<String>,
    /// Broadest permitted binding scope; overrides may narrow it.
    pub scope: BindingScope,
    /// Implemented handler family.
    pub handler: ActionHandler,
    /// Explicit key-repeat policy.
    pub repeat: ActionRepeat,
    /// Shipped bindings or a deliberate unbound policy.
    pub defaults: DefaultBindings,
}
impl ActionDefinition {
    /// Derive invocation needs from the actual typed handler, avoiding divergent metadata.
    pub fn input_requirement(&self) -> ActionInputRequirement {
        match self.handler {
            ActionHandler::Properties => ActionInputRequirement::Selection,
            ActionHandler::CreateLabel => ActionInputRequirement::LabelPlacement,
            ActionHandler::CreatePolyline => ActionInputRequirement::PolylinePlacement,
            ActionHandler::DrawingSettings => ActionInputRequirement::DrawingSettings,
            _ => ActionInputRequirement::None,
        }
    }
}
/// Current eligibility; disabled actions remain visible and searchable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionState {
    /// Concrete reason the action cannot currently run; `None` means enabled.
    pub disabled: Option<ActionDisabled>,
    /// Optional toggle indication. Current built-ins are ordinary non-toggle actions.
    pub checked: Option<bool>,
}
impl ActionState {
    /// Whether a fresh invocation may proceed to parameter validation/execution.
    pub fn enabled(&self) -> bool {
        self.disabled.is_none()
    }
}
/// Registry-backed palette/menu listing, including unavailable actions and reasons.
pub struct ActionListing<'a> {
    /// Shared metadata; no duplicate action implementation is stored here.
    pub definition: &'a ActionDefinition,
    /// State computed against the same captured context as invocation.
    pub state: ActionState,
}
/// Dynamically sized central metadata registry, with no arbitrary action-count limit.
#[derive(Default)]
pub struct ActionRegistry {
    definitions: BTreeMap<ActionId, ActionDefinition>,
}
impl ActionRegistry {
    /// Register validated metadata; duplicate IDs never silently replace a handler.
    /// # Errors
    /// Rejects duplicate IDs and blank required/search metadata without mutation.
    pub fn register(&mut self, definition: ActionDefinition) -> Result<(), ActionConfigError> {
        if self.definitions.contains_key(&definition.id) {
            return Err(ActionConfigError::DuplicateAction(definition.id));
        }
        if [
            &definition.label,
            &definition.category,
            &definition.description,
        ]
        .iter()
        .any(|value| value.trim().is_empty())
            || definition
                .search_terms
                .iter()
                .any(|term| term.trim().is_empty())
        {
            return Err(ActionConfigError::EmptyMetadata);
        }
        if matches!(&definition.defaults, DefaultBindings::Suggested(bindings) if bindings.is_empty())
        {
            return Err(ActionConfigError::EmptyDefaultBindings);
        }
        self.definitions.insert(definition.id.clone(), definition);
        Ok(())
    }
    /// Inspect a stable action reference.
    pub fn get(&self, id: &ActionId) -> Option<&ActionDefinition> {
        self.definitions.get(id)
    }
    /// Iterate all actions in stable ID order, including deliberately unbound actions.
    pub fn definitions(&self) -> impl ExactSizeIterator<Item = &ActionDefinition> {
        self.definitions.values()
    }
    /// Compute shared availability without changing the document or requiring storage IO.
    pub fn state(
        &self,
        id: &ActionId,
        context: &ActionContext,
        session: Option<&Session>,
    ) -> ActionState {
        ActionState {
            disabled: self.disabled(id, context, session),
            checked: None,
        }
    }
    fn disabled(
        &self,
        id: &ActionId,
        context: &ActionContext,
        session: Option<&Session>,
    ) -> Option<ActionDisabled> {
        let Some(definition) = self.get(id) else {
            return Some(ActionDisabled::UnknownAction(id.clone()));
        };
        let handler = definition.handler;
        if handler == ActionHandler::Palette {
            return None;
        }
        if handler == ActionHandler::CancelPlacement {
            return if context.tool_active {
                None
            } else {
                Some(ActionDisabled::NoActiveTool)
            };
        }
        let Some(session) = session else {
            return Some(ActionDisabled::NoDocument);
        };
        if matches!(handler, ActionHandler::Undo | ActionHandler::Redo)
            && crate::shortcut::protected_focus(context)
        {
            return Some(ActionDisabled::TextEditorOwnsUndo);
        }
        if context.revision != Some(session.revision()) {
            return Some(ActionDisabled::StaleContext);
        }
        match handler {
            ActionHandler::Undo if session.undo_len() == 0 => Some(ActionDisabled::NothingToUndo),
            ActionHandler::Redo if session.redo_len() == 0 => Some(ActionDisabled::NothingToRedo),
            ActionHandler::Properties => {
                if context.selection.is_empty() {
                    return Some(ActionDisabled::NoSelection);
                }
                context
                    .selection
                    .iter()
                    .find(|target| session.project().require_object(**target).is_err())
                    .copied()
                    .map(ActionDisabled::MissingTarget)
            }
            ActionHandler::CreateLabel | ActionHandler::CreatePolyline => {
                if context.focus != ActionFocus::Canvas {
                    return Some(ActionDisabled::CanvasRequired);
                }
                if context
                    .view
                    .is_none_or(|view| session.project().drawing().view(view).is_none())
                {
                    return Some(ActionDisabled::NoDrawingView);
                }
                None
            }
            _ => None,
        }
    }
    /// Search metadata using all whitespace-separated case-insensitive terms; include
    /// disabled actions. Rank label prefix, label match, then other metadata, with ID ties.
    pub fn search(
        &self,
        query: &str,
        context: &ActionContext,
        session: Option<&Session>,
    ) -> Vec<ActionListing<'_>> {
        let query = query.trim().to_lowercase();
        let mut matches = Vec::new();
        for definition in self.definitions.values() {
            let label = definition.label.to_lowercase();
            let haystack = format!(
                "{} {} {} {} {}",
                definition.id,
                label,
                definition.category,
                definition.description,
                definition.search_terms.join(" ")
            )
            .to_lowercase();
            if !query.split_whitespace().all(|term| haystack.contains(term)) {
                continue;
            }
            let rank = if label.starts_with(&query) {
                0
            } else if label.contains(&query) {
                1
            } else {
                2
            };
            matches.push((rank, definition));
        }
        matches
            .sort_by(|(a_rank, a), (b_rank, b)| a_rank.cmp(b_rank).then_with(|| a.id.cmp(&b.id)));
        matches
            .into_iter()
            .map(|(_, definition)| ActionListing {
                definition,
                state: self.state(&definition.id, context, session),
            })
            .collect()
    }
}
