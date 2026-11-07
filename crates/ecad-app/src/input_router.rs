//! Deterministic single/chord/sequence routing with explicit focus and replay protection.

use crate::*;
use std::num::NonZeroU64;

/// One normalized key-down event from a shared host input stream.
#[derive(Clone, Debug)]
pub struct ActionKeyEvent {
    /// Monotonic host ID, reused for native/web duplicate deliveries.
    pub id: InputId,
    /// Logical key and actual modifier state.
    pub chord: ResolvedChord,
    /// Host auto-repeat flag; placement/save ignore repeats.
    pub repeat: bool,
}
/// Why routing suppressed an event rather than invoking an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputSuppression {
    /// Duplicate or older event from another delivery path.
    Replayed,
    /// Text/IME/modal/control input is protected.
    Protected,
    /// Action or sequence prefix does not permit repeated key-downs.
    Repeat,
    /// Clock input moved backward; pending input was cancelled.
    ClockReversed,
    /// Document/selection/view/focus/tool changed during an ordered sequence.
    ContextChanged,
    /// Escape cancelled a pending sequence, without also cancelling the active tool.
    SequenceCancelled,
}
/// Result for the host's one keyboard routing path.
pub enum KeyRoute {
    /// No registered action handled the key; unmatched continuations are not reprocessed.
    Unhandled,
    /// Event must not invoke an application action.
    Suppressed(InputSuppression),
    /// Show the ordered prefix and deadline without changing document state.
    Pending {
        /// Platform-normalized pending chords.
        sequence: Vec<ResolvedChord>,
        /// Caller-clock deadline in milliseconds.
        deadline_ms: u64,
    },
    /// Submit through the same action dispatcher as menu/toolbar invocations.
    Invoke(Box<ActionInvocation>),
}
struct PendingSequence {
    sequence: Vec<ResolvedChord>,
    context: ActionContext,
    deadline_ms: u64,
}
/// Stateful router. Time is caller-supplied monotonic milliseconds; no blocking timer runs.
pub struct ActionInputRouter {
    keymap: EffectiveKeymap,
    timeout_ms: NonZeroU64,
    pending: Option<PendingSequence>,
    last_input: Option<InputId>,
    last_time_ms: Option<u64>,
}
impl ActionInputRouter {
    /// Use a validated keymap and explicit nonzero ordered-sequence timeout.
    pub fn new(keymap: EffectiveKeymap, timeout_ms: NonZeroU64) -> Self {
        Self {
            keymap,
            timeout_ms,
            pending: None,
            last_input: None,
            last_time_ms: None,
        }
    }
    /// Replace only with a fully compiled map and cancel pending prefixes; replay protection stays.
    pub fn replace_keymap(&mut self, keymap: EffectiveKeymap) {
        self.keymap = keymap;
        self.pending = None;
    }
    /// Inspect the current pending prefix without IO or document mutation.
    pub fn pending(&self) -> Option<&[ResolvedChord]> {
        self.pending
            .as_ref()
            .map(|pending| pending.sequence.as_slice())
    }
    /// Explicitly cancel a pending sequence, for focus/tool/modal transitions.
    pub fn cancel_pending(&mut self) {
        self.pending = None;
    }
    /// Expire pending feedback when the host timer reaches its deadline, without a key event.
    /// Returns whether a prefix was cleared; no blocking timer or document edit runs.
    pub fn expire_pending(&mut self, now_ms: u64) -> bool {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| now_ms >= pending.deadline_ms)
        {
            self.pending = None;
            true
        } else {
            false
        }
    }
    /// Route one key-down. Expired prefixes are cleared before processing a fresh key;
    /// an invalid continuation is consumed as a failed sequence, never retried as a tool key.
    pub fn route(
        &mut self,
        registry: &ActionRegistry,
        event: ActionKeyEvent,
        context: &ActionContext,
        now_ms: u64,
    ) -> KeyRoute {
        if self.last_input.is_some_and(|previous| event.id <= previous) {
            return KeyRoute::Suppressed(InputSuppression::Replayed);
        }
        self.last_input = Some(event.id);
        if self.last_time_ms.is_some_and(|previous| now_ms < previous) {
            self.pending = None;
            return KeyRoute::Suppressed(InputSuppression::ClockReversed);
        }
        self.last_time_ms = Some(now_ms);
        if crate::shortcut::protected_focus(context) {
            self.pending = None;
            return KeyRoute::Suppressed(InputSuppression::Protected);
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| now_ms >= pending.deadline_ms)
        {
            self.pending = None;
        }
        if let Some(pending) = &self.pending {
            if !pending.context.same_sequence_target(context) {
                self.pending = None;
                return KeyRoute::Suppressed(InputSuppression::ContextChanged);
            }
            if event.chord.key.as_named() == Some(NamedKey::Escape)
                && event.chord.modifiers == PhysicalModifiers::default()
            {
                self.pending = None;
                return KeyRoute::Suppressed(InputSuppression::SequenceCancelled);
            }
            if event.repeat {
                return KeyRoute::Suppressed(InputSuppression::Repeat);
            }
        }
        let mut sequence = self
            .pending
            .as_ref()
            .map(|pending| pending.sequence.clone())
            .unwrap_or_default();
        sequence.push(event.chord);
        let captured = self
            .pending
            .as_ref()
            .map(|pending| pending.context.clone())
            .unwrap_or_else(|| context.clone());
        let candidates: Vec<_> = self
            .keymap
            .bindings()
            .iter()
            .filter(|binding| {
                binding.scope.active(&captured) && binding.sequence.starts_with(&sequence)
            })
            .collect();
        if let Some(binding) = candidates
            .iter()
            .find(|binding| binding.sequence == sequence)
        {
            let repeat = registry
                .get(&binding.action)
                .map(|action| action.repeat)
                .unwrap_or(ActionRepeat::Ignore);
            if event.repeat && repeat == ActionRepeat::Ignore {
                return KeyRoute::Suppressed(InputSuppression::Repeat);
            }
            let invocation = ActionInvocation {
                input_id: event.id,
                action: binding.action.clone(),
                source: InvocationSource::Keyboard,
                repeat: event.repeat,
                context: captured,
                parameters: ActionParameters::None,
            };
            self.pending = None;
            return KeyRoute::Invoke(Box::new(invocation));
        }
        if candidates.is_empty() {
            self.pending = None;
            return KeyRoute::Unhandled;
        }
        if event.repeat {
            return KeyRoute::Suppressed(InputSuppression::Repeat);
        }
        let deadline_ms = self
            .pending
            .as_ref()
            .map(|pending| pending.deadline_ms)
            .unwrap_or_else(|| now_ms.saturating_add(self.timeout_ms.get()));
        self.pending = Some(PendingSequence {
            sequence: sequence.clone(),
            context: captured,
            deadline_ms,
        });
        KeyRoute::Pending {
            sequence,
            deadline_ms,
        }
    }
}
