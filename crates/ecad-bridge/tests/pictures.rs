//! Pictures use captured contexts, validated assets and coherent document history.
use ecad_bridge::{DesktopBridge, Request};
use serde_json::{Value, json};
fn send(b: &mut DesktopBridge, v: Value) -> Result<Value, ecad_bridge::BridgeError> {
    b.handle(serde_json::from_value::<Request>(v).unwrap())
        .map(|r| serde_json::to_value(r).unwrap())
}
fn snapshot(b: &DesktopBridge) -> Value {
    serde_json::to_value(b.snapshot()).unwrap()["snapshot"].clone()
}
fn context(b: &DesktopBridge, selection: Value) -> Value {
    json!({"revision":snapshot(b)["revision"],"selection":selection,"view":format!("{:032x}",1),"focus":"canvas"})
}
#[test]
fn import_is_transient_and_placement_edits_guard_targets_revisions_and_history() {
    let mut b = DesktopBridge::demo().unwrap();
    let before = snapshot(&b);
    let png = ecad_adapters::picture_base64(
        &ecad_adapters::import_png(
            include_bytes!("../../../apps/desktop/tests/fixtures/picture.png").to_vec(),
        )
        .unwrap(),
    );
    let imported = send(&mut b, json!({"type":"import_picture","png":png})).unwrap();
    assert_eq!(snapshot(&b), before);
    assert!(send(&mut b, json!({"type":"import_picture","png":"bad"})).is_err());
    let values = json!({"bounds":["1000","2000","40000","20000"],"picture":imported["imported_picture"]["picture"]});
    let ctx = context(&b, json!([]));
    let created = send(
        &mut b,
        json!({"type":"edit_picture","context":ctx,"id":null,"values":values,"input":"1"}),
    )
    .unwrap();
    let row = created["snapshot"]["scene"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["target"]["kind"] == "picture")
        .unwrap();
    let target = row["target"].clone();
    let id = target["id"].clone();
    assert_eq!(
        send(
            &mut b,
            json!({"type":"edit_picture","context":ctx,"id":null,"values":values,"input":"2"})
        )
        .unwrap_err()
        .code,
        "stale"
    );
    let revision = snapshot(&b)["revision"].clone();
    let ctx = context(&b, json!([]));
    assert!(
        send(
            &mut b,
            json!({"type":"edit_picture","context":ctx,"id":id,"values":null,"input":"3"})
        )
        .is_err()
    );
    assert_eq!(snapshot(&b)["revision"], revision);
    let ctx = context(&b, json!([target]));
    let mut invalid = values.clone();
    invalid["bounds"][2] = json!("0");
    assert!(
        send(
            &mut b,
            json!({"type":"edit_picture","context":ctx,"id":id,"values":invalid,"input":"4"})
        )
        .is_err()
    );
    assert_eq!(snapshot(&b)["revision"], revision);
    send(
        &mut b,
        json!({"type":"edit_picture","context":ctx,"id":id,"values":values,"input":"5"}),
    )
    .unwrap();
    assert_eq!(
        snapshot(&b)["revision"],
        revision,
        "unchanged picture is a no-op"
    );
    send(
        &mut b,
        json!({"type":"edit_picture","context":ctx,"id":id,"values":null,"input":"6"}),
    )
    .unwrap();
    assert_eq!(snapshot(&b)["scene"], before["scene"]);
    let ctx = context(&b, json!([]));
    send(
        &mut b,
        json!({"type":"action","context":ctx,"action":"document.undo","input":"7"}),
    )
    .unwrap();
    assert_eq!(snapshot(&b)["scene"], created["snapshot"]["scene"]);
}

#[test]
fn extended_imports_return_verified_mime_and_forged_formats_cannot_edit() {
    let mut b = DesktopBridge::demo().unwrap();
    for (index, (bytes, mime)) in [
        (
            include_bytes!("../../../apps/desktop/tests/fixtures/picture.jpg").as_slice(),
            "image/jpeg",
        ),
        (
            include_bytes!("../../../apps/desktop/tests/fixtures/picture.svg").as_slice(),
            "image/svg+xml",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let data =
            ecad_adapters::picture_base64(&ecad_adapters::import_picture(bytes.to_vec()).unwrap());
        let imported = send(&mut b, json!({"type":"import_picture","data":data})).unwrap();
        assert_eq!(imported["imported_picture"]["picture"]["mime"], mime);
        let mut values = json!({"bounds":["0","0","40000","20000"],"picture":imported["imported_picture"]["picture"]});
        values["picture"]["mime"] = "image/png".into();
        let ctx = context(&b, json!([]));
        let before = snapshot(&b);
        assert!(
            send(
                &mut b,
                json!({"type":"edit_picture","context":ctx,"id":null,"values":values,"input":(index+1).to_string()})
            )
            .is_err()
        );
        assert_eq!(snapshot(&b)["scene"], before["scene"]);
        assert_eq!(snapshot(&b)["revision"], before["revision"]);
        assert!(!b.is_dirty());
    }
}
