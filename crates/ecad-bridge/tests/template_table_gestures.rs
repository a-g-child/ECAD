//! Closed preview tokens retain fresh inserted IDs and isolate transient interaction from history.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn snapshot(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn context(b: &DesktopBridge) -> Value {
    let s = snapshot(b);
    json!({"revision":s["revision"],"editor":s["template_editor"]["revision"],"selection":[],"view":null,"sheet":null,"focus":"canvas"})
}
fn call(b: &mut DesktopBridge, v: Value) -> Result<Value, ecad_bridge::BridgeError> {
    b.handle(serde_json::from_value::<Request>(v).unwrap())
        .map(|r| serde_json::to_value(r).unwrap())
}
fn action(b: &mut DesktopBridge, id: &str) -> Result<Value, ecad_bridge::BridgeError> {
    let v =
        json!({"type":"action","context":context(b),"action":id,"input":snapshot(b)["next_input"]});
    call(b, v)
}
fn variant(b: &DesktopBridge) -> Value {
    snapshot(b)["template_editor"]["variants"][0].clone()
}
fn preview(
    b: &mut DesktopBridge,
    action: &str,
    edit: Value,
) -> Result<Value, ecad_bridge::BridgeError> {
    let v = variant(b);
    let r = json!({"type":"template_table_preview","context":context(b),"action":action,"values":{"variant":v["id"],"table":v["tables"][0]["id"],"edit":edit},"input":snapshot(b)["next_input"]});
    call(b, r)
}
fn commit(b: &mut DesktopBridge, token: Value) -> Result<Value, ecad_bridge::BridgeError> {
    let r = json!({"type":"template_table_commit","context":context(b),"preview":token,"input":snapshot(b)["next_input"]});
    call(b, r)
}
fn bridge(dir: &tempfile::TempDir) -> DesktopBridge {
    let mut b = DesktopBridge::demo().unwrap();
    b.configure_template_store(dir.path().join("templates.json"))
        .unwrap();
    action(&mut b, "library.templates").unwrap();
    b
}
#[test]
fn inserted_preview_commits_exact_latest_ids_as_one_history_entry_and_keeps_the_far_edge() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(&dir);
    let original = variant(&b);
    let project = snapshot(&b)["revision"].clone();
    let edit = json!({"operation":"insert","axis":"y","at":"40000","size":"10000","anchor":"end"});
    let p1 = preview(&mut b, "template.table.insert_row", edit.clone()).unwrap()["table_preview"]
        .clone();
    assert_eq!(variant(&b), original);
    assert!(b.is_dirty());
    assert!(action(&mut b, "document.save").is_err());
    assert!(action(&mut b, "template.discard").is_err());
    let p2 = preview(&mut b, "template.table.insert_row", edit).unwrap()["table_preview"].clone();
    assert_ne!(
        p1["tables"][0]["cells"][8]["id"],
        p2["tables"][0]["cells"][8]["id"]
    );
    assert!(commit(&mut b, p1["token"].clone()).is_err());
    commit(&mut b, p2["token"].clone()).unwrap();
    assert_eq!(variant(&b)["tables"], p2["tables"]);
    assert!(commit(&mut b, p2["token"].clone()).is_err());
    action(&mut b, "document.undo").unwrap();
    assert_eq!(variant(&b), original);
    assert!(!b.is_dirty());
    action(&mut b, "document.redo").unwrap();
    assert_eq!(variant(&b)["tables"], p2["tables"]);
    assert_eq!(snapshot(&b)["revision"], project);
    action(&mut b, "document.save").unwrap();
    assert!(!b.is_dirty());
}
#[test]
fn invalid_newer_proposals_invalidate_old_tokens_and_cancel_retains_redo_and_checkpoint() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(&dir);
    let original = variant(&b);
    let p = preview(
        &mut b,
        "template.table.resize",
        json!({"operation":"resize","bounds":["100000","150000","180000","40000"]}),
    )
    .unwrap()["table_preview"]
        .clone();
    commit(&mut b, p["token"].clone()).unwrap();
    action(&mut b, "document.undo").unwrap();
    let p = preview(
        &mut b,
        "template.table.boundary",
        json!({"operation":"boundary","axis":"x","from":"90000","to":"80000"}),
    )
    .unwrap()["table_preview"]
        .clone();
    assert!(
        preview(
            &mut b,
            "template.table.boundary",
            json!({"operation":"boundary","axis":"x","from":"90000","to":"0"})
        )
        .is_err()
    );
    assert!(commit(&mut b, p["token"].clone()).is_err());
    assert!(
        preview(
            &mut b,
            "template.table.boundary",
            json!({"operation":"boundary","axis":"y","from":"10000","to":"5000"})
        )
        .is_err()
    );
    assert!(
        preview(
            &mut b,
            "template.table.resize",
            json!({"operation":"resize","bounds":["100000","150000","180000","40000"]})
        )
        .is_err()
    );
    let pending =
        json!({"type":"template_pending","context":context(&b),"input":snapshot(&b)["next_input"]});
    assert!(call(&mut b, pending).is_err());
    action(&mut b, "placement.cancel").unwrap();
    assert_eq!(variant(&b), original);
    assert!(!b.is_dirty());
    action(&mut b, "document.redo").unwrap();
    assert_ne!(variant(&b), original);
    let old = context(&b);
    action(&mut b, "template.discard").unwrap();
    let r = json!({"type":"template_table_commit","context":old,"preview":p["token"],"input":snapshot(&b)["next_input"]});
    assert!(call(&mut b, r).is_err());
}
#[test]
fn dirty_inspector_fields_and_malformed_previews_cannot_commit_or_steal_another_target() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(&dir);
    let original = variant(&b);
    let pending =
        json!({"type":"template_pending","context":context(&b),"input":snapshot(&b)["next_input"]});
    call(&mut b, pending).unwrap();
    let edit = json!({"operation":"resize","bounds":["100000","150000","180000","40000"]});
    assert!(preview(&mut b, "template.table.resize", edit.clone()).is_err());
    action(&mut b, "template.properties.revert").unwrap();
    let p = preview(&mut b, "template.table.resize", edit).unwrap()["table_preview"].clone();
    assert!(
        preview(
            &mut b,
            "template.table.resize",
            json!({"operation":"resize","bounds":["1","2"]})
        )
        .is_err()
    );
    assert!(commit(&mut b, p["token"].clone()).is_err());
    action(&mut b, "placement.cancel").unwrap();
    assert_eq!(variant(&b), original);
    assert!(!b.is_dirty());
    assert!(
        preview(
            &mut b,
            "template.table.insert_row",
            json!({"operation":"insert","axis":"x","at":"0","size":"10000","anchor":"end"})
        )
        .is_err()
    );
    action(&mut b, "placement.cancel").unwrap();
    assert_eq!(variant(&b), original);
}
