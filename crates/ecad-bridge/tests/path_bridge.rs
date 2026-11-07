//! Accepted vertices, tentative legs, strict creation presets and guarded path document history.
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
fn begin(b: &mut DesktopBridge) -> Value {
    let c = ctx(b);
    call(b, json!({"type":"begin_path","context":c})).unwrap()["path"].clone()
}
fn preview(
    b: &mut DesktopBridge,
    session: &Value,
    x: &str,
    y: &str,
    exact: bool,
) -> Result<Value, ecad_bridge::BridgeError> {
    call(b,json!({"type":"preview_path","session":session,"placement":{"x":x,"y":y,"exact":exact,"suppressed":false,"candidate":null,"tolerance_um":3000}})).map(|r|r["path"].clone())
}
fn add(b: &mut DesktopBridge, session: &Value, x: &str, y: &str) -> Value {
    let p = preview(b, session, x, y, true).unwrap();
    call(b, json!({"type":"add_path_vertex","token":p["token"]})).unwrap()["path"].clone()
}
fn action(b: &mut DesktopBridge, action: &str, input: &str) -> Value {
    let c = ctx(b);
    call(
        b,
        json!({"type":"action","context":c,"action":action,"input":input}),
    )
    .unwrap()
}
#[test]
fn finish_uses_accepted_vertices_and_stroke_preset_without_accepting_tentative_leg() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("path.json");
    let mut original = DesktopBridge::demo().unwrap();
    original.save_as(&path).unwrap();
    let mut data: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    data["drawing"]["settings"]["stroke_creation"]["overrides"]["stroke_width_um"] = json!(750);
    data["drawing"]["settings"]["stroke_creation"]["overrides"]["colour"] = json!([144, 64, 32]);
    std::fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    let mut b = DesktopBridge::from_path(&path).unwrap();
    let before = snap(&b);
    let seed = begin(&mut b);
    assert_eq!(seed["can_finish"], false);
    add(&mut b, &seed["session"], "0", "-25000");
    let accepted = add(&mut b, &seed["session"], "25000", "-25000");
    assert_eq!(
        accepted["scene"][0]["points"],
        json!([[0.0, -25.0], [25.0, -25.0]])
    );
    let rubber = preview(&mut b, &seed["session"], "50000", "-10000", true).unwrap();
    assert_eq!(
        rubber["rubber"]["points"],
        json!([[25.0, -25.0], [50.0, -10.0]])
    );
    assert_eq!(rubber["scene"], accepted["scene"]);
    assert_eq!(rubber["scene"][0]["width"], 0.75);
    assert_eq!(rubber["scene"][0]["colour"], "#904020");
    assert_eq!(snap(&b), before);
    assert!(!b.is_dirty());
    let committed = call(
        &mut b,
        json!({"type":"commit_path","token":rubber["token"],"input":"1"}),
    )
    .unwrap();
    assert!(
        committed["snapshot"]["scene"]
            .as_array()
            .unwrap()
            .contains(&accepted["scene"][0])
    );
    assert!(b.is_dirty());
    assert_eq!(
        action(&mut b, "document.undo", "2")["snapshot"]["scene"],
        before["scene"]
    );
    assert!(!b.is_dirty());
    assert_eq!(
        action(&mut b, "document.redo", "3")["snapshot"]["scene"],
        committed["snapshot"]["scene"]
    );
    action(&mut b, "document.save", "4");
    let loaded = DesktopBridge::from_path(&path).unwrap();
    assert_eq!(snap(&loaded)["scene"], snap(&b)["scene"]);
    let saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        saved["drawing"]["polylines"][0]["appearance"],
        data["drawing"]["settings"]["stroke_creation"]
    );
    for key in ["devices", "conductors", "cables"] {
        assert_eq!(saved[key], data[key]);
    }
}
#[test]
fn back_duplicate_invalid_cancelled_and_superseded_drafts_never_apply_old_geometry() {
    let mut b = DesktopBridge::demo().unwrap();
    let before = snap(&b);
    let seed = begin(&mut b);
    let first = add(&mut b, &seed["session"], "10000", "0");
    assert!(
        call(
            &mut b,
            json!({"type":"add_path_vertex","token":first["token"]})
        )
        .is_err()
    );
    assert!(
        call(
            &mut b,
            json!({"type":"commit_path","token":first["token"],"input":"1"})
        )
        .is_err()
    );
    let second = add(&mut b, &seed["session"], "20000", "0");
    assert!(
        call(
            &mut b,
            json!({"type":"add_path_vertex","token":first["token"]})
        )
        .is_err()
    );
    assert!(preview(&mut b, &seed["session"], "9223372036854775808", "0", true).is_err());
    assert!(
        call(
            &mut b,
            json!({"type":"commit_path","token":second["token"],"input":"2"})
        )
        .is_err()
    );
    let p = preview(&mut b, &seed["session"], "20000", "0", true).unwrap();
    let back = call(
        &mut b,
        json!({"type":"back_path_vertex","token":p["token"]}),
    )
    .unwrap()["path"]
        .clone();
    assert_eq!(back["vertices"].as_array().unwrap().len(), 1);
    assert_eq!(back["can_finish"], false);
    assert!(
        call(
            &mut b,
            json!({"type":"back_path_vertex","token":p["token"]})
        )
        .is_err()
    );
    let final_path = add(&mut b, &seed["session"], "30000", "0");
    assert_eq!(
        final_path["vertices"],
        json!([["10000", "0"], ["30000", "0"]])
    );
    assert_eq!(
        action(&mut b, "placement.cancel", "3")["message"],
        "Path cancelled"
    );
    assert!(
        call(
            &mut b,
            json!({"type":"commit_path","token":final_path["token"],"input":"4"})
        )
        .is_err()
    );
    assert_eq!(snap(&b)["scene"], before["scene"]);
    assert!(!b.is_dirty());
    let old = begin(&mut b);
    let new = begin(&mut b);
    assert_ne!(old["session"], new["session"]);
    assert!(preview(&mut b, &old["session"], "1", "2", true).is_err());
}
#[test]
fn exact_large_coordinates_snap_feedback_stale_document_and_replay_guards() {
    let mut b = DesktopBridge::demo().unwrap();
    let seed = begin(&mut b);
    let p = preview(&mut b, &seed["session"], "99100", "1000", false).unwrap();
    assert_eq!(p["snap"], "Terminal");
    assert_eq!(p["x"], "100000");
    let q = add(
        &mut b,
        &seed["session"],
        "9007199254740993",
        "-9223372036854775808",
    );
    assert_eq!(
        q["vertices"],
        json!([["9007199254740993", "-9223372036854775808"]])
    );
    let q = add(
        &mut b,
        &seed["session"],
        "9007199254740994",
        "-9223372036854775808",
    );
    let mut c = ctx(&b);
    c["selection"] = json!([{"kind":"label","id":format!("{:032x}",1)}]);
    call(
        &mut b,
        json!({"type":"edit","context":c,"property":"drawing_text","value":"new text"}),
    )
    .unwrap();
    let edited = snap(&b);
    assert!(
        call(
            &mut b,
            json!({"type":"commit_path","token":q["token"],"input":"1"})
        )
        .is_err()
    );
    assert_eq!(snap(&b)["scene"], edited["scene"]);
    let next = begin(&mut b);
    add(&mut b, &next["session"], "0", "0");
    let q = add(&mut b, &next["session"], "1000", "0");
    call(
        &mut b,
        json!({"type":"commit_path","token":q["token"],"input":"2"}),
    )
    .unwrap();
    assert_eq!(
        call(
            &mut b,
            json!({"type":"commit_path","token":q["token"],"input":"2"})
        )
        .unwrap_err()
        .code,
        "replayed"
    );
}
