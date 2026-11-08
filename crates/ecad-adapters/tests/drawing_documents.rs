//! Document-owned schema, exact history and deterministic legacy ownership migration.
use ecad_adapters::*;
use ecad_app::*;
use ecad_domain::*;
use serde_json::{Value, json};
fn doc(id: u128, number: &str) -> ProjectDrawingDocument {
    ProjectDrawingDocument {
        id: DrawingDocumentId::new(id).unwrap(),
        number: number.into(),
        title: format!("Package {number}\nTitle"),
        revision: "A".into(),
        default_template: ecad_domain::SheetTemplate::StandardV1,
    }
}
fn sheet(id: u128) -> DrawingSheet {
    DrawingSheet {
        inherited_variant: None,
        kind: ecad_domain::SheetKind::Schematic,
        template: None,
        id: SheetId::new(id).unwrap(),
        number: format!("D-{id}"),
        title: "Drawing".into(),
        paper: SheetPaper::default(),
        placement: Some(SheetViewPlacement {
            view: ViewId::new(1).unwrap(),
            paper_origin: DrawingPoint {
                x: 20_000,
                y: 60_000,
            },
            view_origin: DrawingPoint { x: -2000, y: 3000 },
            numerator: 1,
            denominator: 2,
        }),
    }
}
fn setup() -> Project {
    let mut p = decode_project(include_str!("fixtures/routed-conveyor-v8.json")).unwrap();
    let a = p.drawing_documents()[0].id;
    p.insert_sheet_in_document(sheet(1), a, 0).unwrap();
    p.insert_sheet_in_document(sheet(2), a, 1).unwrap();
    p
}
fn execute(d: &mut Document<JsonFile>, command: DrawingDocumentCommand) {
    d.execute(d.session().revision(), Command::DrawingDocument(command))
        .unwrap();
}
#[test]
fn ownership_metadata_and_order_roundtrip_and_undo_exactly_without_rewriting_source_records() {
    let directory = tempfile::tempdir().unwrap();
    let file = JsonFile::new(directory.path().join("documents.json"));
    let mut d = Document::new(setup(), file.clone());
    d.save().unwrap();
    let before = d.session().project().clone();
    execute(
        &mut d,
        DrawingDocumentCommand::Insert {
            document: doc(2, "2"),
            index: 1,
        },
    );
    execute(
        &mut d,
        DrawingDocumentCommand::MoveSheet {
            sheet: SheetId::new(2).unwrap(),
            document: DrawingDocumentId::new(2).unwrap(),
            index: 0,
        },
    );
    execute(
        &mut d,
        DrawingDocumentCommand::Move {
            id: DrawingDocumentId::new(2).unwrap(),
            index: 0,
        },
    );
    execute(
        &mut d,
        DrawingDocumentCommand::Replace(ProjectDrawingDocument {
            title: "Updated\nDocument".into(),
            revision: "B".into(),
            ..doc(2, "2")
        }),
    );
    d.save().unwrap();
    let saved = d.session().project().clone();
    assert_eq!(
        Document::open(file.clone()).unwrap().session().project(),
        &saved
    );
    assert_eq!(saved.drawing(), before.drawing());
    let data: Value = serde_json::from_str(&std::fs::read_to_string(file.path()).unwrap()).unwrap();
    assert_eq!(data["schema_version"], 13);
    assert_eq!(
        data["drawing_documents"][0]["sheets"],
        json!([format!("{:032x}", 2)])
    );
    assert_eq!(data["sheets"][0]["id"], format!("{:032x}", 1));
    for _ in 0..4 {
        d.undo(d.session().revision()).unwrap();
    }
    assert_eq!(d.session().project(), &before);
    for _ in 0..4 {
        d.redo(d.session().revision()).unwrap();
    }
    assert_eq!(d.session().project(), &saved);
    assert!(!d.session().is_dirty());
    d.execute(
        d.session().revision(),
        Command::Sheet(SheetCommand::Remove(SheetId::new(2).unwrap())),
    )
    .unwrap();
    d.undo(d.session().revision()).unwrap();
    assert_eq!(d.session().project(), &saved);
}
#[test]
fn legacy_upgrades_use_stable_default_package_and_keep_shared_or_unplaced_views_without_copying() {
    for text in [
        include_str!("fixtures/minimal-v2.json"),
        include_str!("fixtures/minimal-v3.json"),
        include_str!("fixtures/minimal-v4.json"),
        include_str!("fixtures/minimal-v5.json"),
        include_str!("fixtures/routed-conveyor-v6.json"),
        include_str!("fixtures/routed-conveyor-v7.json"),
        include_str!("fixtures/routed-conveyor-v8.json"),
    ] {
        let a = decode_project(text).unwrap();
        let b = decode_project(text).unwrap();
        assert_eq!(
            a.drawing_documents(),
            &[ProjectDrawingDocument::initial(a.id())]
        );
        assert_eq!(a, b);
        assert_eq!(encode_project(&a).unwrap(), encode_project(&b).unwrap());
        assert_eq!(decode_project(&encode_project(&a).unwrap()).unwrap(), a);
    }
    let mut legacy: Value =
        serde_json::from_str(include_str!("fixtures/routed-conveyor-v8.json")).unwrap();
    let current: Value = serde_json::from_str(&encode_project(&setup()).unwrap()).unwrap();
    legacy["sheets"] = current["sheets"].clone();
    for sheet in legacy["sheets"].as_array_mut().unwrap() {
        sheet.as_object_mut().unwrap().remove("kind");
        sheet.as_object_mut().unwrap().remove("template");
        sheet.as_object_mut().unwrap().remove("inherited_variant");
    }
    legacy["sheets"].as_array_mut().unwrap().reverse();
    let p = decode_project(&legacy.to_string()).unwrap();
    assert_eq!(p.sheets()[0].id, SheetId::new(2).unwrap());
    assert_eq!(p.drawing().views().count(), 1);
    assert_eq!(
        p.sheets()[0].placement.as_ref().unwrap().view,
        p.sheets()[1].placement.as_ref().unwrap().view
    );
    let encoded = encode_project(&p).unwrap();
    assert_eq!(decode_project(&encoded).unwrap(), p);
    let after: Value = serde_json::from_str(&encoded).unwrap();
    for key in ["drawing", "devices", "conductors", "cables"] {
        assert_eq!(after[key], legacy[key]);
    }
}
#[test]
fn strict_v9_rejects_invalid_document_identity_metadata_and_missing_or_duplicate_membership() {
    let text = encode_project(&setup()).unwrap();
    let data: Value = serde_json::from_str(&text).unwrap();
    for (pointer, value) in [
        ("/drawing_documents", json!([])),
        ("/drawing_documents/0/title", json!(" \n")),
        ("/drawing_documents/0/id", json!("not-an-id")),
        ("/drawing_documents/0/sheets", json!([])),
        (
            "/drawing_documents/0/sheets",
            json!([format!("{:032x}", 1), format!("{:032x}", 1)]),
        ),
        (
            "/drawing_documents/0/sheets",
            json!([format!("{:032x}", 999)]),
        ),
    ] {
        let mut bad = data.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(decode_project(&bad.to_string()).is_err());
    }
    let mut bad = data.clone();
    bad["drawing_documents"][0]["future"] = json!(true);
    assert!(decode_project(&bad.to_string()).is_err());
    let mut bad = data.clone();
    bad["drawing_documents"]
        .as_array_mut()
        .unwrap()
        .push(data["drawing_documents"][0].clone());
    assert!(decode_project(&bad.to_string()).is_err());
    let mut bad = data;
    bad.as_object_mut().unwrap().remove("drawing_documents");
    assert!(decode_project(&bad.to_string()).is_err());
}
#[test]
fn noops_failed_batches_registered_actions_and_electrical_library_boundaries_preserve_exact_intent()
{
    let directory = tempfile::tempdir().unwrap();
    let mut d = Document::new(setup(), JsonFile::new(directory.path().join("p.json")));
    execute(
        &mut d,
        DrawingDocumentCommand::Insert {
            document: doc(2, "2"),
            index: 1,
        },
    );
    let before = d.session().project().clone();
    execute(
        &mut d,
        DrawingDocumentCommand::MoveSheet {
            sheet: SheetId::new(1).unwrap(),
            document: DrawingDocumentId::new(2).unwrap(),
            index: 0,
        },
    );
    d.undo(d.session().revision()).unwrap();
    let rev = d.session().revision();
    execute(
        &mut d,
        DrawingDocumentCommand::Replace(before.drawing_documents()[0].clone()),
    );
    assert_eq!(d.session().revision(), rev);
    d.redo(rev).unwrap();
    d.undo(d.session().revision()).unwrap();
    assert_eq!(d.session().project(), &before);
    assert!(
        d.execute_batch(
            d.session().revision(),
            vec![
                Command::DrawingDocument(DrawingDocumentCommand::MoveSheet {
                    sheet: SheetId::new(1).unwrap(),
                    document: DrawingDocumentId::new(2).unwrap(),
                    index: 0
                }),
                Command::DrawingDocument(DrawingDocumentCommand::Remove(
                    before.drawing_documents()[0].id
                ))
            ]
        )
        .is_err()
    );
    assert_eq!(d.session().project(), &before);
    assert_eq!(
        ActionRegistry::builtins()
            .get(&ActionId::new("drawing.documents.edit").unwrap())
            .unwrap()
            .input_requirement(),
        ActionInputRequirement::DrawingDocuments
    );
    let context = ActionContext::for_session(d.session());
    let invocation = ActionInvocation {
        input_id: InputId::new(1).unwrap(),
        action: ActionId::new("drawing.documents.edit").unwrap(),
        source: InvocationSource::Toolbar,
        repeat: false,
        context,
        parameters: ActionParameters::DrawingDocument(DrawingDocumentCommand::Replace(
            ProjectDrawingDocument {
                title: "Registered".into(),
                ..doc(2, "2")
            },
        )),
    };
    ActionDispatcher::default()
        .invoke(&ActionRegistry::builtins(), invocation, Some(&mut d))
        .unwrap();
    assert_eq!(
        d.session()
            .project()
            .drawing_document(DrawingDocumentId::new(2).unwrap())
            .unwrap()
            .title,
        "Registered"
    );
    let mut body = Project::new(ProjectId::new(1).unwrap());
    let root = SubsystemId::new(1).unwrap();
    body.insert_subsystem(Subsystem::new(root, "Assembly", None).unwrap())
        .unwrap();
    body.replace_drawing_document(ProjectDrawingDocument {
        title: "Custom metadata".into(),
        ..body.drawing_documents()[0].clone()
    })
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
        Err(ModelError::Sheet(SheetError::Document(
            DrawingDocumentError::ReusableDocumentsUnsupported
        )))
    ));
}
