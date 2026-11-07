//! Profile IO stays outside document undo; reviewed copies retain revision and exact field intent.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn snap(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn ctx(b: &DesktopBridge) -> Value {
    json!({"revision":snap(b)["revision"],"selection":[],"view":null,"focus":"workspace"})
}
fn call(b: &mut DesktopBridge, value: Value) -> Result<Value, ecad_bridge::BridgeError> {
    b.handle(serde_json::from_value::<Request>(value).unwrap())
        .map(|r| serde_json::to_value(r).unwrap())
}
fn action(b: &mut DesktopBridge, id: &str) -> Value {
    let v = json!({"type":"action","context":ctx(b),"action":id,"input":snap(b)["next_input"]});
    call(b, v).unwrap()
}
fn load(b: &mut DesktopBridge) -> Value {
    call(b, json!({"type":"load_profiles"})).unwrap()["profiles"].clone()
}
fn save(
    b: &mut DesktopBridge,
    c: &Value,
    id: Value,
    name: &str,
    values: Value,
) -> Result<Value, ecad_bridge::BridgeError> {
    let v = json!({"type":"save_profile","token":c["token"],"id":id,"name":name,"values":values,"input":snap(b)["next_input"]});
    call(b, v).map(|r| r["profiles"].clone())
}
fn prepare(
    b: &mut DesktopBridge,
    c: &Value,
    fields: Value,
) -> Result<Value, ecad_bridge::BridgeError> {
    let v = json!({"type":"preview_profile_import","context":ctx(b),"token":c["token"],"id":c["entries"][0]["id"],"fields":fields});
    call(b, v).map(|r| r["profile_import"].clone())
}
fn commit(b: &mut DesktopBridge, p: &Value) -> Result<Value, ecad_bridge::BridgeError> {
    let v =
        json!({"type":"commit_profile_import","token":p["token"],"input":snap(b)["next_input"]});
    call(b, v)
}
fn edit(b: &mut DesktopBridge, values: Value) {
    let v = json!({"type":"update_metadata","context":ctx(b),"values":values,"input":snap(b)["next_input"]});
    call(b, v).unwrap();
}
fn value(b: &mut DesktopBridge, key: &str) -> Value {
    action(b, "document.metadata.edit")["metadata"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["key"] == key)
        .unwrap()["value"]
        .clone()
}
#[test]
fn saved_profiles_do_not_dirty_documents_and_selected_copies_save_undo_and_reopen_without_live_links()
 {
    let directory = tempfile::tempdir().unwrap();
    let project = directory.path().join("project.json");
    let profiles = directory.path().join("profiles.json");
    let mut b = DesktopBridge::demo().unwrap();
    b.configure_profile_store(&profiles);
    b.save_as(&project).unwrap();
    edit(
        &mut b,
        json!({"company.address":"Keep project address","project.title":"Harness","project.issue":"B"}),
    );
    action(&mut b, "document.save");
    let before = snap(&b);
    let c = action(&mut b, "workspace.profiles")["profiles"].clone();
    assert!(!profiles.exists());
    let c=save(&mut b,&c,Value::Null,"Company",json!({"company.name":"Example","company.address":"New\nAddress","engineer.name":"Engineer"})).unwrap();
    assert_eq!(snap(&b)["revision"], before["revision"]);
    assert!(!b.is_dirty());
    let p = prepare(&mut b, &c, json!(["company.name"])).unwrap();
    assert_eq!(p["changes"].as_array().unwrap().len(), 1);
    assert_eq!(snap(&b)["scene"], before["scene"]);
    commit(&mut b, &p).unwrap();
    assert_eq!(value(&mut b, "company.name"), "Example");
    assert_eq!(value(&mut b, "company.address"), "Keep project address");
    assert_eq!(value(&mut b, "project.issue"), "B");
    action(&mut b, "document.save");
    action(&mut b, "document.undo");
    assert_eq!(value(&mut b, "company.name"), "");
    action(&mut b, "document.redo");
    assert!(!b.is_dirty());
    let mut other = DesktopBridge::from_path(&project).unwrap();
    other.configure_profile_store(&profiles);
    let c = load(&mut other);
    save(
        &mut other,
        &c,
        c["entries"][0]["id"].clone(),
        "Renamed",
        json!({"company.name":"Changed personal default"}),
    )
    .unwrap();
    assert!(!other.is_dirty());
    assert_eq!(value(&mut b, "company.name"), "Example");
    assert_eq!(value(&mut other, "company.name"), "Example");
    assert_eq!(snap(&b)["scene"], before["scene"]);
}
#[test]
fn imports_invalidate_wrong_proposals_reject_stale_documents_and_preserve_redo_for_noops_and_empty_selection()
 {
    let directory = tempfile::tempdir().unwrap();
    let mut b = DesktopBridge::demo().unwrap();
    b.configure_profile_store(directory.path().join("profiles.json"));
    let c = load(&mut b);
    let c = save(
        &mut b,
        &c,
        Value::Null,
        "Company",
        json!({"company.name":"Example"}),
    )
    .unwrap();
    let p = prepare(&mut b, &c, json!(["company.name"])).unwrap();
    assert!(prepare(&mut b, &c, json!(["project.issue"])).is_err());
    assert!(commit(&mut b, &p).is_err());
    let p = prepare(&mut b, &c, json!(["company.name"])).unwrap();
    edit(
        &mut b,
        json!({"project.title":"Intervening edit","company.address":"Keep"}),
    );
    assert!(commit(&mut b, &p).is_err());
    action(&mut b, "document.undo");
    let revision = snap(&b)["revision"].clone();
    let p = prepare(&mut b, &c, json!([])).unwrap();
    assert_eq!(p["changes"], json!([]));
    commit(&mut b, &p).unwrap();
    assert_eq!(snap(&b)["revision"], revision);
    action(&mut b, "document.redo");
    assert_eq!(value(&mut b, "company.address"), "Keep");
    let p = prepare(&mut b, &c, json!(["company.address"])).unwrap();
    assert_eq!(p["changes"][0]["after"], Value::Null);
    commit(&mut b, &p).unwrap();
    assert_eq!(value(&mut b, "company.address"), "");
    action(&mut b, "document.undo");
    assert_eq!(value(&mut b, "company.address"), "Keep");
    let cancelled = prepare(&mut b, &c, json!(["company.name"])).unwrap();
    call(&mut b, json!({"type":"cancel_profile_import"})).unwrap();
    assert!(commit(&mut b, &cancelled).is_err());
    let p = prepare(&mut b, &c, json!(["company.name"])).unwrap();
    let _ = load(&mut b);
    assert!(commit(&mut b, &p).is_err());
}
#[test]
fn external_profile_changes_and_failed_refresh_cannot_authorize_overwrite_or_mutate_project_history()
 {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("profiles.json");
    let mut a = DesktopBridge::demo().unwrap();
    a.configure_profile_store(&path);
    let empty = load(&mut a);
    let first = save(
        &mut a,
        &empty,
        Value::Null,
        "First",
        json!({"company.name":"A"}),
    )
    .unwrap();
    let mut b = DesktopBridge::demo().unwrap();
    b.configure_profile_store(&path);
    let old = load(&mut b);
    let before = snap(&b)["revision"].clone();
    let updated = save(
        &mut a,
        &first,
        first["entries"][0]["id"].clone(),
        "Updated",
        json!({"company.name":"Updated"}),
    )
    .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(
        save(
            &mut b,
            &old,
            old["entries"][0]["id"].clone(),
            "Stale",
            json!({"company.name":"Stale"})
        )
        .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(snap(&b)["revision"], before);
    let fresh = load(&mut b);
    assert_eq!(fresh["entries"], updated["entries"]);
    assert!(save(&mut b, &old, Value::Null, "Stale token", json!({})).is_err());
    std::fs::write(&path, "broken").unwrap();
    assert!(call(&mut b, json!({"type":"load_profiles"})).is_err());
    assert!(save(&mut b, &fresh, Value::Null, "Unsafe", json!({})).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"broken");
    std::fs::write(&path, bytes).unwrap();
    let recovered = load(&mut b);
    assert!(
        save(
            &mut b,
            &recovered,
            Value::Null,
            "Rejected scope",
            json!({"project.title":""})
        )
        .is_err()
    );
    assert_eq!(snap(&b)["revision"], before);
}
