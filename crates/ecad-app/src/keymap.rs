//! Layer declarative per-action overrides and reject unresolved overlap/prefix ambiguity.

use crate::*;
use std::collections::{BTreeMap, BTreeSet};

/// An ordered shortcut in one input scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedShortcut {
    /// Context in which the sequence can start/continue.
    pub scope: BindingScope,
    /// Ordered simultaneous chords.
    pub sequence: KeySequence,
}
/// User overrides replace all shipped bindings for an action; an empty list unbinds it.
/// Unknown IDs are preserved here and diagnosed during compilation, never discarded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeymapOverrides {
    /// Stable action IDs to replacement binding lists, in canonical ID order.
    pub actions: BTreeMap<ActionId, Vec<ScopedShortcut>>,
}
/// One normalized effective binding, used by routing and shortcut displays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveBinding {
    /// Registered stable action ID.
    pub action: ActionId,
    /// Active input scope.
    pub scope: BindingScope,
    /// Platform-resolved logical key sequence.
    pub sequence: Vec<ResolvedChord>,
}
/// Import/compilation diagnostics retain original action references.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeymapIssue {
    /// Imported overrides name an unregistered/retired action.
    UnknownAction(ActionId),
    /// A binding widens the handler beyond its declared input context.
    InvalidScope(ActionId, BindingScope),
    /// Identical sequences overlap; even duplicate same-action entries are rejected.
    Duplicate(EffectiveBinding, EffectiveBinding),
    /// One sequence prefixes another in overlapping contexts; direct keys are not delayed.
    Prefix(EffectiveBinding, EffectiveBinding),
}
/// Fully validated effective bindings; failed compilation cannot partially install a map.
#[derive(Clone, Debug)]
pub struct EffectiveKeymap {
    platform: ShortcutPlatform,
    bindings: Vec<EffectiveBinding>,
}
impl EffectiveKeymap {
    /// Inspect all resolved bindings in deterministic action/supplied-binding order.
    pub fn bindings(&self) -> &[EffectiveBinding] {
        &self.bindings
    }
    /// Modifier convention used for normalization and user-visible shortcut text.
    pub fn platform(&self) -> ShortcutPlatform {
        self.platform
    }
    /// Derive menu/help labels from effective bindings; duplicate context labels collapse.
    pub fn shortcuts(&self, id: &ActionId) -> Vec<String> {
        self.bindings
            .iter()
            .filter(|binding| &binding.action == id)
            .map(|binding| {
                binding
                    .sequence
                    .iter()
                    .map(|chord| chord.display(self.platform))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
fn narrower(scope: BindingScope, requested: BindingScope) -> bool {
    match scope {
        BindingScope::Application => true,
        BindingScope::Document => requested != BindingScope::Application,
        BindingScope::Canvas => matches!(requested, BindingScope::Canvas | BindingScope::Tool),
        other => other == requested,
    }
}
/// Compile shipped defaults plus per-action user overrides for one platform.
/// # Errors
/// Reports all unknown IDs, widened scopes, duplicate chords, and overlapping prefixes.
/// Mutually exclusive scopes may reuse keys. No new effective map escapes on failure.
pub fn compile_keymap(
    registry: &ActionRegistry,
    overrides: &KeymapOverrides,
    platform: ShortcutPlatform,
) -> Result<EffectiveKeymap, Vec<KeymapIssue>> {
    let mut issues = Vec::new();
    for id in overrides.actions.keys() {
        if registry.get(id).is_none() {
            issues.push(KeymapIssue::UnknownAction(id.clone()));
        }
    }
    let mut bindings: Vec<EffectiveBinding> = Vec::new();
    for definition in registry.definitions() {
        let declared = overrides
            .actions
            .get(&definition.id)
            .map(Vec::as_slice)
            .unwrap_or_else(|| match &definition.defaults {
                DefaultBindings::Suggested(bindings) => bindings.as_slice(),
                DefaultBindings::Unbound => &[],
            });
        for binding in declared {
            if !narrower(definition.scope, binding.scope) {
                issues.push(KeymapIssue::InvalidScope(
                    definition.id.clone(),
                    binding.scope,
                ));
            }
            let effective = EffectiveBinding {
                action: definition.id.clone(),
                scope: binding.scope,
                sequence: binding.sequence.resolve(platform),
            };
            for previous in &bindings {
                if !previous.scope.overlaps(effective.scope) {
                    continue;
                }
                if previous.sequence == effective.sequence {
                    issues.push(KeymapIssue::Duplicate(previous.clone(), effective.clone()));
                } else if previous.sequence.starts_with(&effective.sequence)
                    || effective.sequence.starts_with(&previous.sequence)
                {
                    issues.push(KeymapIssue::Prefix(previous.clone(), effective.clone()));
                }
            }
            bindings.push(effective);
        }
    }
    if issues.is_empty() {
        Ok(EffectiveKeymap { platform, bindings })
    } else {
        Err(issues)
    }
}
