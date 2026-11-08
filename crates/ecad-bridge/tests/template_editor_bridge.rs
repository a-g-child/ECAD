//! Independent template workspace targets, durable publication and explicit project import.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn snapshot(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn context(b: &DesktopBridge) -> Value {
    let s = snapshot(b);
    json!({"revision":s["revision"],"editor":s["template_editor"]["revision"],"selection":[],"view":null,"sheet":null,"focus":"canvas"})
}
fn call(b: &mut DesktopBridge, value: Value) -> Result<Value, ecad_bridge::BridgeError> {
    b.handle(serde_json::from_value::<Request>(value).unwrap())
        .map(|r| serde_json::to_value(r).unwrap())
}
fn action(b: &mut DesktopBridge, id: &str) -> Result<Value, ecad_bridge::BridgeError> {
    let r =
        json!({"type":"action","action":id,"context":context(b),"input":snapshot(b)["next_input"]});
    call(b, r)
}
fn proposal(b: &DesktopBridge) -> Value {
    let s = snapshot(b);
    let v = &s["template_editor"]["variants"][0];
    json!({"variant":v["id"],"name":v["name"],"paper":v["paper"],"content":v["content"],"cell":null})
}
fn apply(b: &mut DesktopBridge, value: Value) -> Result<Value, ecad_bridge::BridgeError> {
    let r = json!({"type":"template_operation","context":context(b),"action":"template.properties.apply","source":null,"values":value,"input":snapshot(b)["next_input"]});
    call(b, r)
}
#[test]
fn template_edit_save_and_project_import_have_separate_history_pins_and_native_dirty_guards() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = DesktopBridge::demo().unwrap();
    let library = dir.path().join("config").join("templates.json");
    b.configure_template_store(&library).unwrap();
    let project = dir.path().join("project.json");
    b.save_as(&project).unwrap();
    let before = snapshot(&b);
    assert!(
        before["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["id"] == "library.templates" && a["enabled"] == true)
    );
    action(&mut b, "library.templates").unwrap();
    let opened = snapshot(&b);
    assert!(!opened["template_editor"].is_null());
    assert_eq!(opened["revision"], before["revision"]);
    assert_eq!(opened["scene"], before["scene"]);
    assert!(!library.exists());
    assert!(!library.parent().unwrap().exists());
    assert!(!b.is_dirty());
    let old = context(&b);
    let mut edited = proposal(&b);
    edited["name"] = "Edited\nTitle block".into();
    apply(&mut b, edited).unwrap();
    assert!(b.is_dirty());
    assert_eq!(snapshot(&b)["dirty"], false);
    assert_eq!(snapshot(&b)["sheets"], before["sheets"]);
    assert!(b.open(&project).is_err());
    assert!(b.save_as(&dir.path().join("wrong.json")).is_err());
    assert!(action(&mut b, "template.close").is_err());
    let stale = json!({"type":"action","context":old,"action":"document.save","input":snapshot(&b)["next_input"]});
    assert!(call(&mut b, stale).is_err());
    action(&mut b, "document.save").unwrap();
    assert!(library.exists());
    assert!(!b.is_dirty());
    let saved = snapshot(&b)["template_editor"]["published"].clone();
    action(&mut b, "document.undo").unwrap();
    assert!(b.is_dirty());
    assert_eq!(snapshot(&b)["revision"], before["revision"]);
    action(&mut b, "document.redo").unwrap();
    assert!(!b.is_dirty());
    assert_eq!(snapshot(&b)["template_editor"]["published"], saved);
    action(&mut b, "template.import").unwrap();
    let imported = snapshot(&b);
    assert_eq!(imported["dirty"], true);
    assert_eq!(imported["sheets"], before["sheets"]);
    assert_eq!(imported["sheet_frames"], before["sheet_frames"]);
    action(&mut b, "template.close").unwrap();
    assert!(snapshot(&b)["template_editor"].is_null());
    action(&mut b, "document.undo").unwrap();
    assert_eq!(snapshot(&b)["dirty"], false);
    assert_eq!(snapshot(&b)["template_choices"], before["template_choices"]);
    assert!(library.exists());
}
#[test]
fn pending_invalid_properties_stale_epochs_and_project_requests_never_retarget_an_editor() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = DesktopBridge::demo().unwrap();
    b.configure_template_store(dir.path().join("templates.json"))
        .unwrap();
    let project_context = context(&b);
    let pending = json!({"type":"template_pending","context":project_context,"input":snapshot(&b)["next_input"]});
    assert!(call(&mut b, pending).is_err());
    assert!(!b.is_dirty());
    action(&mut b, "library.templates").unwrap();
    let pending =
        json!({"type":"template_pending","context":context(&b),"input":snapshot(&b)["next_input"]});
    call(&mut b, pending.clone()).unwrap();
    assert!(b.is_dirty());
    assert!(call(&mut b, pending).is_err());
    assert!(action(&mut b, "template.close").is_err());
    let before = snapshot(&b);
    let mut bad = proposal(&b);
    bad["paper"][0] = "0".into();
    assert!(apply(&mut b, bad).is_err());
    assert_eq!(
        snapshot(&b)["template_editor"]["revision"],
        before["template_editor"]["revision"]
    );
    assert!(b.is_dirty());
    let old = context(&b);
    action(&mut b, "template.properties.revert").unwrap();
    assert!(!b.is_dirty());
    action(&mut b, "template.new").unwrap();
    assert!(b.is_dirty());
    let stale = json!({"type":"action","context":old,"action":"document.undo","input":snapshot(&b)["next_input"]});
    assert!(call(&mut b, stale).is_err());
    let stale = json!({"type":"action","context":project_context,"action":"document.undo","input":snapshot(&b)["next_input"]});
    assert!(call(&mut b, stale).is_err());
    let edit = json!({"type":"edit","context":context(&b),"property":"designation","value":"must not edit"});
    assert!(call(&mut b, edit).is_err());
    assert!(
        b.configure_template_store(dir.path().join("other.json"))
            .is_err()
    );
    action(&mut b, "template.discard").unwrap();
    assert!(!b.is_dirty());
}
#[test]
fn failed_publication_retains_unsaved_draft_and_saved_library_reopens_independently() {
    let dir = tempfile::tempdir().unwrap();
    let library = dir.path().join("templates.json");
    let mut b = DesktopBridge::demo().unwrap();
    b.configure_template_store(&library).unwrap();
    action(&mut b, "library.templates").unwrap();
    let mut value = proposal(&b);
    value["name"] = "Saved name".into();
    apply(&mut b, value).unwrap();
    action(&mut b, "document.save").unwrap();
    let mut value = proposal(&b);
    value["name"] = "Unsaved next name".into();
    apply(&mut b, value).unwrap();
    let before = snapshot(&b)["template_editor"].clone();
    let bytes = std::fs::read(&library).unwrap();
    std::fs::write(&library, "external invalid replacement").unwrap();
    assert!(action(&mut b, "document.save").is_err());
    assert_eq!(snapshot(&b)["template_editor"], before);
    assert!(b.is_dirty());
    assert!(action(&mut b, "template.library.reload").is_err());
    assert_eq!(snapshot(&b)["template_editor"], before);
    std::fs::write(&library, bytes).unwrap();
    action(&mut b, "template.library.reload").unwrap();
    action(&mut b, "document.save").unwrap();
    action(&mut b, "template.discard").unwrap();
    assert!(action(&mut b, "library.templates").is_ok()); // The existing publisher retains its observed validated catalogue; no silent reload.
    assert!(
        snapshot(&b)["template_editor"]["catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r[1] == "Saved name")
    );
}
