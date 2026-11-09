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
    id: &str,
    edit: Value,
) -> Result<Value, ecad_bridge::BridgeError> {
    let v = variant(b);
    let r = json!({"type":"template_geometry_preview","context":context(b),"action":action,"values":{"variant":v["id"],"graphic":id,"edit":edit},"input":snapshot(b)["next_input"]});
    call(b, r).map(|v| v["table_preview"].clone())
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

const ID: &str = "00000000000000000000000000000001";
#[test]
fn armed_tools_guard_close_and_closed_creation_move_preserve_history_and_identity() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(&dir);
    let original = variant(&b);
    let p = preview(
        &mut b,
        "template.draw.text",
        ID,
        json!({"start":null,"end":null}),
    )
    .unwrap();
    assert_eq!(p["token"], "");
    assert!(b.is_dirty());
    assert_eq!(variant(&b), original);
    assert!(action(&mut b, "document.save").is_err());
    assert!(action(&mut b, "template.close").is_err());
    assert!(commit(&mut b, p["token"].clone()).is_err());
    let p = preview(
        &mut b,
        "template.draw.text",
        ID,
        json!({"start":["50000","40000"],"end":["10000","20000"]}),
    )
    .unwrap();
    commit(&mut b, p["token"].clone()).unwrap();
    assert_eq!(variant(&b)["graphics"], p["graphics"]);
    action(&mut b, "document.undo").unwrap();
    assert_eq!(variant(&b), original);
    action(&mut b, "document.redo").unwrap();
    let before = variant(&b);
    let p = preview(
        &mut b,
        "template.artwork.move",
        ID,
        json!({"start":["0","0"],"end":["30000","30000"]}),
    )
    .unwrap();
    commit(&mut b, p["token"].clone()).unwrap();
    assert_eq!(
        variant(&b)["graphics"][0]["contents"],
        before["graphics"][0]["contents"]
    );
    assert_eq!(variant(&b)["graphics"][0]["id"], ID);
    action(&mut b, "document.undo").unwrap();
    assert_eq!(variant(&b), before);
    preview(
        &mut b,
        "template.draw.line",
        "00000000000000000000000000000002",
        json!({"start":null,"end":null}),
    )
    .unwrap();
    action(&mut b, "placement.cancel").unwrap();
    action(&mut b, "document.redo").unwrap();
    assert_eq!(variant(&b)["graphics"], p["graphics"]);
}
#[test]
fn invalid_superseded_and_retargeted_proposals_cannot_commit_and_cancel_keeps_state() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(&dir);
    let original = variant(&b);
    let edit = json!({"start":["10000","20000"],"end":["50000","40000"]});
    let p = preview(&mut b, "template.draw.rectangle", ID, edit.clone()).unwrap();
    assert!(preview(&mut b, "template.draw.ellipse", ID, edit.clone()).is_err());
    assert!(commit(&mut b, p["token"].clone()).is_err());
    let p = preview(&mut b, "template.draw.rectangle", ID, edit.clone()).unwrap();
    assert!(
        preview(
            &mut b,
            "template.draw.rectangle",
            ID,
            json!({"start":["0","0"],"end":["-1","10"]})
        )
        .is_err()
    );
    assert!(commit(&mut b, p["token"].clone()).is_err());
    let p = preview(&mut b, "template.draw.rectangle", ID, edit.clone()).unwrap();
    let latest = preview(&mut b, "template.draw.rectangle", ID, edit).unwrap();
    assert!(commit(&mut b, p["token"].clone()).is_err());
    action(&mut b, "placement.cancel").unwrap();
    assert!(commit(&mut b, latest["token"].clone()).is_err());
    assert_eq!(variant(&b), original);
}
