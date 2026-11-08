//! Project properties and typed page drafts use the registry, strict DTOs and ordinary history.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn snap(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn key(b: &DesktopBridge, label: &str) -> Value {
    snap(b)["template_choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| {
            v["label"]
                .as_str()
                .unwrap()
                .starts_with(&format!("Legacy {label} /"))
        })
        .unwrap()["key"]
        .clone()
}
fn ctx(b: &DesktopBridge) -> Value {
    json!({"revision":snap(b)["revision"],"view":null,"sheet":null,"selection":[],"focus":"tree"})
}
fn call(b: &mut DesktopBridge, v: Value) -> Result<Value, ecad_bridge::BridgeError> {
    b.handle(serde_json::from_value::<Request>(v).unwrap())
        .map(|r| serde_json::to_value(r).unwrap())
}
fn action(b: &mut DesktopBridge, id: &str) -> Value {
    let v = json!({"type":"action","context":ctx(b),"action":id,"input":snap(b)["next_input"]});
    call(b, v).unwrap()
}
fn seed(b: &mut DesktopBridge, kind: &str) -> Value {
    let v = json!({"type":"new_sheet","context":ctx(b),"kind":kind});
    call(b, v).unwrap()["sheet_draft"].clone()
}
fn preview(b: &mut DesktopBridge, row: &Value, create: bool) -> Value {
    let v = json!({"type":"preview_sheet","context":ctx(b),"sheet":row,"create":create});
    call(b, v).unwrap()["sheet_preview"].clone()
}
fn commit(b: &mut DesktopBridge, p: &Value) -> Value {
    let v = json!({"type":"commit_sheet","token":p["token"],"input":snap(b)["next_input"]});
    call(b, v).unwrap()
}
#[test]
fn typed_creation_is_reviewed_and_source_view_is_created_only_with_the_schematic() {
    let mut b = DesktopBridge::demo().unwrap();
    let before = snap(&b);
    assert_eq!(
        action(&mut b, "drawing.sheet.new_schematic")["request"],
        "new_schematic"
    );
    let row = seed(&mut b, "schematic");
    let view = row["view"].clone();
    assert_eq!(row["template"], key(&b, "Standard drawing"));
    assert!(!view.is_null());
    assert_eq!(snap(&b)["views"], before["views"]);
    let p = preview(&mut b, &row, true);
    assert_eq!(snap(&b)["revision"], before["revision"]);
    call(&mut b, json!({"type":"cancel_sheet"})).unwrap();
    let bad = json!({"type":"commit_sheet","token":p["token"],"input":snap(&b)["next_input"]});
    assert!(call(&mut b, bad).is_err());
    assert_eq!(snap(&b)["views"], before["views"]);
    let row = seed(&mut b, "schematic");
    let p = preview(&mut b, &row, true);
    let result = commit(&mut b, &p);
    assert_eq!(result["activate_sheet"], row["id"]);
    assert!(
        snap(&b)["views"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["id"] == row["view"])
    );
    action(&mut b, "document.undo");
    assert_eq!(snap(&b)["views"], before["views"]);
    assert_eq!(snap(&b)["sheets"], before["sheets"]);
    action(&mut b, "document.redo");
    assert!(
        snap(&b)["views"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["id"] == row["view"])
    );
    let mut cover = seed(&mut b, "cover");
    assert!(cover["view"].is_null());
    assert_eq!(cover["template"], key(&b, "Cover page"));
    cover["view"] = view;
    let v = json!({"type":"preview_sheet","context":ctx(&b),"sheet":cover,"create":true});
    assert!(call(&mut b, v).is_err());
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("replacement.json");
    std::fs::write(
        &file,
        ecad_adapters::encode_project(&ecad_domain::Project::new(
            ecad_domain::ProjectId::new(1).unwrap(),
        ))
        .unwrap(),
    )
    .unwrap();
    let mut reopened = DesktopBridge::demo().unwrap();
    let old = seed(&mut reopened, "schematic");
    reopened.open(&file).unwrap();
    let request =
        json!({"type":"preview_sheet","context":ctx(&reopened),"sheet":old,"create":true});
    assert!(call(&mut reopened, request).is_err());
    assert_eq!(snap(&reopened)["views"], json!([]));
    assert!(!reopened.is_dirty());
}
#[test]
fn project_template_defaults_and_metadata_apply_atomically_and_invalid_values_leave_both_unchanged()
{
    let mut b = DesktopBridge::demo().unwrap();
    let before = snap(&b);
    let props = action(&mut b, "document.metadata.edit");
    assert_eq!(
        props["project_templates"]["defaults"]["cover"],
        key(&b, "Cover page")
    );
    let v = json!({"type":"update_metadata","context":ctx(&b),"values":{"company.name":"Company\nName"},"template_defaults":{"schematic":key(&b,"Cover page"),"cover":key(&b,"Standard drawing"),"drawing_list":key(&b,"Drawing list")},"input":snap(&b)["next_input"]});
    assert!(
        call(&mut b, v).unwrap()["message"]
            .as_str()
            .unwrap()
            .starts_with("Project properties updated")
    );
    let current = snap(&b);
    assert_eq!(current["sheets"], before["sheets"]);
    assert_eq!(current["scene"], before["scene"]);
    let legacy = json!({"type":"new_sheet","context":ctx(&b)});
    assert_eq!(
        call(&mut b, legacy).unwrap()["sheet_draft"]["template"],
        key(&b, "Cover page")
    );
    let row = seed(&mut b, "cover");
    assert_eq!(row["template"], key(&b, "Standard drawing"));
    let v = json!({"type":"update_metadata","context":ctx(&b),"values":{"company.name":"Must not apply"},"template_defaults":{"schematic":"unknown","cover":key(&b,"Cover page"),"drawing_list":key(&b,"Drawing list")},"input":snap(&b)["next_input"]});
    assert!(call(&mut b, v).is_err());
    assert_eq!(snap(&b)["revision"], current["revision"]);
    action(&mut b, "document.undo");
    assert_eq!(
        action(&mut b, "document.metadata.edit")["project_templates"]["defaults"],
        props["project_templates"]["defaults"]
    );
    assert_eq!(snap(&b)["sheets"], before["sheets"]);
    let old = ctx(&b);
    action(&mut b, "document.redo");
    let v = json!({"type":"new_sheet","context":old,"kind":"cover"});
    assert!(call(&mut b, v).is_err());
}
#[test]
fn generated_list_uses_real_sheet_numbers_titles_and_stable_destinations_and_refreshes_after_edits()
{
    let mut b = DesktopBridge::demo().unwrap();
    let mut cover = seed(&mut b, "cover");
    cover["number"] = json!("10");
    cover["title"] = json!("Cover <literal>\nTitle");
    let p = preview(&mut b, &cover, true);
    commit(&mut b, &p);
    let mut schematic = seed(&mut b, "schematic");
    schematic["number"] = json!("2");
    let p = preview(&mut b, &schematic, true);
    commit(&mut b, &p);
    let list = seed(&mut b, "drawing-list");
    assert!(list["view"].is_null());
    let p = preview(&mut b, &list, true);
    assert_eq!(p["list"]["rows"][0]["sheet"], schematic["id"]);
    assert_eq!(p["list"]["rows"][1]["sheet"], cover["id"]);
    assert_eq!(p["list"]["rows"][1]["title"], cover["title"]);
    commit(&mut b, &p);
    assert_eq!(snap(&b)["drawing_lists"][0], p["list"]);
    schematic["number"] = json!("20");
    schematic["title"] = json!("Renamed");
    let p = preview(&mut b, &schematic, false);
    commit(&mut b, &p);
    assert_eq!(
        snap(&b)["drawing_lists"][0]["rows"][0]["sheet"],
        cover["id"]
    );
    assert_eq!(
        snap(&b)["drawing_lists"][0]["rows"][1]["sheet"],
        schematic["id"]
    );
    assert_eq!(snap(&b)["drawing_lists"][0]["rows"][1]["title"], "Renamed");
}
