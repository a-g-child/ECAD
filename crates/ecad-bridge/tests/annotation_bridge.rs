//! New-object preview validation, preset/style parity, persistent identity and normal T history.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn call(b: &mut DesktopBridge, value: Value) -> Result<Value, ecad_bridge::BridgeError> {
    b.handle(serde_json::from_value::<Request>(value).unwrap())
        .map(|r| serde_json::to_value(r).unwrap())
}
fn snapshot(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn context(b: &DesktopBridge) -> Value {
    json!({"revision":snapshot(b)["revision"],"selection":[],"view":format!("{:032x}",1),"focus":"canvas"})
}
fn begin(b: &mut DesktopBridge) -> Value {
    let c = context(b);
    call(b, json!({"type":"begin_annotation","context":c})).unwrap()["annotation"].clone()
}
fn update(
    b: &mut DesktopBridge,
    session: &Value,
    text: &str,
    x: &str,
    y: &str,
    exact: bool,
) -> Result<Value, ecad_bridge::BridgeError> {
    call(
        b,
        json!({"type":"preview_annotation","session":session,"placement":{"text":text,"x":x,"y":y,"exact":exact,"suppressed":false,"candidate":null,"tolerance_um":3000}}),
    )
}
fn action(b: &mut DesktopBridge, action: &str, input: &str) -> Value {
    let c = context(b);
    call(
        b,
        json!({"type":"action","context":c,"action":action,"input":input}),
    )
    .unwrap()
}
#[test]
fn created_annotation_matches_preview_and_presets_with_one_step_history_and_saved_identity() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("annotation.json");
    let mut original = DesktopBridge::demo().unwrap();
    original.save_as(&path).unwrap();
    let mut data: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    data["drawing"]["settings"]["text_creation"]["overrides"]["font"] = json!("monospace");
    data["drawing"]["settings"]["text_creation"]["overrides"]["text_height_um"] = json!(4100);
    data["drawing"]["settings"]["text_creation"]["overrides"]["colour"] = json!([32, 64, 96]);
    std::fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    let mut b = DesktopBridge::from_path(&path).unwrap();
    let before = snapshot(&b);
    let seed = begin(&mut b);
    assert_eq!(seed["valid"], false);
    assert_eq!(seed["scene"], json!([]));
    let reviewed = update(
        &mut b,
        &seed["session"],
        "<switch> Ω",
        "10000",
        "-5000",
        true,
    )
    .unwrap()["annotation"]
        .clone();
    let row = &reviewed["scene"][0];
    assert_eq!(row["font"], "monospace");
    assert_eq!(row["text_height"], 4.1);
    assert_eq!(row["colour"], "#204060");
    assert_eq!(snapshot(&b), before);
    assert!(!b.is_dirty());
    let committed = call(
        &mut b,
        json!({"type":"commit_annotation","token":reviewed["token"],"input":"1"}),
    )
    .unwrap();
    assert!(
        committed["snapshot"]["scene"]
            .as_array()
            .unwrap()
            .contains(row)
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
    assert_eq!(snapshot(&loaded)["scene"], snapshot(&b)["scene"]);
    let saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let stored = saved["drawing"]["labels"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == row["target"]["id"])
        .unwrap();
    assert_eq!(
        stored["appearance"],
        data["drawing"]["settings"]["text_creation"]
    );
    assert_eq!(stored["text"], "<switch> Ω");
}
#[test]
fn malformed_blank_superseded_cancelled_and_stale_proposals_cannot_create_old_text() {
    let mut b = DesktopBridge::demo().unwrap();
    let before = snapshot(&b);
    let seed = begin(&mut b);
    let good =
        update(&mut b, &seed["session"], "text", "0", "0", true).unwrap()["annotation"].clone();
    assert!(update(&mut b, &seed["session"], "  ", "0", "0", true).is_err());
    assert!(
        call(
            &mut b,
            json!({"type":"commit_annotation","token":good["token"],"input":"1"})
        )
        .is_err()
    );
    let good =
        update(&mut b, &seed["session"], "fixed", "0", "0", true).unwrap()["annotation"].clone();
    assert!(call(&mut b,json!({"type":"preview_annotation","session":seed["session"],"placement":{"unexpected":true}})).is_err());
    assert!(
        call(
            &mut b,
            json!({"type":"commit_annotation","token":good["token"],"input":"2"})
        )
        .is_err()
    );
    let good =
        update(&mut b, &seed["session"], "cancel", "0", "0", true).unwrap()["annotation"].clone();
    assert_eq!(
        action(&mut b, "placement.cancel", "3")["message"],
        "Annotation cancelled"
    );
    assert!(
        call(
            &mut b,
            json!({"type":"commit_annotation","token":good["token"],"input":"4"})
        )
        .is_err()
    );
    assert_eq!(snapshot(&b)["scene"], before["scene"]);
    assert_eq!(snapshot(&b)["revision"], before["revision"]);
    assert!(!b.is_dirty());
    let seed = begin(&mut b);
    let good =
        update(&mut b, &seed["session"], "stale", "0", "0", true).unwrap()["annotation"].clone();
    let mut c = context(&b);
    c["selection"] = json!([{"kind":"label","id":format!("{:032x}",1)}]);
    call(
        &mut b,
        json!({"type":"edit","context":c,"property":"drawing_text","value":"changed"}),
    )
    .unwrap();
    let changed = snapshot(&b);
    assert!(
        call(
            &mut b,
            json!({"type":"commit_annotation","token":good["token"],"input":"5"})
        )
        .is_err()
    );
    assert_eq!(snapshot(&b)["scene"], changed["scene"]);
}
#[test]
fn snapped_position_and_large_numeric_values_are_exact_and_drafts_have_distinct_ids() {
    let mut b = DesktopBridge::demo().unwrap();
    let seed = begin(&mut b);
    let p =
        update(&mut b, &seed["session"], "snapped", "99100", "1000", false).unwrap()["annotation"]
            .clone();
    assert_eq!(p["snap"], "Terminal");
    assert_eq!(p["x"], "100000");
    assert_eq!(p["y"], "0");
    let exact = update(
        &mut b,
        &seed["session"],
        "precise",
        "9007199254740993",
        "-9223372036854775808",
        true,
    )
    .unwrap()["annotation"]
        .clone();
    assert_eq!(exact["x"], "9007199254740993");
    assert_eq!(exact["y"], "-9223372036854775808");
    assert!(
        update(
            &mut b,
            &seed["session"],
            "bad",
            "9223372036854775808",
            "0",
            true
        )
        .is_err()
    );
    let other = begin(&mut b);
    let q = update(&mut b, &other["session"], "new", "0", "0", true).unwrap()["annotation"].clone();
    assert_ne!(p["scene"][0]["target"], q["scene"][0]["target"]);
    assert!(
        call(
            &mut b,
            json!({"type":"commit_annotation","token":p["token"],"input":"1"})
        )
        .is_err()
    );
    call(
        &mut b,
        json!({"type":"commit_annotation","token":q["token"],"input":"2"}),
    )
    .unwrap();
    assert_eq!(
        call(
            &mut b,
            json!({"type":"commit_annotation","token":q["token"],"input":"2"})
        )
        .unwrap_err()
        .code,
        "replayed"
    );
}
