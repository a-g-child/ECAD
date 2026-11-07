//! Exact transform projections, guarded lifetimes and normal document/history integration.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn call(b: &mut DesktopBridge, value: Value) -> Result<Value, ecad_bridge::BridgeError> {
    b.handle(serde_json::from_value::<Request>(value).unwrap())
        .map(|r| serde_json::to_value(r).unwrap())
}
fn snapshot(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn target(kind: &str, id: u128) -> Value {
    json!({"kind":kind,"id":format!("{id:032x}")})
}
fn context(b: &DesktopBridge) -> Value {
    json!({"revision":snapshot(b)["revision"],"selection":[target("anchor",1),target("anchor",2)],"view":format!("{:032x}",1),"focus":"canvas"})
}
fn begin(b: &mut DesktopBridge, kind: &str) -> Value {
    let c = context(b);
    call(
        b,
        json!({"type":"begin_transform","context":c,"reference":target("anchor",1),"kind":kind}),
    )
    .unwrap()["transform"]
        .clone()
}
fn update(
    b: &mut DesktopBridge,
    session: &Value,
    x: &str,
    y: &str,
    operation: &str,
) -> Result<Value, ecad_bridge::BridgeError> {
    call(
        b,
        json!({"type":"preview_transform","session":session,"x":x,"y":y,"operation":operation}),
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
fn rotation_previews_full_internal_routes_and_commits_exactly_once_with_round_trip_history() {
    let mut b = DesktopBridge::demo().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("rotated.json");
    b.save_as(&path).unwrap();
    let before = snapshot(&b);
    let preview = begin(&mut b, "rotate");
    let route = preview["scene"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["target"] == target("route", 1))
        .unwrap();
    assert_eq!(
        route["points"],
        json!([
            [0.0, 0.0],
            [0.0, 20.0],
            [-15.0, 20.0],
            [-15.0, 50.0],
            [0.0, 50.0]
        ])
    );
    assert_eq!(snapshot(&b), before);
    assert!(!b.is_dirty());
    let reviewed =
        update(&mut b, &preview["session"], "10000", "5000", "rotate_180").unwrap()["transform"]
            .clone();
    let committed = call(
        &mut b,
        json!({"type":"commit_transform","token":reviewed["token"],"input":"1"}),
    )
    .unwrap();
    for row in reviewed["scene"].as_array().unwrap() {
        assert!(
            committed["snapshot"]["scene"]
                .as_array()
                .unwrap()
                .contains(row)
        );
    }
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
}
#[test]
fn invalid_overflow_wrong_kind_and_cancelled_proposals_cannot_commit_previous_geometry() {
    let mut b = DesktopBridge::demo().unwrap();
    let before = snapshot(&b);
    let seed = begin(&mut b, "mirror");
    assert!(update(&mut b, &seed["session"], "0", "0", "rotate_90").is_err());
    assert!(
        call(
            &mut b,
            json!({"type":"commit_transform","token":seed["token"],"input":"1"})
        )
        .is_err()
    );
    let valid =
        update(&mut b, &seed["session"], "25000", "0", "mirror_vertical").unwrap()["transform"]
            .clone();
    let route = valid["scene"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["target"] == target("route", 1))
        .unwrap();
    assert_eq!(
        route["points"],
        json!([
            [50.0, 0.0],
            [30.0, 0.0],
            [30.0, 15.0],
            [0.0, 15.0],
            [0.0, 0.0]
        ])
    );
    assert!(
        update(
            &mut b,
            &seed["session"],
            "9223372036854775807",
            "0",
            "mirror_vertical"
        )
        .is_err()
    );
    assert!(
        call(
            &mut b,
            json!({"type":"commit_transform","token":valid["token"],"input":"2"})
        )
        .is_err()
    );
    let fresh =
        update(&mut b, &seed["session"], "25000", "0", "mirror_horizontal").unwrap()["transform"]
            .clone();
    assert_eq!(
        action(&mut b, "placement.cancel", "3")["message"],
        "Transform cancelled"
    );
    assert!(
        call(
            &mut b,
            json!({"type":"commit_transform","token":fresh["token"],"input":"4"})
        )
        .is_err()
    );
    assert_eq!(snapshot(&b)["scene"], before["scene"]);
    assert_eq!(snapshot(&b)["revision"], before["revision"]);
    assert!(!b.is_dirty());
}
#[test]
fn changed_documents_other_tool_lifetimes_and_replayed_commits_are_guarded() {
    let mut b = DesktopBridge::demo().unwrap();
    let seed = begin(&mut b, "rotate");
    let mut c = context(&b);
    c["selection"] = json!([target("label", 1)]);
    call(
        &mut b,
        json!({"type":"edit","context":c,"property":"drawing_text","value":"new text"}),
    )
    .unwrap();
    let edited = snapshot(&b);
    assert!(
        call(
            &mut b,
            json!({"type":"commit_transform","token":seed["token"],"input":"1"})
        )
        .is_err()
    );
    assert_eq!(snapshot(&b)["scene"], edited["scene"]);
    let seed = begin(&mut b, "mirror");
    let c = context(&b);
    call(
        &mut b,
        json!({"type":"begin_move","context":c,"reference":target("anchor",1)}),
    )
    .unwrap();
    assert!(
        call(
            &mut b,
            json!({"type":"commit_transform","token":seed["token"],"input":"2"})
        )
        .is_err()
    );
    let seed = begin(&mut b, "mirror");
    call(
        &mut b,
        json!({"type":"commit_transform","token":seed["token"],"input":"3"}),
    )
    .unwrap();
    assert_eq!(
        call(
            &mut b,
            json!({"type":"commit_transform","token":seed["token"],"input":"3"})
        )
        .unwrap_err()
        .code,
        "replayed"
    );
}

#[test]
fn pivot_coordinates_beyond_javascript_safe_integers_remain_exact() {
    let mut b = DesktopBridge::demo().unwrap();
    let mut c = context(&b);
    c["selection"] = json!([target("label", 1)]);
    let initial = call(
        &mut b,
        json!({"type":"begin_transform","context":c,"reference":target("label",1),"kind":"mirror"}),
    )
    .unwrap()["transform"]
        .clone();
    let reviewed = update(
        &mut b,
        &initial["session"],
        "9007199254740993",
        "-33000",
        "mirror_vertical",
    )
    .unwrap()["transform"]
        .clone();
    assert_eq!(reviewed["x"], "9007199254740993");
    assert!(
        call(
            &mut b,
            json!({"type":"commit_transform","token":initial["token"],"input":"1"})
        )
        .is_err()
    );
    call(
        &mut b,
        json!({"type":"commit_transform","token":reviewed["token"],"input":"2"}),
    )
    .unwrap();
    let mut c = context(&b);
    c["selection"] = json!([target("label", 1)]);
    let inspected = call(&mut b, json!({"type":"inspect","context":c})).unwrap();
    assert!(
        inspected["properties"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["key"] == "drawing_position" && r["value"] == "18014398509481986, -33000")
    );
}
