//! Sheet-first startup and reviewed attachment leave loaded legacy sources authoritative.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn snapshot(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn call(b: &mut DesktopBridge, v: Value) -> Value {
    serde_json::to_value(
        b.handle(serde_json::from_value::<Request>(v).unwrap())
            .unwrap(),
    )
    .unwrap()
}
#[test]
fn demonstration_starts_on_a_clean_template_backed_sheet_without_a_history_edit() {
    let b = DesktopBridge::demo().unwrap();
    let s = snapshot(&b);
    assert!(!b.is_dirty());
    assert_eq!(s["sheets"].as_array().unwrap().len(), 1);
    let sheet = &s["sheets"][0];
    assert_eq!(sheet["number"], "DEMO");
    assert_eq!(sheet["view"], s["views"][0]["id"]);
    assert_eq!(sheet["paper_x"], "20000");
    assert_eq!(sheet["paper_y"], "60000");
    assert!(
        s["sheet_frames"][0]["template"]
            .as_str()
            .unwrap()
            .starts_with("saved:")
    );
    assert!(
        s["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["id"] == "document.undo" && a["enabled"] == false)
    );
}
#[test]
fn legacy_unplaced_view_is_unchanged_until_reviewed_attachment_and_undo_restores_it() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("legacy.json");
    let bytes = include_str!("../../ecad-adapters/tests/fixtures/routed-conveyor-v8.json");
    std::fs::write(&path, bytes).unwrap();
    let mut b = DesktopBridge::from_path(&path).unwrap();
    let before = snapshot(&b);
    assert_eq!(before["sheets"], json!([]));
    assert!(!b.is_dirty());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), bytes);
    let ctx = json!({"revision":before["revision"],"view":null,"selection":[],"focus":"workspace"});
    let mut draft = call(&mut b, json!({"type":"new_sheet","context":ctx}))["sheet_draft"].clone();
    assert!(draft["view"].is_null());
    draft["view"] = before["views"][0]["id"].clone();
    let preview = call(
        &mut b,
        json!({"type":"preview_sheet","context":ctx,"sheet":draft,"create":true}),
    )["sheet_preview"]
        .clone();
    assert_eq!(preview["scene"], before["scene"]);
    assert_eq!(snapshot(&b)["sheets"], json!([]));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), bytes);
    let input = snapshot(&b)["next_input"].clone();
    call(
        &mut b,
        json!({"type":"commit_sheet","token":preview["token"],"input":input}),
    );
    assert_eq!(snapshot(&b)["scene"], before["scene"]);
    assert_eq!(snapshot(&b)["sheets"][0], draft);
    let after = snapshot(&b);
    call(
        &mut b,
        json!({"type":"action","context":{"revision":after["revision"],"view":null,"selection":[],"focus":"workspace"},"action":"document.undo","input":after["next_input"]}),
    );
    assert_eq!(snapshot(&b)["sheets"], json!([]));
    assert_eq!(snapshot(&b)["scene"], before["scene"]);
    assert!(!b.is_dirty());
}

#[test]
fn captured_sheet_fit_and_context_guards_use_the_registry_without_editing() {
    let mut b = DesktopBridge::demo().unwrap();
    let s = snapshot(&b);
    let context = json!({"revision":s["revision"],"selection":[],"view":s["views"][0]["id"],"sheet":s["sheets"][0]["id"],"focus":"canvas"});
    let fit = call(
        &mut b,
        json!({"type":"action","context":context,"action":"drawing.view.fit","input":s["next_input"]}),
    );
    assert_eq!(fit["request"], "fit");
    assert_eq!(snapshot(&b)["revision"], s["revision"]);
    for (sheet, view) in [
        (json!(format!("{:032x}", 999)), s["views"][0]["id"].clone()),
        (s["sheets"][0]["id"].clone(), Value::Null),
    ] {
        let request = json!({"type":"inspect","context":{"revision":s["revision"],"selection":[],"view":view,"sheet":sheet,"focus":"canvas"}});
        assert!(
            b.handle(serde_json::from_value::<Request>(request).unwrap())
                .is_err()
        );
    }
    assert!(!b.is_dirty());
}
