//! Project information uses shared actions, captured revisions and exact persistent history.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn snap(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn ctx(b: &DesktopBridge) -> Value {
    json!({"revision":snap(b)["revision"],"selection":[],"view":null,"focus":"workspace"})
}
fn call(b: &mut DesktopBridge, value: Value) -> Result<Value, ecad_bridge::BridgeError> {
    b.handle(serde_json::from_value::<Request>(value).unwrap())
        .map(|r| serde_json::to_value(r).unwrap())
}
fn action(b: &mut DesktopBridge, id: &str) -> Value {
    let value = json!({"type":"action","context":ctx(b),"action":id,"input":snap(b)["next_input"]});
    call(b, value).unwrap()
}
fn update(b: &mut DesktopBridge, values: Value) -> Result<Value, ecad_bridge::BridgeError> {
    let request = json!({"type":"update_metadata","context":ctx(b),"values":values,"input":snap(b)["next_input"]});
    call(b, request)
}
fn value(rows: &Value, key: &str) -> Value {
    rows.as_array()
        .unwrap()
        .iter()
        .find(|r| r["key"] == key)
        .unwrap()["value"]
        .clone()
}
#[test]
fn metadata_editor_request_and_commit_are_portable_one_step_and_noop_preserves_redo() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let mut b = DesktopBridge::demo().unwrap();
    b.save_as(&path).unwrap();
    let before = snap(&b);
    let seed = action(&mut b, "document.metadata.edit");
    assert_eq!(seed["request"], "metadata");
    assert_eq!(seed["metadata"].as_array().unwrap().len(), 8);
    assert!(!b.is_dirty());
    let updated=update(&mut b,json!({"company.name":"Example","company.address":"First line\n\nLast line\n","project.title":"Vehicle harness","project.issue":"B"})).unwrap();
    assert_eq!(updated["snapshot"]["scene"], before["scene"]);
    action(&mut b, "document.save");
    let mut loaded = DesktopBridge::from_path(&path).unwrap();
    assert_eq!(
        value(
            &action(&mut loaded, "document.metadata.edit")["metadata"],
            "company.address"
        ),
        "First line\n\nLast line\n"
    );
    assert_eq!(
        value(
            &action(&mut loaded, "document.metadata.edit")["metadata"],
            "project.title"
        ),
        "Vehicle harness"
    );
    action(&mut b, "document.undo");
    let revision = snap(&b)["revision"].clone();
    update(&mut b, json!({"company.name":""})).unwrap();
    assert_eq!(snap(&b)["revision"], revision);
    action(&mut b, "document.redo");
    assert!(!b.is_dirty());
    assert_eq!(
        value(
            &action(&mut b, "document.metadata.edit")["metadata"],
            "project.issue"
        ),
        "B"
    );
}
#[test]
fn unknown_fields_and_stale_or_replayed_metadata_requests_preserve_current_values() {
    let mut b = DesktopBridge::demo().unwrap();
    let before = snap(&b);
    assert!(update(&mut b, json!({"sheet.title":"wrong scope"})).is_err());
    assert_eq!(snap(&b)["revision"], before["revision"]);
    let captured = ctx(&b);
    let input = snap(&b)["next_input"].clone();
    update(&mut b, json!({"project.title":"Current"})).unwrap();
    let current = ctx(&b);
    assert!(
        call(
            &mut b,
            json!({"type":"update_metadata","context":current,"values":{},"input":input})
        )
        .is_err()
    );
    let fresh = snap(&b)["next_input"].clone();
    assert!(call(&mut b,json!({"type":"update_metadata","context":captured,"values":{"project.title":"Stale"},"input":fresh})).is_err());
    assert_eq!(
        value(
            &action(&mut b, "document.metadata.edit")["metadata"],
            "project.title"
        ),
        "Current"
    );
}
