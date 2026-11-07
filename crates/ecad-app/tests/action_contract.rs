//! Registry, effective-keymap, and routing contracts runnable without native UI.

#[path = "actions/router.rs"]
mod routing;
#[path = "actions/support.rs"]
mod support;
use ecad_app::*;
use ecad_domain::*;
use support::*;

#[test]
fn builtin_metadata_is_complete_and_disabled_actions_stay_searchable() {
    let registry = ActionRegistry::builtins();
    assert_eq!(registry.definitions().len(), 9);
    let session = session();
    let context = context(&session);
    assert_eq!(
        registry
            .state(&id("document.undo"), &context, Some(&session))
            .disabled,
        Some(ActionDisabled::NothingToUndo)
    );
    assert_eq!(
        registry
            .state(&id("document.redo"), &context, Some(&session))
            .disabled,
        Some(ActionDisabled::NothingToRedo)
    );
    assert_eq!(
        registry
            .state(&id("selection.properties"), &context, Some(&session))
            .disabled,
        Some(ActionDisabled::NoSelection)
    );
    let results = registry.search("unDO", &context, Some(&session));
    assert!(!results.is_empty());
    assert!(!results[0].state.enabled());
    assert_eq!(results[0].definition.id, id("document.undo"));
    assert!(registry.search("wire", &context, Some(&session)).is_empty());
    assert_eq!(registry.search("", &context, Some(&session)).len(), 9);
    for action in registry.definitions() {
        assert!(!action.label.is_empty());
        assert!(!action.description.is_empty());
        let _ = action.input_requirement();
    }
    assert!(
        registry
            .state(&id("workspace.palette"), &ActionContext::default(), None)
            .enabled()
    );
    assert_eq!(
        registry
            .state(&id("document.save"), &ActionContext::default(), None)
            .disabled,
        Some(ActionDisabled::NoDocument)
    );
}

#[test]
fn registry_rejects_duplicate_or_invalid_metadata_and_grows_without_an_action_count_cap() {
    let mut registry = ActionRegistry::builtins();
    let template = registry.get(&id("workspace.palette")).unwrap().clone();
    assert!(matches!(
        registry.register(template.clone()),
        Err(ActionConfigError::DuplicateAction(_))
    ));
    let mut invalid = template.clone();
    invalid.id = id("extension.invalid");
    invalid.description.clear();
    assert_eq!(
        registry.register(invalid),
        Err(ActionConfigError::EmptyMetadata)
    );
    let mut invalid_policy = template.clone();
    invalid_policy.id = id("extension.invalid_policy");
    invalid_policy.defaults = DefaultBindings::Suggested(vec![]);
    assert_eq!(
        registry.register(invalid_policy),
        Err(ActionConfigError::EmptyDefaultBindings)
    );
    for number in 0..128 {
        let mut extra = template.clone();
        extra.id = id(&format!("extension.action_{number}"));
        extra.defaults = DefaultBindings::Unbound;
        registry.register(extra).unwrap();
    }
    assert_eq!(registry.definitions().len(), 137);
    for invalid in [
        "",
        "undo",
        "Document.undo",
        "document..undo",
        "document.1undo",
        "document.bad-name",
    ] {
        assert!(ActionId::new(invalid).is_err());
    }
    for platform in [
        ShortcutPlatform::Linux,
        ShortcutPlatform::Windows,
        ShortcutPlatform::MacOS,
    ] {
        assert!(compile_keymap(&registry, &KeymapOverrides::default(), platform).is_ok());
    }
}

#[test]
fn context_revalidation_uses_actual_history_selection_view_and_session_identity() {
    let registry = ActionRegistry::builtins();
    let mut session = session();
    let old = context(&session);
    session
        .execute(
            session.revision(),
            Command::InsertDevice(Device::new(DeviceId::new(1).unwrap(), "-X1", []).unwrap()),
        )
        .unwrap();
    assert_eq!(
        registry
            .state(&id("document.undo"), &old, Some(&session))
            .disabled,
        Some(ActionDisabled::StaleContext)
    );
    let mut current = context(&session);
    assert!(
        registry
            .state(&id("document.undo"), &current, Some(&session))
            .enabled()
    );
    current
        .selection
        .insert(ObjectRef::Device(DeviceId::new(999).unwrap()));
    assert!(matches!(
        registry
            .state(&id("selection.properties"), &current, Some(&session))
            .disabled,
        Some(ActionDisabled::MissingTarget(_))
    ));
    current.view = None;
    assert_eq!(
        registry
            .state(&id("drawing.label.create"), &current, Some(&session))
            .disabled,
        Some(ActionDisabled::NoDrawingView)
    );
    current.focus = ActionFocus::TextInput;
    assert_eq!(
        registry
            .state(&id("document.undo"), &current, Some(&session))
            .disabled,
        Some(ActionDisabled::TextEditorOwnsUndo)
    );
    let other = support::session();
    assert_eq!(
        registry
            .state(&id("document.save"), &context(&other), Some(&session))
            .disabled,
        Some(ActionDisabled::StaleContext)
    );
}

#[test]
fn unbinding_and_remapping_change_effective_help_without_removing_actions() {
    let registry = ActionRegistry::builtins();
    let mut overrides = KeymapOverrides::default();
    overrides.actions.insert(
        id("drawing.label.create"),
        vec![binding(
            BindingScope::Canvas,
            vec![chord('a', false, false)],
        )],
    );
    overrides
        .actions
        .insert(id("drawing.polyline.create"), vec![]);
    let map = compile_keymap(&registry, &overrides, ShortcutPlatform::MacOS).unwrap();
    assert_eq!(map.shortcuts(&id("drawing.label.create")), ["A"]);
    assert!(map.shortcuts(&id("drawing.polyline.create")).is_empty());
    assert_eq!(map.shortcuts(&id("document.save")), ["Cmd+S"]);
    assert_eq!(map.shortcuts(&id("drawing.styles.edit")), ["Cmd+K, S"]);
    assert!(registry.get(&id("drawing.polyline.create")).is_some());
    let linux = compile_keymap(&registry, &overrides, ShortcutPlatform::Linux).unwrap();
    assert_eq!(linux.shortcuts(&id("document.save")), ["Ctrl+S"]);
}

#[test]
fn overlapping_duplicates_and_prefixes_fail_but_exclusive_contexts_can_share_keys() {
    let registry = ActionRegistry::builtins();
    let mut overrides = KeymapOverrides::default();
    overrides.actions.insert(
        id("drawing.polyline.create"),
        vec![binding(
            BindingScope::Canvas,
            vec![chord('t', false, false)],
        )],
    );
    assert!(
        compile_keymap(&registry, &overrides, ShortcutPlatform::Linux)
            .unwrap_err()
            .iter()
            .any(|issue| matches!(issue, KeymapIssue::Duplicate(_, _)))
    );
    overrides.actions.insert(
        id("drawing.polyline.create"),
        vec![binding(
            BindingScope::Canvas,
            vec![chord('t', false, false), chord('a', false, false)],
        )],
    );
    assert!(
        compile_keymap(&registry, &overrides, ShortcutPlatform::Linux)
            .unwrap_err()
            .iter()
            .any(|issue| matches!(issue, KeymapIssue::Prefix(_, _)))
    );
    overrides.actions.clear();
    overrides.actions.insert(
        id("document.save"),
        vec![binding(
            BindingScope::Document,
            vec![chord('t', false, false)],
        )],
    );
    assert!(compile_keymap(&registry, &overrides, ShortcutPlatform::Linux).is_err());
    overrides.actions.clear();
    overrides.actions.insert(
        id("selection.properties"),
        vec![binding(
            BindingScope::ProjectTree,
            vec![chord('t', false, false)],
        )],
    );
    assert!(compile_keymap(&registry, &overrides, ShortcutPlatform::Linux).is_ok());
}

#[test]
fn unknown_ids_invalid_scopes_and_platform_alias_conflicts_are_reported_without_mutating_input() {
    let registry = ActionRegistry::builtins();
    let mut overrides = KeymapOverrides::default();
    overrides.actions.insert(id("retired.operation"), vec![]);
    overrides.actions.insert(
        id("drawing.label.create"),
        vec![binding(
            BindingScope::Application,
            vec![chord('q', false, false)],
        )],
    );
    let before = overrides.clone();
    let issues = compile_keymap(&registry, &overrides, ShortcutPlatform::Linux).unwrap_err();
    assert!(
        issues
            .iter()
            .any(|issue| matches!(issue, KeymapIssue::UnknownAction(_)))
    );
    assert!(
        issues
            .iter()
            .any(|issue| matches!(issue, KeymapIssue::InvalidScope(_, _)))
    );
    assert_eq!(overrides, before);
    overrides.actions.clear();
    let mut control_z = chord('Z', false, false);
    control_z.modifiers.control = true;
    overrides.actions.insert(
        id("drawing.label.create"),
        vec![binding(BindingScope::Canvas, vec![control_z])],
    );
    assert!(compile_keymap(&registry, &overrides, ShortcutPlatform::Linux).is_err());
    assert!(compile_keymap(&registry, &overrides, ShortcutPlatform::MacOS).is_ok());
    assert_eq!(
        ShortcutKey::character('T').unwrap(),
        ShortcutKey::character('t').unwrap()
    );
    assert!(ShortcutKey::character(' ').is_err());
    assert_eq!(ShortcutKey::function(24).unwrap().to_string(), "F24");
    assert!(ShortcutKey::function(0).is_err());
    assert!(ShortcutKey::function(25).is_err());
}
