//! Project default edits preserve inheritance intent, presets, checkpoints and exact history.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn call(b: &mut DesktopBridge, value: Value) -> Result<Value, ecad_bridge::BridgeError> {
    b.handle(serde_json::from_value::<Request>(value).unwrap())
        .map(|r| serde_json::to_value(r).unwrap())
}
fn snap(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn ctx(b: &DesktopBridge) -> Value {
    json!({"revision":snap(b)["revision"],"selection":[],"view":format!("{:032x}",1),"focus":"canvas"})
}
fn action(b: &mut DesktopBridge, action: &str, input: &str) -> Value {
    let context = ctx(b);
    call(
        b,
        json!({"type":"action","context":context,"action":action,"input":input}),
    )
    .unwrap()
}
fn defaults(b: &mut DesktopBridge, input: &str) -> Value {
    action(b, "drawing.styles.edit", input)["defaults"].clone()
}
fn update(
    b: &mut DesktopBridge,
    values: Value,
    input: &str,
) -> Result<Value, ecad_bridge::BridgeError> {
    let context = ctx(b);
    call(
        b,
        json!({"type":"update_defaults","context":context,"defaults":values,"input":input}),
    )
}
#[test]
fn fallback_edits_preserve_explicit_geometry_and_appearance_with_saved_history() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("defaults.json");
    let mut b = DesktopBridge::demo().unwrap();
    b.save_as(&path).unwrap();
    let original: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let before = snap(&b);
    let initial = defaults(&mut b, "1");
    assert_eq!(initial["text_height"], "2500");
    assert_eq!(snap(&b)["revision"], before["revision"]);
    let changed = update(&mut b,json!({"font":"Example\nFamily","text_height":"4500","stroke_width":"1250","colour":"#abCD12"}),"2").unwrap()["snapshot"].clone();
    let label = changed["scene"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["shape"] == "text")
        .unwrap();
    assert_eq!(label["font"], "Example\nFamily");
    assert_eq!(label["text_height"], 3.5); // Explicit object height overrides the project value.
    assert_eq!(label["colour"], "#abcd12");
    for route in changed["scene"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["target"]["kind"] == "route")
    {
        assert!(before["scene"].as_array().unwrap().contains(route));
    }
    action(&mut b, "document.save", "3");
    let mut saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    saved["drawing"]["settings"]["base"] = original["drawing"]["settings"]["base"].clone();
    assert_eq!(saved, original); // No dependent records, presets or electrical data are rewritten.
    assert_eq!(
        snap(&DesktopBridge::from_path(&path).unwrap())["scene"],
        changed["scene"]
    );
    assert_eq!(
        action(&mut b, "document.undo", "4")["snapshot"]["scene"],
        before["scene"]
    );
    assert_eq!(
        action(&mut b, "document.redo", "5")["snapshot"]["scene"],
        changed["scene"]
    );
    assert!(!b.is_dirty());
}
#[test]
fn invalid_defaults_are_atomic_strict_and_cannot_use_stale_or_replayed_input() {
    let mut b = DesktopBridge::demo().unwrap();
    let original = defaults(&mut b, "1");
    let before = snap(&b);
    let mut input = 2;
    for (key, value) in [
        ("font", json!(" \n ")),
        ("text_height", json!("0")),
        ("text_height", json!("4294967296")),
        ("stroke_width", json!("-1")),
        ("colour", json!("#gg0000")),
        ("colour", json!("#é0000")),
        ("text_height", json!(2500)),
        ("unknown", json!("x")),
    ] {
        let mut invalid = original.clone();
        invalid[key] = value;
        assert!(update(&mut b, invalid, &input.to_string()).is_err());
        assert_eq!(snap(&b)["revision"], before["revision"]);
        assert_eq!(snap(&b)["scene"], before["scene"]);
        input += 1;
    }
    let stale = ctx(&b);
    let mut changed = original.clone();
    changed["text_height"] = json!("3200");
    update(&mut b, changed, &input.to_string()).unwrap();
    assert!(update(&mut b, original.clone(), &input.to_string()).is_err());
    input += 1;
    assert!(call(&mut b,json!({"type":"update_defaults","context":stale,"defaults":original,"input":input.to_string()})).is_err());
    let latest = defaults(&mut b, &(input + 1).to_string());
    assert_eq!(latest["text_height"], "3200");
}
#[test]
fn unchanged_defaults_preserve_redo_and_nondefault_presets_and_styles() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("styled.json");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ecad-adapters/tests/fixtures/styled-overview-v6.json");
    std::fs::copy(fixture, &path).unwrap();
    let mut b = DesktopBridge::from_path(&path).unwrap();
    let original = defaults(&mut b, "1");
    let mut changed = original.clone();
    changed["font"] = json!("New fallback");
    update(&mut b, changed.clone(), "2").unwrap();
    action(&mut b, "document.undo", "3");
    let before = snap(&b);
    let reply = update(&mut b, original, "4").unwrap();
    assert_eq!(reply["snapshot"]["revision"], before["revision"]);
    assert!(
        reply["message"]
            .as_str()
            .unwrap()
            .contains("no document changes")
    );
    assert!(!b.is_dirty());
    action(&mut b, "document.redo", "5");
    assert_eq!(defaults(&mut b, "6"), changed);
    let original_file: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    action(&mut b, "document.save", "7");
    let saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for key in ["text_creation", "stroke_creation"] {
        assert_eq!(
            saved["drawing"]["settings"][key],
            original_file["drawing"]["settings"][key]
        );
    }
    for key in ["route_creation", "styles", "views", "labels", "polylines"] {
        assert_eq!(saved["drawing"][key], original_file["drawing"][key]);
    }
}
