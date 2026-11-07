//! Portable declarative overrides, canonical export, and explicit import diagnostics.

use ecad_adapters::{KeymapFileError, decode_keymap, encode_keymap};
use ecad_app::*;
use serde_json::{Value, json};

const FIXTURE: &str = include_str!("fixtures/keymap-v1.json");
fn id(value: &str) -> ActionId {
    ActionId::new(value).unwrap()
}

#[test]
fn portable_unicode_function_keys_and_explicit_unbinding_roundtrip_canonically() {
    let registry = ActionRegistry::builtins();
    let overrides = decode_keymap(FIXTURE).unwrap();
    assert_eq!(encode_keymap(&overrides).unwrap(), FIXTURE);
    assert!(overrides.actions[&id("document.redo")].is_empty());
    let mac = compile_keymap(&registry, &overrides, ShortcutPlatform::MacOS).unwrap();
    assert_eq!(mac.shortcuts(&id("drawing.styles.edit")), ["Cmd+F12"]);
    assert_eq!(mac.shortcuts(&id("drawing.label.create")), ["λ"]);
    assert!(mac.shortcuts(&id("document.redo")).is_empty());
    let windows = compile_keymap(&registry, &overrides, ShortcutPlatform::Windows).unwrap();
    assert_eq!(windows.shortcuts(&id("drawing.styles.edit")), ["Ctrl+F12"]);
    let mut shuffled: Value = serde_json::from_str(FIXTURE).unwrap();
    shuffled["overrides"].as_array_mut().unwrap().reverse();
    assert_eq!(
        encode_keymap(&decode_keymap(&shuffled.to_string()).unwrap()).unwrap(),
        FIXTURE
    );
}

#[test]
fn unknown_actions_survive_import_export_and_fail_compilation_as_visible_diagnostics() {
    let mut data: Value = serde_json::from_str(FIXTURE).unwrap();
    data["overrides"]
        .as_array_mut()
        .unwrap()
        .push(json!({"action":"retired.operation","bindings":[]}));
    let imported = decode_keymap(&data.to_string()).unwrap();
    assert!(imported.actions.contains_key(&id("retired.operation")));
    let registry = ActionRegistry::builtins();
    assert!(
        compile_keymap(&registry, &imported, ShortcutPlatform::Linux)
            .unwrap_err()
            .iter()
            .any(|issue| *issue == KeymapIssue::UnknownAction(id("retired.operation")))
    );
    assert_eq!(
        decode_keymap(&encode_keymap(&imported).unwrap()).unwrap(),
        imported
    );
}

#[test]
fn strict_import_rejects_unknown_fields_duplicates_empty_sequences_and_invalid_keys_modifiers() {
    let data: Value = serde_json::from_str(FIXTURE).unwrap();
    for pointer in [
        "",
        "/overrides/1",
        "/overrides/1/bindings/0",
        "/overrides/1/bindings/0/sequence/0",
        "/overrides/1/bindings/0/sequence/0/key",
    ] {
        let mut invalid = data.clone();
        invalid.pointer_mut(pointer).unwrap()["unexpected"] = json!(true);
        assert!(
            decode_keymap(&invalid.to_string()).is_err(),
            "accepted {pointer}"
        );
    }
    for (pointer, value) in [
        ("/kind", json!("other-kind")),
        ("/overrides/1/action", json!("Drawing.label")),
        ("/overrides/1/bindings/0/scope", json!("unknown_scope")),
        ("/overrides/1/bindings/0/sequence", json!([])),
        ("/overrides/1/bindings/0/sequence/0/key/value", json!("ab")),
        ("/overrides/1/bindings/0/sequence/0/key/value", json!(" ")),
        (
            "/overrides/1/bindings/0/sequence/0/modifiers",
            json!(["primary", "primary"]),
        ),
        (
            "/overrides/1/bindings/0/sequence/0/modifiers",
            json!(["executable"]),
        ),
        ("/overrides/2/bindings/0/sequence/0/key/value", json!(0)),
        ("/overrides/2/bindings/0/sequence/0/key/value", json!(25)),
    ] {
        let mut invalid = data.clone();
        *invalid.pointer_mut(pointer).unwrap() = value;
        assert!(
            decode_keymap(&invalid.to_string()).is_err(),
            "accepted {pointer}"
        );
    }
    let mut repeated = data.clone();
    let entry = repeated["overrides"][0].clone();
    repeated["overrides"].as_array_mut().unwrap().push(entry);
    assert!(matches!(
        decode_keymap(&repeated.to_string()),
        Err(KeymapFileError::Configuration(
            ActionConfigError::DuplicateOverride(_)
        ))
    ));
    let mut future = data;
    future["schema_version"] = json!(2);
    assert!(matches!(
        decode_keymap(&future.to_string()),
        Err(KeymapFileError::UnsupportedVersion(2))
    ));
}

#[test]
fn conflicting_imports_never_partially_replace_the_previously_valid_effective_keymap() {
    let registry = ActionRegistry::builtins();
    let original = compile_keymap(
        &registry,
        &KeymapOverrides::default(),
        ShortcutPlatform::Linux,
    )
    .unwrap();
    let mut data: Value = serde_json::from_str(FIXTURE).unwrap();
    data["overrides"].as_array_mut().unwrap().push(json!({"action":"drawing.polyline.create","bindings":[{"scope":"canvas","sequence":[{"key":{"kind":"character","value":"λ"},"modifiers":[]}]}]}));
    let imported = decode_keymap(&data.to_string()).unwrap();
    let before = imported.clone();
    assert!(
        compile_keymap(&registry, &imported, ShortcutPlatform::Linux)
            .unwrap_err()
            .iter()
            .any(|issue| matches!(issue, KeymapIssue::Duplicate(_, _)))
    );
    assert_eq!(imported, before);
    assert_eq!(original.shortcuts(&id("drawing.label.create")), ["T"]);
    assert_eq!(original.shortcuts(&id("drawing.polyline.create")), ["P"]);
}
