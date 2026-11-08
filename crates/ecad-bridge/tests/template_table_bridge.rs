//! Real document/history/publication boundaries for captured structural table edits.
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
        json!({"type":"action","action":id,"context":context(b),"input":snapshot(b)["next_input"]});
    call(b, v)
}
fn variant(b: &DesktopBridge) -> Value {
    snapshot(b)["template_editor"]["variants"][0].clone()
}
fn table(b: &DesktopBridge) -> Value {
    variant(b)["tables"][0].clone()
}
fn request(b: &DesktopBridge, id: &str, edit: Value) -> Value {
    json!({"type":"template_operation","context":context(b),"action":id,"input":snapshot(b)["next_input"],"source":null,"values":{"variant":variant(b)["id"],"table":table(b)["id"],"edit":edit}})
}
fn edit(b: &mut DesktopBridge, id: &str, value: Value) -> Result<Value, ecad_bridge::BridgeError> {
    let v = request(b, id, value);
    call(b, v)
}
fn pending(b: &mut DesktopBridge, id: Option<&str>) -> Result<Value, ecad_bridge::BridgeError> {
    let v = json!({"type":"template_pending","context":context(b),"table_action":id,"input":snapshot(b)["next_input"]});
    call(b, v)
}
#[test]
fn structural_edits_validate_capture_exact_history_and_publish_without_changing_project() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("templates.json");
    let mut b = DesktopBridge::demo().unwrap();
    b.configure_template_store(&path).unwrap();
    let project = snapshot(&b);
    action(&mut b, "library.templates").unwrap();
    assert_eq!(
        action(&mut b, "template.cell.split").unwrap()["request"],
        "template_cell_split"
    );
    let original = variant(&b);
    let first = table(&b)["cells"][0]["id"].clone();
    let stale = request(
        &b,
        "template.cell.split",
        json!({"operation":"split","cell":first,"columns":"2","rows":"1"}),
    );
    pending(&mut b, Some("template.cell.split")).unwrap();
    assert!(b.is_dirty());
    assert!(action(&mut b, "document.save").is_err());
    let before = snapshot(&b)["template_editor"].clone();
    assert!(
        edit(
            &mut b,
            "template.cell.split",
            json!({"operation":"split","cell":first,"columns":"0","rows":"1"})
        )
        .is_err()
    );
    assert_eq!(snapshot(&b)["template_editor"], before);
    edit(
        &mut b,
        "template.cell.split",
        json!({"operation":"split","cell":first,"columns":"3","rows":"2"}),
    )
    .unwrap();
    let split = variant(&b);
    assert_eq!(split["tables"][0]["cells"].as_array().unwrap().len(), 13);
    assert_eq!(split["tables"][0]["cells"][0]["id"], first);
    assert_eq!(snapshot(&b)["template_editor"]["pending"], false);
    let mut stale = stale;
    stale["input"] = snapshot(&b)["next_input"].clone();
    assert!(call(&mut b, stale).is_err());
    action(&mut b, "document.undo").unwrap();
    assert_eq!(variant(&b), original);
    action(&mut b, "document.redo").unwrap();
    assert_eq!(variant(&b), split);
    let cells = table(&b)["cells"].as_array().unwrap()[..6]
        .iter()
        .map(|c| c["id"].clone())
        .collect::<Vec<_>>();
    edit(
        &mut b,
        "template.cells.merge",
        json!({"operation":"merge","cells":cells,"primary":first,"contents":"combine"}),
    )
    .unwrap();
    assert_eq!(variant(&b), original);
    action(&mut b, "document.undo").unwrap();
    assert_eq!(variant(&b), split);
    action(&mut b, "document.redo").unwrap();
    let bounds = table(&b)["bounds"].clone();
    edit(
        &mut b,
        "template.table.resize",
        json!({"operation":"resize","bounds":["100000","150000","180001","40001"]}),
    )
    .unwrap();
    let resized = variant(&b);
    assert_ne!(table(&b)["bounds"], bounds);
    let x = table(&b)["cells"][0]["bounds"][2].clone();
    edit(
        &mut b,
        "template.table.boundary",
        json!({"operation":"boundary","axis":"x","from":x,"to":"80000"}),
    )
    .unwrap();
    let boundary = variant(&b);
    assert_ne!(boundary, resized);
    action(&mut b, "document.undo").unwrap();
    assert_eq!(variant(&b), resized);
    action(&mut b, "document.redo").unwrap();
    assert_eq!(variant(&b), boundary);
    action(&mut b, "document.save").unwrap();
    assert!(!b.is_dirty());
    let library =
        ecad_adapters::decode_template_library(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert!(
        library
            .revisions()
            .flat_map(|r| r.variants())
            .any(|v| v.layout().tables()[0].cells()[0].bounds.width == 80000)
    );
    assert_eq!(snapshot(&b)["revision"], project["revision"]);
    assert_eq!(snapshot(&b)["sheets"], project["sheets"]);
    assert_eq!(snapshot(&b)["scene"], project["scene"]);
}
#[test]
fn invalid_payloads_pending_ownership_and_missing_targets_preserve_the_complete_draft() {
    let dir = tempfile::tempdir().unwrap();
    let mut b = DesktopBridge::demo().unwrap();
    b.configure_template_store(dir.path().join("templates.json"))
        .unwrap();
    action(&mut b, "library.templates").unwrap();
    let before = snapshot(&b)["template_editor"].clone();
    let first = table(&b)["cells"][0]["id"].clone();
    for (id, value) in [
        (
            "template.table.resize",
            json!({"operation":"resize","bounds":["4294967295","0","180000","40000"]}),
        ),
        (
            "template.cell.split",
            json!({"operation":"split","cell":first,"columns":"4097","rows":"1"}),
        ),
        (
            "template.cell.split",
            json!({"operation":"split","cell":first,"columns":"2.5","rows":"1"}),
        ),
        (
            "template.cell.split",
            json!({"operation":"resize","bounds":["0","0","1","1"]}),
        ),
        (
            "template.cell.split",
            json!({"operation":"split","cell":first,"columns":"2","rows":"1","unknown":true}),
        ),
        (
            "template.cells.merge",
            json!({"operation":"merge","cells":[first,first],"primary":first,"contents":"combine"}),
        ),
        (
            "template.table.boundary",
            json!({"operation":"boundary","axis":"x","from":"0","to":"5"}),
        ),
    ] {
        assert!(edit(&mut b, id, value).is_err());
        assert_eq!(snapshot(&b)["template_editor"], before);
    }
    let split = json!({"operation":"split","cell":first,"columns":"2","rows":"1"});
    pending(&mut b, None).unwrap();
    assert!(edit(&mut b, "template.cell.split", split.clone()).is_err());
    assert!(pending(&mut b, Some("template.cell.split")).is_err());
    action(&mut b, "template.properties.revert").unwrap();
    pending(&mut b, Some("template.table.resize")).unwrap();
    assert!(action(&mut b, "template.properties.apply").is_err());
    assert!(edit(&mut b, "template.cell.split", split).is_err());
    assert!(action(&mut b, "template.cell.split").is_err());
    action(&mut b, "template.properties.revert").unwrap();
    assert_eq!(snapshot(&b)["template_editor"], before);
    assert!(!b.is_dirty());
}
