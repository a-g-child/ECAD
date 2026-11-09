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

#[test]
fn inline_text_contents_remain_transient_validate_strictly_and_commit_one_complete_object() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(&dir);
    let original = variant(&b);
    let runs = json!([{"kind":"literal","value":"Company\n  Office\n"},{"kind":"field","value":"company.name"}]);
    let edit = json!({"start":["10000","20000"],"end":["70000","40000"],"contents":runs});
    let p = preview(&mut b, "template.draw.text", ID, edit.clone()).unwrap();
    assert_eq!(variant(&b), original);
    assert_eq!(p["graphics"][0]["contents"], runs);
    assert!(action(&mut b, "document.save").is_err());
    assert!(action(&mut b, "template.close").is_err());
    for contents in [
        json!([]),
        json!([{"kind":"literal","value":" \n "}]),
        json!([{"kind":"field","value":"unknown.field"}]),
    ] {
        let mut invalid = edit.clone();
        invalid["contents"] = contents;
        assert!(preview(&mut b, "template.draw.text", ID, invalid).is_err());
        assert!(commit(&mut b, p["token"].clone()).is_err());
        assert_eq!(variant(&b), original);
    }
    let p = preview(&mut b, "template.draw.text", ID, edit).unwrap();
    commit(&mut b, p["token"].clone()).unwrap();
    let created = variant(&b);
    assert_eq!(created["graphics"][0]["contents"], runs);
    action(&mut b, "document.undo").unwrap();
    assert_eq!(variant(&b), original);
    let variable = json!({"start":["10000","20000"],"end":["70000","40000"],"contents":[{"kind":"field","value":"sheet.title"}]});
    preview(&mut b, "template.draw.text", ID, variable.clone()).unwrap();
    action(&mut b, "placement.cancel").unwrap();
    action(&mut b, "document.redo").unwrap();
    assert_eq!(variant(&b), created);
    assert!(preview(&mut b, "template.artwork.move", ID, variable.clone()).is_err());
    action(&mut b, "placement.cancel").unwrap();
    assert!(
        preview(
            &mut b,
            "template.draw.rectangle",
            "00000000000000000000000000000002",
            variable
        )
        .is_err()
    );
    action(&mut b, "placement.cancel").unwrap();
    assert_eq!(variant(&b), created);
}

#[test]
fn complete_path_proposals_keep_vertices_and_reject_mixed_invalid_or_obsolete_commits() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(&dir);
    assert_eq!(
        action(&mut b, "template.draw.polyline").unwrap()["request"],
        "template_draw_polyline"
    );
    let original = variant(&b);
    let edit =
        json!({"points":[["50000","40000"],["10000","40000"],["10000","20000"],["50000","40000"]]});
    let valid = preview(&mut b, "template.draw.polyline", ID, edit.clone()).unwrap();
    assert_eq!(variant(&b), original);
    assert!(action(&mut b, "document.save").is_err());
    for invalid in [
        json!({"points":[]}),
        json!({"points":[["0","0"],["-1","0"]]}),
        json!({"points":[["0","0"],["0","0"]]}),
        json!({"points":[["0","0"],["10","10"]],"start":["0","0"]}),
        json!({"points":[["0","0"],["10","10"]],"contents":[]}),
    ] {
        assert!(preview(&mut b, "template.draw.polyline", ID, invalid).is_err());
        assert!(commit(&mut b, valid["token"].clone()).is_err());
        assert_eq!(variant(&b), original);
    }
    let valid = preview(&mut b, "template.draw.polyline", ID, edit.clone()).unwrap();
    commit(&mut b, valid["token"].clone()).unwrap();
    let created = variant(&b);
    assert_eq!(
        created["graphics"][0]["points"],
        json!([
            ["40000", "20000"],
            ["0", "20000"],
            ["0", "0"],
            ["40000", "20000"]
        ])
    );
    action(&mut b, "document.undo").unwrap();
    assert_eq!(variant(&b), original);
    preview(
        &mut b,
        "template.draw.polyline",
        ID,
        json!({"start":null,"end":null}),
    )
    .unwrap();
    action(&mut b, "placement.cancel").unwrap();
    action(&mut b, "document.redo").unwrap();
    assert_eq!(variant(&b), created);
    assert!(preview(&mut b, "template.artwork.move", ID, edit).is_err());
    action(&mut b, "placement.cancel").unwrap();
    assert_eq!(variant(&b), created);
}
