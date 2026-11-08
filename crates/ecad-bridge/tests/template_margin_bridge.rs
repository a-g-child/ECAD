//! Margin snapping/fitting travel through captured previews, strict DTOs and exact template history.
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
    let r =
        json!({"type":"action","context":context(b),"action":id,"input":snapshot(b)["next_input"]});
    call(b, r)
}
fn variant(b: &DesktopBridge) -> Value {
    snapshot(b)["template_editor"]["variants"][0].clone()
}
fn bridge(dir: &tempfile::TempDir) -> DesktopBridge {
    let mut b = DesktopBridge::demo().unwrap();
    b.configure_template_store(dir.path().join("templates.json"))
        .unwrap();
    action(&mut b, "library.templates").unwrap();
    action(&mut b, "template.new").unwrap();
    b
}
fn table_values(b: &DesktopBridge, edit: Value) -> Value {
    let v = variant(b);
    json!({"variant":v["id"],"table":v["tables"][0]["id"],"edit":edit})
}
fn preview(b: &mut DesktopBridge, edit: Value) -> Result<Value, ecad_bridge::BridgeError> {
    let r = json!({"type":"template_table_preview","context":context(b),"action":"template.table.resize","values":table_values(b,edit),"input":snapshot(b)["next_input"]});
    call(b, r)
}
fn commit(b: &mut DesktopBridge, token: Value) -> Result<Value, ecad_bridge::BridgeError> {
    let r = json!({"type":"template_table_commit","context":context(b),"preview":token,"input":snapshot(b)["next_input"]});
    call(b, r)
}
fn properties(b: &DesktopBridge) -> Value {
    let v = variant(b);
    json!({"variant":v["id"],"name":v["name"],"paper":v["paper"],"content":v["content"],"cell":null})
}
fn apply(
    b: &mut DesktopBridge,
    id: &str,
    values: Value,
) -> Result<Value, ecad_bridge::BridgeError> {
    let r = json!({"type":"template_operation","context":context(b),"action":id,"values":values,"source":null,"input":snapshot(b)["next_input"]});
    call(b, r)
}
#[test]
fn pointer_snaps_report_exact_borders_while_bypassed_and_numeric_geometry_stays_exact() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(&dir);
    let numeric = table_values(
        &b,
        json!({"operation":"resize","bounds":["106500","159400","180000","40000"]}),
    );
    apply(&mut b, "template.table.resize", numeric).unwrap();
    let before = variant(&b);
    assert_eq!(before["tables"][0]["bounds"][0], "106500");
    let bypass=preview(&mut b,json!({"operation":"pointer","drag":{"kind":"move","x":"106500","y":"159400"},"snap_tolerance":null})).unwrap()["table_preview"].clone();
    assert_eq!(bypass["snapped"], json!([]));
    assert_eq!(bypass["tables"], before["tables"]);
    action(&mut b, "placement.cancel").unwrap();
    let p=preview(&mut b,json!({"operation":"pointer","drag":{"kind":"move","x":"106500","y":"159400"},"snap_tolerance":"1000"})).unwrap()["table_preview"].clone();
    assert_eq!(p["snapped"], json!(["right", "bottom"]));
    assert_eq!(
        p["tables"][0]["bounds"],
        json!(["107000", "160000", "180000", "40000"])
    );
    assert_eq!(variant(&b), before);
    commit(&mut b, p["token"].clone()).unwrap();
    assert_eq!(variant(&b)["tables"], p["tables"]);
    action(&mut b, "document.undo").unwrap();
    assert_eq!(variant(&b), before);
    action(&mut b, "document.redo").unwrap();
    let original = variant(&b);
    let p=preview(&mut b,json!({"operation":"pointer","drag":{"kind":"resize","edge":"left","coordinate":"10100"},"snap_tolerance":"200"})).unwrap()["table_preview"].clone();
    assert_eq!(p["snapped"], json!(["left"]));
    assert_eq!(
        p["tables"][0]["bounds"],
        json!(["10000", "160000", "277000", "40000"])
    );
    assert!(preview(&mut b,json!({"operation":"pointer","drag":{"kind":"resize","edge":"right","coordinate":"287000"},"snap_tolerance":"200"})).is_err());
    assert!(commit(&mut b, p["token"].clone()).is_err());
    action(&mut b, "placement.cancel").unwrap();
    assert_eq!(variant(&b), original);
    for bad in [
        json!({"operation":"pointer","drag":{"kind":"move","x":"NaN","y":"159400"},"snap_tolerance":"200"}),
        json!({"operation":"pointer","drag":{"kind":"resize","edge":"left","coordinate":"10000"},"snap_tolerance":"-1"}),
    ] {
        assert!(preview(&mut b, bad).is_err());
        action(&mut b, "placement.cancel").unwrap();
        assert_eq!(variant(&b), original);
    }
}
#[test]
fn explicit_uniform_margin_fit_restores_full_layout_history_and_saved_checkpoint() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(&dir);
    action(&mut b, "document.save").unwrap();
    let original = variant(&b);
    let project = snapshot(&b);
    let mut edit = properties(&b);
    for i in 2..6 {
        edit["paper"][i] = "20000".into();
    }
    assert!(apply(&mut b, "template.properties.apply", edit.clone()).is_err());
    assert_eq!(variant(&b), original);
    edit["fit_margins"] = true.into();
    apply(&mut b, "template.properties.apply", edit).unwrap();
    let fitted = variant(&b);
    assert_eq!(
        fitted["tables"][0]["bounds"],
        json!(["97000", "150000", "180000", "40000"])
    );
    assert_eq!(fitted["tables"][0]["cells"], original["tables"][0]["cells"]);
    action(&mut b, "document.save").unwrap();
    assert!(!b.is_dirty());
    action(&mut b, "document.undo").unwrap();
    assert_eq!(variant(&b), original);
    assert!(b.is_dirty());
    let mut unchanged = properties(&b);
    unchanged["fit_margins"] = true.into();
    apply(&mut b, "template.properties.apply", unchanged).unwrap();
    action(&mut b, "document.redo").unwrap();
    assert_eq!(variant(&b), fitted);
    assert!(!b.is_dirty());
    let mut invalid = properties(&b);
    for i in 2..6 {
        invalid["paper"][i] = "60000".into();
    }
    invalid["fit_margins"] = true.into();
    assert!(apply(&mut b, "template.properties.apply", invalid).is_err());
    assert_eq!(variant(&b), fitted);
    assert!(!b.is_dirty());
    assert_eq!(snapshot(&b)["revision"], project["revision"]);
    assert_eq!(snapshot(&b)["sheets"], project["sheets"]);
    let library = ecad_adapters::decode_template_library(
        &std::fs::read_to_string(dir.path().join("templates.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(library.revisions().len(), 2);
}
