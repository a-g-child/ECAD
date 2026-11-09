//! Strict DTO/context validation, real property/history edits, and guarded file lifecycle.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn request(bridge: &mut DesktopBridge, value: Value) -> Result<Value, ecad_bridge::BridgeError> {
    bridge
        .handle(serde_json::from_value::<Request>(value).unwrap())
        .map(|v| serde_json::to_value(v).unwrap())
}
fn snapshot(bridge: &DesktopBridge) -> Value {
    serde_json::to_value(bridge.snapshot()).unwrap()["snapshot"].clone()
}
fn target(kind: &str, id: u128) -> Value {
    json!({"kind":kind,"id":format!("{id:032x}")})
}
fn context(bridge: &DesktopBridge, selection: Value) -> Value {
    json!({"revision":snapshot(bridge)["revision"],"selection":selection,"view":format!("{:032x}",1),"focus":"canvas"})
}
fn action(
    bridge: &mut DesktopBridge,
    id: &str,
    input: &str,
) -> Result<Value, ecad_bridge::BridgeError> {
    let ctx = context(bridge, json!([]));
    request(
        bridge,
        json!({"type":"action","context":ctx,"action":id,"input":input}),
    )
}
#[test]
fn sample_snapshot_and_full_union_inspector_have_real_ids_styles_and_exact_values() {
    let mut bridge = DesktopBridge::demo().unwrap();
    let s = snapshot(&bridge);
    assert_eq!(s["bridge_version"], 1);
    assert!(!bridge.is_dirty());
    assert_eq!(s["scene"].as_array().unwrap().len(), 6);
    let ctx = context(&bridge, json!([target("route", 1), target("label", 1)]));
    let inspected = request(&mut bridge, json!({"type":"inspect","context":ctx})).unwrap();
    let rows = inspected["properties"].as_array().unwrap();
    let colour = rows.iter().find(|r| r["key"] == "drawing_colour").unwrap();
    assert_eq!(colour["applicable"], 2);
    assert_eq!(colour["mixed"], true);
    let width = rows.iter().find(|r| r["key"] == "stroke_width").unwrap();
    assert_eq!(width["applicable"], 1);
    assert_eq!(width["unsupported"], 1);
    assert_eq!(width["value"], "500");
    assert!(
        rows.iter()
            .any(|r| r["key"] == "route_conductor" && r["restricted"] == 1)
    );
    assert_eq!(snapshot(&bridge), s);
}
#[test]
fn mixed_property_edit_is_one_real_undo_step_and_stale_or_invalid_edits_preserve_state() {
    let mut bridge = DesktopBridge::demo().unwrap();
    let ctx = context(&bridge, json!([target("route", 1), target("label", 1)]));
    let before = snapshot(&bridge);
    let edited = request(
        &mut bridge,
        json!({"type":"edit","context":ctx,"property":"drawing_colour","value":"#a02030"}),
    )
    .unwrap();
    assert!(bridge.is_dirty());
    for kind in ["route", "label"] {
        assert_eq!(
            edited["snapshot"]["scene"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["target"] == target(kind, 1))
                .unwrap()["colour"],
            "#a02030"
        );
    }
    assert_eq!(
        request(
            &mut bridge,
            json!({"type":"edit","context":ctx,"property":"drawing_colour","value":"#ffffff"})
        )
        .unwrap_err()
        .code,
        "stale"
    );
    let ctx = context(&bridge, json!([target("route", 1)]));
    let snapshot_before = snapshot(&bridge);
    assert!(
        request(
            &mut bridge,
            json!({"type":"edit","context":ctx,"property":"stroke_width","value":"0"})
        )
        .is_err()
    );
    assert_eq!(snapshot(&bridge), snapshot_before);
    let undone = action(&mut bridge, "document.undo", "1").unwrap();
    assert_eq!(undone["snapshot"]["scene"], before["scene"]);
    let redone = action(&mut bridge, "document.redo", "2").unwrap();
    assert_eq!(redone["snapshot"]["scene"], edited["snapshot"]["scene"]);
}
#[test]
fn save_as_and_open_use_guarded_documents_and_keep_original_state_on_failures() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sample.json");
    let mut bridge = DesktopBridge::demo().unwrap();
    bridge.save_as(&path).unwrap();
    assert!(!bridge.is_dirty());
    assert!(!bridge.needs_save_as());
    let before = snapshot(&bridge);
    let mut loaded = DesktopBridge::from_path(&path).unwrap();
    assert_eq!(snapshot(&loaded)["scene"], before["scene"]);
    assert_ne!(snapshot(&loaded)["revision"], before["revision"]);
    let ctx = context(&bridge, json!([target("route", 1)]));
    request(
        &mut bridge,
        json!({"type":"edit","context":ctx,"property":"stroke_width","value":"700"}),
    )
    .unwrap();
    let edited = snapshot(&bridge);
    assert_eq!(bridge.open(&path).unwrap_err().code, "dirty");
    assert_eq!(snapshot(&bridge), edited);
    let bytes = std::fs::read(&path).unwrap();
    std::fs::write(&path, b"externally changed").unwrap();
    assert!(action(&mut bridge, "document.save", "1").is_err());
    assert_eq!(snapshot(&bridge)["scene"], edited["scene"]);
    assert!(bridge.is_dirty());
    std::fs::write(&path, bytes).unwrap();
    action(&mut bridge, "document.save", "2").unwrap();
    assert!(!bridge.is_dirty());
    loaded.open(&path).unwrap();
    assert_eq!(snapshot(&loaded)["scene"], snapshot(&bridge)["scene"]);
    assert!(bridge.open(&directory.path().join("missing.json")).is_err());
    assert_eq!(snapshot(&bridge)["scene"], edited["scene"]);
}
#[test]
fn shared_input_stream_rejects_replay_even_for_save_as_requests_and_resynchronizes_host_counter() {
    let mut bridge = DesktopBridge::demo().unwrap();
    assert_eq!(
        action(&mut bridge, "document.save", "9007199254740993").unwrap()["request"],
        "save_as"
    );
    assert_eq!(
        action(&mut bridge, "document.save", "9007199254740993")
            .unwrap_err()
            .code,
        "replayed"
    );
    assert_eq!(snapshot(&bridge)["next_input"], "9007199254740994");
    let mut ctx = context(&bridge, json!([]));
    ctx["focus"] = json!("text");
    let before = snapshot(&bridge);
    let reply=request(&mut bridge,json!({"type":"key","context":ctx,"key":"z","control":true,"meta":true,"alt":false,"shift":false,"repeat":false,"composing":true,"input":"9007199254740994","now_ms":1})).unwrap();
    assert!(reply["snapshot"].is_null());
    assert_eq!(snapshot(&bridge)["revision"], before["revision"]);
}
#[test]
fn strict_context_and_cross_session_tokens_cannot_address_stale_or_foreign_objects() {
    let bridge = DesktopBridge::demo().unwrap();
    let mut other = DesktopBridge::demo().unwrap();
    let ctx = context(&bridge, json!([target("anchor", 1)]));
    assert_eq!(
        request(&mut other, json!({"type":"inspect","context":ctx}))
            .unwrap_err()
            .code,
        "stale"
    );
    let mut ctx = context(&other, json!([]));
    ctx["unexpected"] = json!(true);
    assert!(request(&mut other, json!({"type":"inspect","context":ctx})).is_err());
    let ctx = context(&other, json!([{"kind":"device","id":"1"}]));
    assert!(request(&mut other, json!({"type":"inspect","context":ctx})).is_err());
    assert!(serde_json::from_value::<Request>(json!({"type":"snapshot","extra":1})).is_err());
}

#[test]
fn exact_i64_properties_and_u128_identities_never_cross_the_bridge_as_lossy_js_numbers() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("exact.json");
    let mut bridge = DesktopBridge::demo().unwrap();
    bridge.save_as(&path).unwrap();
    let mut data: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    data["drawing"]["labels"][0]["id"] = json!(format!("{:032x}", u128::MAX));
    data["drawing"]["labels"][0]["position_um"] = json!([i64::MAX, i64::MIN]);
    std::fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    let mut exact = DesktopBridge::from_path(&path).unwrap();
    let s = snapshot(&exact);
    assert!(
        s["objects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["target"] == target("label", u128::MAX))
    );
    let ctx = context(&exact, json!([target("label", u128::MAX)]));
    let reply = request(&mut exact, json!({"type":"inspect","context":ctx})).unwrap();
    let position = reply["properties"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["key"] == "drawing_position")
        .unwrap();
    assert_eq!(position["value"], format!("{}, {}", i64::MAX, i64::MIN));
}

#[test]
fn compound_font_edits_and_defaults_are_atomic_and_restore_exact_properties() {
    let mut bridge = DesktopBridge::demo().unwrap();
    let selection = json!([target("label", 1), target("route", 1)]);
    let inspect = |bridge: &mut DesktopBridge| {
        let ctx = context(bridge, selection.clone());
        request(bridge, json!({"type":"inspect","context":ctx})).unwrap()["properties"].clone()
    };
    let original = inspect(&mut bridge);
    let ctx = context(&bridge, selection.clone());
    request(
        &mut bridge,
        json!({"type":"edit_appearance","context":ctx,
        "changes":{"font_family":"Custom\nFamily","text_height":"4200"}}),
    )
    .unwrap();
    let explicit = inspect(&mut bridge);
    for (key, value) in [("font_family", "Custom\nFamily"), ("text_height", "4200")] {
        let row = explicit
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["key"] == key)
            .unwrap();
        assert_eq!(row["value"], value);
        assert_eq!(row["applicable"], 1);
        assert_eq!(row["unsupported"], 1);
    }
    assert_eq!(
        request(
            &mut bridge,
            json!({"type":"edit_appearance","context":ctx,
        "changes":{"font_family":null}})
        )
        .unwrap_err()
        .code,
        "stale"
    );
    let ctx = context(&bridge, selection.clone());
    let before = snapshot(&bridge);
    for changes in [
        json!({"font_family":"Arial","text_height":"0"}),
        json!({"drawing_colour":null}),
    ] {
        assert!(
            request(
                &mut bridge,
                json!({"type":"edit_appearance","context":ctx,"changes":changes})
            )
            .is_err()
        );
        assert_eq!(snapshot(&bridge), before);
    }
    request(
        &mut bridge,
        json!({"type":"edit_appearance","context":ctx,
        "changes":{"font_family":null,"text_height":null}}),
    )
    .unwrap();
    let defaults = inspect(&mut bridge);
    for key in ["font_family", "text_height"] {
        let row = defaults
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["key"] == key)
            .unwrap();
        assert_eq!(row["origin"], "Project defaults");
    }
    action(&mut bridge, "document.undo", "1").unwrap();
    assert_eq!(inspect(&mut bridge), explicit);
    action(&mut bridge, "document.undo", "2").unwrap();
    assert_eq!(inspect(&mut bridge), original);
    action(&mut bridge, "document.redo", "3").unwrap();
    assert_eq!(inspect(&mut bridge), explicit);
}
