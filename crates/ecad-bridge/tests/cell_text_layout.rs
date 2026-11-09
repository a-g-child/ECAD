//! Captured block-layout edits validate mixed selections atomically and restore exact styles.
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
        .map(|v| serde_json::to_value(v).unwrap())
}
fn action(b: &mut DesktopBridge, id: &str) {
    let req =
        json!({"type":"action","context":context(b),"action":id,"input":snapshot(b)["next_input"]});
    call(b, req).unwrap();
}
fn variant(b: &DesktopBridge) -> Value {
    snapshot(b)["template_editor"]["variants"][0].clone()
}
fn apply(
    b: &mut DesktopBridge,
    ctx: Value,
    values: Value,
) -> Result<Value, ecad_bridge::BridgeError> {
    let req = json!({"type":"template_operation","context":ctx,"action":"template.properties.apply","values":values,"source":null,"input":snapshot(b)["next_input"]});
    call(b, req)
}
#[test]
fn layout_range_edits_reject_invalid_or_stale_proposals_and_undo_exactly() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = DesktopBridge::demo().unwrap();
    b.configure_template_store(dir.path().join("templates.json"))
        .unwrap();
    action(&mut b, "library.templates");
    action(&mut b, "template.new");
    let before = variant(&b);
    let captured = context(&b);
    let mut values = json!({"variant":before["id"],"name":before["name"],"paper":before["paper"],"content":before["content"],"cell":null,"cell_styles":[]});
    let mut edits = vec![];
    for c in before["tables"][0]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .take(2)
    {
        let mut styles = c["text_styles"].clone();
        styles["layout"] = json!({"caption_position":"inline","separator":" : ","horizontal":"center","vertical":"middle"});
        edits.push(json!({"id":c["id"],"styles":styles}));
    }
    values["cell_styles"] = json!(edits);
    let mut invalid = values.clone();
    invalid["cell_styles"][1]["styles"]["layout"]["separator"] = json!("bad\nseparator");
    assert!(apply(&mut b, captured.clone(), invalid).is_err());
    assert_eq!(variant(&b), before);
    apply(&mut b, captured.clone(), values.clone()).unwrap();
    let after = variant(&b);
    for c in after["tables"][0]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .take(2)
    {
        assert_eq!(c["text_styles"]["layout"]["horizontal"], "center");
        assert_eq!(c["text_styles"]["layout"]["separator"], " : ");
    }
    assert!(apply(&mut b, captured, values.clone()).is_err());
    assert_eq!(variant(&b), after);
    action(&mut b, "document.undo");
    assert_eq!(variant(&b), before);
    action(&mut b, "document.redo");
    assert_eq!(variant(&b), after);
}
