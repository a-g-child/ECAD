//! Exact paper proposals and ordered-set actions retain source drawing/electrical records.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn snap(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn ctx(b: &DesktopBridge) -> Value {
    json!({"revision":snap(b)["revision"],"selection":[],"view":format!("{:032x}",1),"focus":"canvas"})
}
fn call(b: &mut DesktopBridge, v: Value) -> Result<Value, ecad_bridge::BridgeError> {
    b.handle(serde_json::from_value::<Request>(v).unwrap())
        .map(|r| serde_json::to_value(r).unwrap())
}
fn action(b: &mut DesktopBridge, id: &str) -> Value {
    let v = json!({"type":"action","context":ctx(b),"action":id,"input":snap(b)["next_input"]});
    call(b, v).unwrap()
}
fn seed(b: &mut DesktopBridge) -> Value {
    let v = json!({"type":"new_sheet","context":ctx(b)});
    call(b, v).unwrap()["sheet_draft"].clone()
}
fn preview(
    b: &mut DesktopBridge,
    sheet: Value,
    create: bool,
) -> Result<Value, ecad_bridge::BridgeError> {
    let v = json!({"type":"preview_sheet","context":ctx(b),"sheet":sheet,"create":create});
    call(b, v).map(|r| r["sheet_preview"].clone())
}
fn commit(b: &mut DesktopBridge, p: &Value) -> Result<Value, ecad_bridge::BridgeError> {
    let v = json!({"type":"commit_sheet","token":p["token"],"input":snap(b)["next_input"]});
    call(b, v)
}
fn change(
    b: &mut DesktopBridge,
    id: &Value,
    operation: &str,
    index: usize,
) -> Result<Value, ecad_bridge::BridgeError> {
    let v = json!({"type":"change_sheet","context":ctx(b),"id":id,"operation":operation,"index":index,"input":snap(b)["next_input"]});
    call(b, v)
}
#[test]
fn paper_preview_commit_save_and_history_preserve_source_geometry_and_exact_editor_values() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sheets.json");
    let mut b = DesktopBridge::demo().unwrap();
    b.save_as(&path).unwrap();
    let before = snap(&b);
    let original: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(action(&mut b, "drawing.sheets.edit")["request"], "sheets");
    let mut row = seed(&mut b);
    let id = row["id"].clone();
    row["title"] = json!("Harness\nDetail");
    row["source_x"] = json!("9007199254740993");
    row["numerator"] = json!("1");
    row["denominator"] = json!("2");
    let p = preview(&mut b, row.clone(), true).unwrap();
    assert_eq!(p["sheet"], row);
    assert_eq!(p["scene"], before["scene"]);
    assert_eq!(snap(&b)["revision"], before["revision"]);
    assert!(!b.is_dirty());
    commit(&mut b, &p).unwrap();
    assert_eq!(snap(&b)["sheets"][0]["id"], id);
    assert_eq!(snap(&b)["scene"], before["scene"]);
    action(&mut b, "document.save");
    let loaded = DesktopBridge::from_path(&path).unwrap();
    assert_eq!(snap(&loaded)["sheets"], snap(&b)["sheets"]);
    let mut saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    saved["sheets"] = json!([]);
    saved["drawing_documents"] = original["drawing_documents"].clone();
    assert_eq!(saved, original);
    action(&mut b, "document.undo");
    assert_eq!(snap(&b)["sheets"], json!([]));
    action(&mut b, "document.redo");
    assert!(!b.is_dirty());
    assert_eq!(seed(&mut b)["number"], "2");
}
#[test]
fn invalid_cancelled_and_stale_paper_proposals_cannot_commit_and_noops_preserve_redo() {
    let mut b = DesktopBridge::demo().unwrap();
    let row = seed(&mut b);
    let p = preview(&mut b, row.clone(), true).unwrap();
    let mut bad = row.clone();
    bad["width"] = json!("0");
    assert!(preview(&mut b, bad, true).is_err());
    assert!(commit(&mut b, &p).is_err());
    let p = preview(&mut b, row.clone(), true).unwrap();
    call(&mut b, json!({"type":"cancel_sheet"})).unwrap();
    assert!(commit(&mut b, &p).is_err());
    let p = preview(&mut b, row.clone(), true).unwrap();
    let v = json!({"type":"update_metadata","context":ctx(&b),"values":{"project.issue":"B"},"input":snap(&b)["next_input"]});
    call(&mut b, v).unwrap();
    assert!(commit(&mut b, &p).is_err());
    action(&mut b, "document.undo");
    let p = preview(&mut b, row.clone(), true).unwrap();
    commit(&mut b, &p).unwrap();
    let v = json!({"type":"update_metadata","context":ctx(&b),"values":{"project.issue":"C"},"input":snap(&b)["next_input"]});
    call(&mut b, v).unwrap();
    action(&mut b, "document.undo");
    let revision = snap(&b)["revision"].clone();
    let p = preview(&mut b, row, false).unwrap();
    commit(&mut b, &p).unwrap();
    assert_eq!(snap(&b)["revision"], revision);
    action(&mut b, "document.redo");
    let meta = action(&mut b, "document.metadata.edit");
    assert!(
        meta["metadata"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["key"] == "project.issue" && r["value"] == "C")
    );
}
#[test]
fn sheets_can_be_blank_reordered_and_removed_without_deleting_or_renumbering_views() {
    let mut b = DesktopBridge::demo().unwrap();
    let scene = snap(&b)["scene"].clone();
    let first = seed(&mut b);
    let p = preview(&mut b, first.clone(), true).unwrap();
    commit(&mut b, &p).unwrap();
    let mut second = seed(&mut b);
    second["view"] = Value::Null;
    second["title"] = json!("Cover");
    let p = preview(&mut b, second.clone(), true).unwrap();
    assert_eq!(p["scene"], json!([]));
    commit(&mut b, &p).unwrap();
    change(&mut b, &second["id"], "move", 0).unwrap();
    assert_eq!(snap(&b)["sheets"][0]["number"], "2");
    assert_eq!(snap(&b)["sheets"][1]["number"], "1");
    assert!(change(&mut b, &first["id"], "move", 2).is_err());
    change(&mut b, &first["id"], "remove", 0).unwrap();
    assert_eq!(snap(&b)["scene"], scene);
    action(&mut b, "document.undo");
    assert_eq!(snap(&b)["sheets"][1]["id"], first["id"]);
    action(&mut b, "document.undo");
    assert_eq!(snap(&b)["sheets"][0]["id"], first["id"]);
}
