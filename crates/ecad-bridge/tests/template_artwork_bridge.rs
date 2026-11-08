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
fn artwork_properties_history_validation_publication_and_table_edits_are_coherent() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = bridge(&dir);
    let before = variant(&b);
    let g = json!({"id":"00000000000000000000000000000001","kind":"text","bounds":["1000","1000","80000","20000"],"contents":[{"kind":"literal","value":"Title\n"},{"kind":"field","value":"sheet.title"}],"points":[],"font":"sans-serif","text_height":"3500","stroke_width":"250","colour":"#112233"});
    let mut values = properties(&b);
    values["graphics"] = json!([g.clone()]);
    apply(&mut b, "template.properties.apply", values).unwrap();
    let art = variant(&b);
    assert_eq!(art["graphics"], json!([g]));
    action(&mut b, "document.undo").unwrap();
    assert_eq!(variant(&b), before);
    action(&mut b, "document.redo").unwrap();
    assert_eq!(variant(&b), art);
    let mut values = properties(&b);
    values["graphics"] = art["graphics"].clone();
    values["graphics"][0]["contents"][1]["value"] = "future.key".into();
    assert!(apply(&mut b, "template.properties.apply", values).is_err());
    assert_eq!(variant(&b), art);
    let v = variant(&b);
    let values = json!({"variant":v["id"],"table":v["tables"][0]["id"],"edit":{"operation":"resize","bounds":["107000","159000","180000","40000"]}});
    apply(&mut b, "template.table.resize", values).unwrap();
    assert_eq!(variant(&b)["graphics"], art["graphics"]);
    let values = properties(&b);
    apply(&mut b, "template.properties.apply", values).unwrap();
    assert_eq!(variant(&b)["graphics"], art["graphics"]);
    action(&mut b, "document.save").unwrap();
    let disk = std::fs::read_to_string(dir.path().join("templates.json")).unwrap();
    let catalog = ecad_adapters::decode_template_library(&disk).unwrap();
    assert!(
        catalog
            .revisions()
            .any(|r| r.variants().any(|v| !v.layout().graphics().is_empty()))
    );
}
