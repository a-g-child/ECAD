//! Paper layout persistence, exact document history and explicit legacy upgrades.
use ecad_adapters::*;
use ecad_app::*;
use ecad_domain::*;
use serde_json::{Value, json};
fn sheet(id: u128, number: &str) -> DrawingSheet {
    DrawingSheet {
        id: SheetId::new(id).unwrap(),
        number: number.into(),
        title: "Harness\nDetail".into(),
        paper: SheetPaper::default(),
        placement: Some(SheetViewPlacement {
            view: ViewId::new(1).unwrap(),
            paper_origin: DrawingPoint {
                x: 20_000,
                y: 60_000,
            },
            view_origin: DrawingPoint { x: -5000, y: 1000 },
            numerator: 1,
            denominator: 2,
        }),
    }
}
fn dispatch(
    d: &mut Document<JsonFile>,
    parameters: ActionParameters,
    id: u64,
) -> Result<ActionOutcome, ActionDispatchError<ProjectFileError>> {
    let invocation = ActionInvocation {
        input_id: InputId::new(id).unwrap(),
        action: ActionId::new("drawing.sheets.edit").unwrap(),
        source: InvocationSource::Toolbar,
        repeat: false,
        context: ActionContext::for_session(d.session()),
        parameters,
    };
    ActionDispatcher::default().invoke(&ActionRegistry::builtins(), invocation, Some(d))
}
#[test]
fn reviewed_sheet_creation_saves_and_undoes_without_changing_any_source_record() {
    let directory = tempfile::tempdir().unwrap();
    let file = JsonFile::new(directory.path().join("paper.json"));
    let original = decode_project(include_str!("fixtures/routed-conveyor-v7.json")).unwrap();
    let mut d = Document::new(original.clone(), file.clone());
    d.save().unwrap();
    let preview = preview_sheet(d.session(), sheet(1, "DWG-10"), true).unwrap();
    assert_eq!(d.session().project(), &original);
    dispatch(&mut d, ActionParameters::SheetPreview(preview), 1).unwrap();
    d.save().unwrap();
    assert_eq!(
        Document::open(file.clone()).unwrap().session().project(),
        d.session().project()
    );
    let before: Value = serde_json::from_str(&encode_project(&original).unwrap()).unwrap();
    let mut saved: Value =
        serde_json::from_str(&std::fs::read_to_string(file.path()).unwrap()).unwrap();
    assert_eq!(saved["schema_version"], 10);
    saved["sheets"] = json!([]);
    saved["drawing_documents"] = before["drawing_documents"].clone();
    assert_eq!(saved, before);
    d.undo(d.session().revision()).unwrap();
    assert_eq!(d.session().project(), &original);
    let preview = preview_sheet(d.session(), sheet(2, "Stale"), true).unwrap();
    d.redo(d.session().revision()).unwrap();
    assert!(dispatch(&mut d, ActionParameters::SheetPreview(preview), 2).is_err());
    assert!(!d.session().is_dirty());
}
#[test]
fn sheet_reorder_remove_noop_and_failed_transactions_restore_exact_set_and_geometry() {
    let directory = tempfile::tempdir().unwrap();
    let mut d = Document::new(
        decode_project(include_str!("fixtures/routed-conveyor-v7.json")).unwrap(),
        JsonFile::new(directory.path().join("p.json")),
    );
    for id in 1..=3 {
        d.execute(
            d.session().revision(),
            Command::Sheet(SheetCommand::Insert {
                sheet: sheet(id, &format!("D-{id}")),
                index: (id - 1) as usize,
            }),
        )
        .unwrap();
    }
    d.save().unwrap();
    let saved = d.session().project().clone();
    d.execute(
        d.session().revision(),
        Command::Sheet(SheetCommand::Move {
            id: SheetId::new(3).unwrap(),
            index: 0,
        }),
    )
    .unwrap();
    assert_eq!(d.session().project().sheets()[0].number, "D-3");
    d.undo(d.session().revision()).unwrap();
    let revision = d.session().revision();
    d.execute(
        revision,
        Command::Sheet(SheetCommand::Replace(saved.sheets()[0].clone())),
    )
    .unwrap();
    assert_eq!(d.session().revision(), revision);
    d.redo(revision).unwrap();
    d.undo(d.session().revision()).unwrap();
    assert!(!d.session().is_dirty());
    d.execute(
        d.session().revision(),
        Command::Sheet(SheetCommand::Remove(SheetId::new(2).unwrap())),
    )
    .unwrap();
    d.undo(d.session().revision()).unwrap();
    assert_eq!(d.session().project(), &saved);
    assert!(
        d.execute_batch(
            d.session().revision(),
            vec![
                Command::Sheet(SheetCommand::Insert {
                    sheet: sheet(4, "D-4"),
                    index: 3
                }),
                Command::Sheet(SheetCommand::Replace(sheet(1, "D-2")))
            ]
        )
        .is_err()
    );
    assert_eq!(d.session().project(), &saved);
}
#[test]
fn v8_rejects_invalid_paper_and_references_and_legacy_projects_receive_no_invented_sheets() {
    for text in [
        include_str!("fixtures/minimal-v2.json"),
        include_str!("fixtures/minimal-v3.json"),
        include_str!("fixtures/minimal-v4.json"),
        include_str!("fixtures/minimal-v5.json"),
        include_str!("fixtures/routed-conveyor-v6.json"),
        include_str!("fixtures/routed-conveyor-v7.json"),
    ] {
        let p = decode_project(text).unwrap();
        assert!(p.sheets().is_empty());
        assert_eq!(decode_project(&encode_project(&p).unwrap()).unwrap(), p);
    }
    let mut p = decode_project(include_str!("fixtures/routed-conveyor-v7.json")).unwrap();
    p.insert_sheet(sheet(1, "A"), 0).unwrap();
    let text = encode_project(&p).unwrap();
    assert_eq!(
        encode_project(&decode_project(&text).unwrap()).unwrap(),
        text
    );
    let data: Value = serde_json::from_str(&text).unwrap();
    for (pointer, value) in [
        ("/sheets/0/paper/width_um", json!(0)),
        ("/sheets/0/paper/margins_um", json!([297000, 0, 0, 0])),
        ("/sheets/0/placement/scale", json!([1, 0])),
        ("/sheets/0/placement/view", json!(format!("{:032x}", 999))),
        ("/sheets/0/number", json!(" ")),
    ] {
        let mut bad = data.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(decode_project(&bad.to_string()).is_err());
    }
    let mut duplicate = data.clone();
    duplicate["sheets"]
        .as_array_mut()
        .unwrap()
        .push(data["sheets"][0].clone());
    assert!(decode_project(&duplicate.to_string()).is_err());
    let mut unknown = data.clone();
    unknown["sheets"][0]["future"] = json!(true);
    assert!(decode_project(&unknown.to_string()).is_err());
    let mut body = Project::new(ProjectId::new(1).unwrap());
    let root = SubsystemId::new(1).unwrap();
    body.insert_subsystem(Subsystem::new(root, "Assembly", None).unwrap())
        .unwrap();
    body.insert_sheet(
        DrawingSheet {
            placement: None,
            ..sheet(1, "A")
        },
        0,
    )
    .unwrap();
    assert!(matches!(
        SubsystemRevision::new(
            DefinitionRef::new(
                DefinitionId::new(1).unwrap(),
                DefinitionRevisionId::new(1).unwrap()
            ),
            root,
            body
        ),
        Err(ModelError::Sheet(SheetError::ReusableSheetsUnsupported))
    ));
}
