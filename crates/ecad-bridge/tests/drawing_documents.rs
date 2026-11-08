//! The existing sheet editor preserves new document ownership, including exact removal undo.
use ecad_bridge::{DesktopBridge, Request};
use ecad_domain::*;
use serde_json::{Value, json};
fn snap(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn context(b: &DesktopBridge) -> Value {
    json!({"revision":snap(b)["revision"],"selection":[],"view":format!("{:032x}",1),"focus":"canvas"})
}
fn call(b: &mut DesktopBridge, value: Value) -> Value {
    serde_json::to_value(
        b.handle(serde_json::from_value::<Request>(value).unwrap())
            .unwrap(),
    )
    .unwrap()
}
fn action(b: &mut DesktopBridge, id: &str) {
    let value =
        json!({"type":"action","context":context(b),"action":id,"input":snap(b)["next_input"]});
    call(b, value);
}
#[test]
fn bridge_projects_packages_and_preserves_nondefault_ownership_during_sheet_edit_save_and_undo() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("documents.json");
    let mut p = ecad_adapters::decode_project(include_str!(
        "../../ecad-adapters/tests/fixtures/routed-conveyor-v8.json"
    ))
    .unwrap();
    let owner = DrawingDocumentId::new(2).unwrap();
    p.insert_drawing_document(
        ProjectDrawingDocument {
            id: owner,
            number: "WIRING".into(),
            title: "Harness\nDrawings".into(),
            revision: "A".into(),
            default_template: ecad_domain::SheetTemplate::StandardV1,
        },
        1,
    )
    .unwrap();
    p.insert_sheet_in_document(
        DrawingSheet {
            kind: ecad_domain::SheetKind::Schematic,
            template: None,
            id: SheetId::new(1).unwrap(),
            number: "D-1".into(),
            title: "Detail".into(),
            paper: SheetPaper::default(),
            placement: Some(SheetViewPlacement {
                view: ViewId::new(1).unwrap(),
                paper_origin: DrawingPoint {
                    x: 20_000,
                    y: 60_000,
                },
                view_origin: DrawingPoint::default(),
                numerator: 1,
                denominator: 1,
            }),
        },
        owner,
        0,
    )
    .unwrap();
    std::fs::write(&path, ecad_adapters::encode_project(&p).unwrap()).unwrap();
    let mut b = DesktopBridge::from_path(&path).unwrap();
    let before = snap(&b);
    assert_eq!(before["drawing_documents"][1]["id"], owner.to_string());
    assert_eq!(
        before["drawing_documents"][1]["sheets"],
        json!([format!("{:032x}", 1)])
    );
    let mut sheet = before["sheets"][0].clone();
    sheet["title"] = json!("Edited");
    let request =
        json!({"type":"preview_sheet","context":context(&b),"sheet":sheet,"create":false});
    let preview = call(&mut b, request)["sheet_preview"].clone();
    let request =
        json!({"type":"commit_sheet","token":preview["token"],"input":snap(&b)["next_input"]});
    call(&mut b, request);
    action(&mut b, "document.save");
    assert_eq!(snap(&b)["drawing_documents"], before["drawing_documents"]);
    assert_eq!(snap(&b)["scene"], before["scene"]);
    let request = json!({"type":"change_sheet","context":context(&b),"id":format!("{:032x}",1),"operation":"remove","index":0,"input":snap(&b)["next_input"]});
    call(&mut b, request);
    assert_eq!(snap(&b)["drawing_documents"][1]["sheets"], json!([]));
    action(&mut b, "document.undo");
    assert_eq!(snap(&b)["drawing_documents"], before["drawing_documents"]);
    assert!(!b.is_dirty());
    let reopened = DesktopBridge::from_path(&path).unwrap();
    assert_eq!(
        snap(&reopened)["drawing_documents"],
        before["drawing_documents"]
    );
}
