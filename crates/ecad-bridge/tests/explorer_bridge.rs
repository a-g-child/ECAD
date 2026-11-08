//! Explorer requests are transient, revision-guarded actions; grouping reflects real document identity.
use ecad_bridge::{DesktopBridge, Request};
use ecad_domain::*;
use serde_json::{Value, json};
fn snapshot(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn context(s: &Value) -> Value {
    json!({"revision":s["revision"],"selection":[],"view":null,"sheet":null,"focus":"tree"})
}
#[test]
fn modes_share_the_dispatcher_preserve_document_history_and_reject_replayed_or_stale_requests() {
    let mut b = DesktopBridge::demo().unwrap();
    let before = snapshot(&b);
    for (mode, request) in [
        ("pages", "explorer_pages"),
        ("electrical", "explorer_electrical"),
    ] {
        let s = snapshot(&b);
        let value = json!({"type":"action","action":format!("workspace.explorer.{mode}"),"context":context(&s),"input":s["next_input"]});
        let reply = serde_json::to_value(
            b.handle(serde_json::from_value::<Request>(value.clone()).unwrap())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(reply["request"], request);
        assert!(
            b.handle(serde_json::from_value::<Request>(value).unwrap())
                .is_err()
        );
        assert_eq!(snapshot(&b)["revision"], before["revision"]);
        assert_eq!(snapshot(&b)["scene"], before["scene"]);
        assert!(!b.is_dirty());
    }
    let s = snapshot(&b);
    let edit = json!({"type":"update_metadata","context":context(&s),"values":{"project.title":"Changed"},"input":s["next_input"]});
    b.handle(serde_json::from_value::<Request>(edit).unwrap())
        .unwrap();
    let input = snapshot(&b)["next_input"].clone();
    let stale = json!({"type":"action","action":"workspace.explorer.pages","context":context(&before),"input":input});
    assert!(
        b.handle(serde_json::from_value::<Request>(stale).unwrap())
            .is_err()
    );
    assert!(b.is_dirty());
}
#[test]
fn default_group_hint_depends_on_exact_metadata_and_identity_not_just_its_title_or_order() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("groups.json");
    let mut project = Project::new(ProjectId::new(1).unwrap());
    let initial = project.drawing_documents()[0].clone();
    project
        .insert_drawing_document(
            ProjectDrawingDocument {
                id: DrawingDocumentId::new(2).unwrap(),
                number: "2".into(),
                ..initial.clone()
            },
            0,
        )
        .unwrap();
    std::fs::write(&path, ecad_adapters::encode_project(&project).unwrap()).unwrap();
    let s = snapshot(&DesktopBridge::from_path(&path).unwrap());
    assert_eq!(s["drawing_documents"][0]["is_default"], false);
    assert_eq!(s["drawing_documents"][1]["is_default"], true);
    project
        .replace_drawing_document(ProjectDrawingDocument {
            title: "Harness package".into(),
            ..initial
        })
        .unwrap();
    std::fs::write(&path, ecad_adapters::encode_project(&project).unwrap()).unwrap();
    let s = snapshot(&DesktopBridge::from_path(&path).unwrap());
    assert!(
        s["drawing_documents"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["is_default"] == false)
    );
}
