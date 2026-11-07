//! Closed host previews, exact connected-route edits, and cancellation/revision guards.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn call(b: &mut DesktopBridge, v: Value) -> Result<Value, ecad_bridge::BridgeError> {
    b.handle(serde_json::from_value::<Request>(v).unwrap())
        .map(|r| serde_json::to_value(r).unwrap())
}
fn snap(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn target(kind: &str, id: u128) -> Value {
    json!({"kind":kind,"id":format!("{id:032x}")})
}
fn ctx(b: &DesktopBridge) -> Value {
    json!({"revision":snap(b)["revision"],"view":format!("{:032x}",1),"selection":[target("anchor",1),target("anchor",2)],"focus":"canvas"})
}
fn begin(b: &mut DesktopBridge) -> Value {
    let c = ctx(b);
    call(
        b,
        json!({"type":"begin_move","context":c,"reference":target("anchor",1)}),
    )
    .unwrap()["movement"]
        .clone()
}
fn preview(
    b: &mut DesktopBridge,
    session: &Value,
    x: &str,
    y: &str,
    exact: bool,
) -> Result<Value, ecad_bridge::BridgeError> {
    call(
        b,
        json!({"type":"preview_move","session":session,"x":x,"y":y,"exact":exact,"suppressed":false,"candidate":null,"tolerance_um":3000}),
    )
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
fn connected_group_preview_commits_exact_geometry_once_and_round_trips() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("move.json");
    let mut b = DesktopBridge::demo().unwrap();
    b.save_as(&path).unwrap();
    let before = snap(&b);
    let initial = begin(&mut b);
    let moved =
        preview(&mut b, &initial["session"], "10000", "5000", true).unwrap()["movement"].clone();
    assert_eq!(snap(&b), before);
    assert!(!b.is_dirty());
    // Route one translates every waypoint; route two adjusts only the connected end.
    let route = moved["scene"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["target"] == target("route", 1))
        .unwrap();
    assert_eq!(
        route["points"],
        json!([
            [10.0, 5.0],
            [30.0, 5.0],
            [30.0, 20.0],
            [60.0, 20.0],
            [60.0, 5.0]
        ])
    );
    assert_eq!(moved["scene"].as_array().unwrap().len(), 4);
    let committed = call(
        &mut b,
        json!({"type":"commit_move","token":moved["token"],"input":"1"}),
    )
    .unwrap();
    assert!(b.is_dirty());
    for row in moved["scene"].as_array().unwrap() {
        let actual = committed["snapshot"]["scene"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["target"] == row["target"])
            .unwrap();
        assert_eq!(actual, row);
    }
    assert!(
        call(
            &mut b,
            json!({"type":"commit_move","token":moved["token"],"input":"1"})
        )
        .is_err()
    );
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
}
#[test]
fn invalid_updates_invalidate_old_tokens_and_cancel_preserves_history() {
    let mut b = DesktopBridge::demo().unwrap();
    let before = snap(&b);
    let initial = begin(&mut b);
    let first =
        preview(&mut b, &initial["session"], "10000", "5000", true).unwrap()["movement"].clone();
    assert!(
        preview(
            &mut b,
            &initial["session"],
            "9223372036854775808",
            "5000",
            true
        )
        .is_err()
    );
    assert!(
        call(
            &mut b,
            json!({"type":"commit_move","token":first["token"],"input":"1"})
        )
        .is_err()
    );
    let second =
        preview(&mut b, &initial["session"], "12000", "5000", true).unwrap()["movement"].clone();
    assert_eq!(
        action(&mut b, "placement.cancel", "2")["request"],
        "cancel_move"
    );
    assert!(
        call(
            &mut b,
            json!({"type":"commit_move","token":second["token"],"input":"3"})
        )
        .is_err()
    );
    assert_eq!(snap(&b)["scene"], before["scene"]);
    assert_eq!(snap(&b)["revision"], before["revision"]);
    assert!(!b.is_dirty());
}
#[test]
fn snapped_proposals_supersession_and_changed_document_cannot_commit_old_geometry() {
    let mut b = DesktopBridge::demo().unwrap();
    let first = begin(&mut b);
    // Selected anchors and their touching routes are excluded; remaining terminal wins over grid.
    let p = preview(&mut b, &first["session"], "99100", "1000", false).unwrap()["movement"].clone();
    assert_eq!(p["snap"], "Terminal");
    assert_eq!(p["x"], "100000");
    assert_eq!(p["y"], "0");
    assert!(
        call(
            &mut b,
            json!({"type":"commit_move","token":first["token"],"input":"1"})
        )
        .is_err()
    );
    let mut c = ctx(&b);
    c["selection"] = json!([target("label", 1)]);
    call(
        &mut b,
        json!({"type":"edit","context":c,"property":"drawing_text","value":"changed"}),
    )
    .unwrap();
    let changed = snap(&b);
    assert!(
        call(
            &mut b,
            json!({"type":"commit_move","token":p["token"],"input":"2"})
        )
        .is_err()
    );
    assert_eq!(snap(&b)["scene"], changed["scene"]);
    // New movement has a separate lifetime even at the same revision.
    let a = begin(&mut b);
    let z = begin(&mut b);
    assert_ne!(a["session"], z["session"]);
    assert!(preview(&mut b, &a["session"], "1", "2", true).is_err());
}
#[test]
fn exact_numeric_coordinates_cross_js_safe_integer_range_without_rounding() {
    let mut b = DesktopBridge::demo().unwrap();
    let mut c = ctx(&b);
    c["selection"] = json!([target("label", 1)]);
    let initial = call(
        &mut b,
        json!({"type":"begin_move","context":c,"reference":target("label",1)}),
    )
    .unwrap()["movement"]
        .clone();
    let p = preview(
        &mut b,
        &initial["session"],
        "9007199254740993",
        "-33000",
        true,
    )
    .unwrap()["movement"]
        .clone();
    assert_eq!(p["x"], "9007199254740993");
    call(
        &mut b,
        json!({"type":"commit_move","token":p["token"],"input":"1"}),
    )
    .unwrap();
    let mut c = ctx(&b);
    c["selection"] = json!([target("label", 1)]);
    let properties = call(&mut b, json!({"type":"inspect","context":c})).unwrap();
    assert!(
        properties["properties"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["key"] == "drawing_position" && r["value"] == "9007199254740993, -33000")
    );
}

#[test]
fn frontend_clock_reset_does_not_suppress_first_shortcut_after_reload() {
    let mut b = DesktopBridge::demo().unwrap();
    for (input, now) in [("1", 50000), ("2", 1)] {
        let c = ctx(&b);
        let reply = call(&mut b, json!({"type":"key","context":c,"key":"m","control":false,"meta":false,"alt":false,"shift":false,"repeat":false,"composing":false,"input":input,"now_ms":now})).unwrap();
        assert_eq!(reply["request"], "move");
    }
}
