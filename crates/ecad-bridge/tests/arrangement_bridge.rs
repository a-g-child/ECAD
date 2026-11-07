//! Origin alignment/distribution use closed layout proposals with exact connected-route history.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn call(b: &mut DesktopBridge, value: Value) -> Result<Value, ecad_bridge::BridgeError> {
    b.handle(serde_json::from_value::<Request>(value).unwrap())
        .map(|r| serde_json::to_value(r).unwrap())
}
fn snap(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn target(kind: &str, id: u128) -> Value {
    json!({"kind":kind,"id":format!("{id:032x}")})
}
fn ctx(b: &DesktopBridge, ids: &[u128]) -> Value {
    json!({"revision":snap(b)["revision"],"selection":ids.iter().map(|id|target("anchor",*id)).collect::<Vec<_>>(),"view":format!("{:032x}",1),"focus":"canvas"})
}
fn begin(b: &mut DesktopBridge, kind: &str, ids: &[u128]) -> Value {
    let context = ctx(b, ids);
    call(b,json!({"type":"begin_transform","context":context,"reference":target("anchor",ids[0]),"kind":kind})).unwrap()["transform"].clone()
}
fn update(
    b: &mut DesktopBridge,
    seed: &Value,
    x: &str,
    operation: &str,
) -> Result<Value, ecad_bridge::BridgeError> {
    call(b,json!({"type":"preview_transform","session":seed["session"],"x":x,"y":"0","operation":operation})).map(|r|r["transform"].clone())
}
fn action(b: &mut DesktopBridge, action: &str, input: &str) -> Value {
    let context = ctx(b, &[1, 2, 3]);
    call(
        b,
        json!({"type":"action","context":context,"action":action,"input":input}),
    )
    .unwrap()
}
fn row<'a>(scene: &'a Value, kind: &str, id: u128) -> &'a Value {
    scene
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["target"] == target(kind, id))
        .unwrap()
}
#[test]
fn aligning_anchor_origins_preserves_whole_routes_and_one_step_saved_history() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("aligned.json");
    let mut b = DesktopBridge::demo().unwrap();
    b.save_as(&path).unwrap();
    let before = snap(&b);
    let electrical: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let seed = begin(&mut b, "align", &[1, 2, 3]);
    let p = update(&mut b, &seed, "-10000", "align_y").unwrap();
    assert_eq!(p["valid"], true);
    assert_eq!(p["changed"], true);
    assert_eq!(
        row(&p["scene"], "route", 1)["points"],
        json!([
            [0.0, -10.0],
            [20.0, -10.0],
            [20.0, 5.0],
            [50.0, 5.0],
            [50.0, -10.0]
        ])
    );
    for id in 1..=3 {
        assert_eq!(row(&p["scene"], "anchor", id)["points"][0][1], -10.0);
    }
    assert_eq!(snap(&b), before);
    let committed = call(
        &mut b,
        json!({"type":"commit_transform","token":p["token"],"input":"1"}),
    )
    .unwrap();
    for r in p["scene"].as_array().unwrap() {
        assert!(
            committed["snapshot"]["scene"]
                .as_array()
                .unwrap()
                .contains(r)
        );
    }
    action(&mut b, "document.save", "2");
    let loaded = DesktopBridge::from_path(&path).unwrap();
    assert_eq!(snap(&loaded)["scene"], snap(&b)["scene"]);
    let saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for key in ["devices", "conductors", "cables"] {
        assert_eq!(saved[key], electrical[key]);
    }
    assert_eq!(
        action(&mut b, "document.undo", "3")["snapshot"]["scene"],
        before["scene"]
    );
    assert_eq!(
        action(&mut b, "document.redo", "4")["snapshot"]["scene"],
        committed["snapshot"]["scene"]
    );
    assert!(!b.is_dirty());
}
#[test]
fn already_distributed_origins_report_no_change_and_preserve_redo_and_checkpoint() {
    let directory = tempfile::tempdir().unwrap();
    let mut b = DesktopBridge::demo().unwrap();
    b.save_as(&directory.path().join("noop.json")).unwrap();
    let mut context = ctx(&b, &[]);
    context["selection"] = json!([target("label", 1)]);
    call(
        &mut b,
        json!({"type":"edit","context":context,"property":"drawing_text","value":"redo retained"}),
    )
    .unwrap();
    action(&mut b, "document.undo", "1");
    let before = snap(&b);
    let p = begin(&mut b, "distribute", &[1, 2, 3]);
    assert_eq!(p["valid"], true);
    assert_eq!(p["changed"], false);
    let after = call(
        &mut b,
        json!({"type":"commit_transform","token":p["token"],"input":"2"}),
    )
    .unwrap()["snapshot"]
        .clone();
    assert_eq!(before["revision"], after["revision"]);
    assert_eq!(before["scene"], after["scene"]);
    assert!(!b.is_dirty());
    let redone = action(&mut b, "document.redo", "3");
    assert_eq!(
        row(&redone["snapshot"]["scene"], "label", 1)["label"],
        "redo retained"
    );
}
#[test]
fn invalid_initial_alignment_remains_editable_and_cannot_commit_until_corrected() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("straight.json");
    let mut b = DesktopBridge::demo().unwrap();
    b.save_as(&path).unwrap();
    let mut data: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    data["drawing"]["routes"][0]["points_um"] = json!([[0, 0], [50000, 0]]);
    std::fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    let mut b = DesktopBridge::from_path(&path).unwrap();
    let p = begin(&mut b, "align", &[1, 2]);
    assert_eq!(p["valid"], false);
    assert_eq!(p["scene"], json!([]));
    assert!(
        call(
            &mut b,
            json!({"type":"commit_transform","token":p["token"],"input":"1"})
        )
        .is_err()
    );
    // Fix the same retained draft after a rejected invalid-preview commit.
    let fixed = update(&mut b, &p, "-10000", "align_y").unwrap();
    assert_eq!(fixed["valid"], true);
    assert_eq!(
        row(&fixed["scene"], "route", 1)["points"],
        json!([[0.0, -10.0], [50.0, -10.0]])
    );
    call(
        &mut b,
        json!({"type":"commit_transform","token":fixed["token"],"input":"2"}),
    )
    .unwrap();
}
#[test]
fn distribution_uses_original_extrema_and_guards_wrong_operations_superseded_and_stale_proposals() {
    let mut b = DesktopBridge::demo().unwrap();
    let context = ctx(&b, &[1, 2]);
    assert!(call(&mut b,json!({"type":"begin_transform","context":context,"reference":target("anchor",1),"kind":"distribute"})).is_err());
    let context = ctx(&b, &[2]);
    let seed = call(
        &mut b,
        json!({"type":"begin_move","context":context,"reference":target("anchor",2)}),
    )
    .unwrap()["movement"]
        .clone();
    let p=call(&mut b,json!({"type":"preview_move","session":seed["session"],"x":"30000","y":"0","exact":true,"suppressed":false,"candidate":null,"tolerance_um":0})).unwrap()["movement"].clone();
    call(
        &mut b,
        json!({"type":"commit_move","token":p["token"],"input":"1"}),
    )
    .unwrap();
    let moved = snap(&b);
    let seed = begin(&mut b, "distribute", &[1, 2, 3]);
    assert_eq!(
        row(&seed["scene"], "anchor", 1)["points"],
        json!([[0.0, 0.0]])
    );
    assert_eq!(
        row(&seed["scene"], "anchor", 2)["points"],
        json!([[50.0, 0.0]])
    );
    assert_eq!(
        row(&seed["scene"], "anchor", 3)["points"],
        json!([[100.0, 0.0]])
    );
    assert!(update(&mut b, &seed, "0", "align_x").is_err());
    assert!(
        call(
            &mut b,
            json!({"type":"commit_transform","token":seed["token"],"input":"2"})
        )
        .is_err()
    );
    let reviewed = update(&mut b, &seed, "0", "distribute_x").unwrap();
    call(
        &mut b,
        json!({"type":"commit_transform","token":reviewed["token"],"input":"3"}),
    )
    .unwrap();
    assert_eq!(
        action(&mut b, "document.undo", "4")["snapshot"]["scene"],
        moved["scene"]
    );
    let stale = begin(&mut b, "distribute", &[1, 2, 3]);
    let mut context = ctx(&b, &[]);
    context["selection"] = json!([target("label", 1)]);
    call(
        &mut b,
        json!({"type":"edit","context":context,"property":"drawing_text","value":"changed"}),
    )
    .unwrap();
    let changed = snap(&b);
    assert!(
        call(
            &mut b,
            json!({"type":"commit_transform","token":stale["token"],"input":"5"})
        )
        .is_err()
    );
    assert_eq!(snap(&b)["scene"], changed["scene"]);
}
