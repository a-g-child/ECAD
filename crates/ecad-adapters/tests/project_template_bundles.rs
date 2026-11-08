//! Project bundles and deterministic legacy preservation, through real commands/save/reopen.
use ecad_adapters::*;
use ecad_app::*;
use ecad_domain::*;
use serde_json::Value;
use std::{fs, sync::Arc};
fn base() -> Project {
    decode_project(include_str!("fixtures/routed-conveyor-v11.json")).unwrap()
}
fn revision(id: u128, name: &str) -> Arc<TemplateRevision> {
    let pin = TemplateVariantRef {
        definition: TemplateDefinitionId::new(1).unwrap(),
        revision: TemplateRevisionId::new(id).unwrap(),
        variant: TemplateVariantId::new(1).unwrap(),
    };
    Arc::new(
        TemplateRevision::new(
            pin.revision_ref(),
            vec![
                SavedTemplateVariant::new(
                    pin,
                    name.into(),
                    SheetTemplate::StandardV1
                        .layout(&SheetPaper::default())
                        .unwrap(),
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    )
}
#[test]
fn imported_complete_revisions_reopen_offline_with_exact_project_undo_and_library_independence() {
    let dir = tempfile::tempdir().unwrap();
    let library_store = TemplateLibraryFile::new(dir.path().join("templates.json"));
    let mut publisher = TemplatePublisher::new(library_store.clone());
    let first = revision(1, "Saved custom template");
    publisher
        .publish(first.clone(), Recommendation::RecommendPublished)
        .unwrap();
    let before = base();
    let store = JsonFile::new(dir.path().join("project.json"));
    let mut document = Document::new(before.clone(), store.clone());
    document.save().unwrap();
    let import = template_library_import(publisher.catalog(), first.reference()).unwrap();
    document
        .execute(document.session().revision(), import.clone())
        .unwrap();
    assert!(document.session().is_dirty());
    let imported = document.session().project().clone();
    document.save().unwrap();
    assert!(!document.session().is_dirty());
    publisher
        .publish(
            revision(2, "Changed recommendation"),
            Recommendation::RecommendPublished,
        )
        .unwrap();
    assert_eq!(document.session().project(), &imported);
    fs::remove_file(library_store.path()).unwrap();
    let reopened = Document::open(store).unwrap();
    assert_eq!(reopened.session().project(), &imported);
    assert_eq!(
        reopened
            .session()
            .project()
            .template_revision(first.reference()),
        Some(first.as_ref())
    );
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &before);
    assert!(document.session().is_dirty());
    let redo = document.session().redo_len();
    assert!(redo > 0);
    document.redo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &imported);
    assert!(!document.session().is_dirty());
    let token = document.session().revision();
    assert!(!document.execute(token, import).unwrap());
    assert_eq!(document.session().revision(), token);
}
#[test]
fn conflicting_import_rolls_back_a_prefix_and_preserves_existing_exact_bodies() {
    let first = revision(1, "First");
    let second = revision(2, "Second");
    let mut session = Session::new(base());
    session
        .execute(
            session.revision(),
            Command::InsertTemplateRevision(first.clone()),
        )
        .unwrap();
    let before = session.project().clone();
    let history = session.undo_len();
    let commands = vec![
        Command::InsertTemplateRevision(second),
        Command::InsertTemplateRevision(revision(1, "Conflicting")),
    ];
    assert!(session.execute_batch(session.revision(), commands).is_err());
    assert_eq!(session.project(), &before);
    assert_eq!(session.undo_len(), history);
    assert_eq!(
        session.project().template_revision(first.reference()),
        Some(first.as_ref())
    );
}
fn varied_project() -> Project {
    let mut p = base();
    let mut drawing = p.drawing().clone();
    drawing
        .insert_view(DrawingView {
            id: ViewId::new(99).unwrap(),
            name: "Unplaced source".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    p.replace_drawing(drawing).unwrap();
    let original = DrawingSheet {
        inherited_variant: None,
        id: SheetId::new(1).unwrap(),
        kind: SheetKind::Schematic,
        template: None,
        number: "1".into(),
        title: "Overview\nLiteral title".into(),
        paper: SheetPaper::default(),
        placement: Some(SheetViewPlacement {
            view: p.drawing().views().next().unwrap().id,
            paper_origin: DrawingPoint {
                x: 19_000,
                y: 23_000,
            },
            view_origin: DrawingPoint { x: -91, y: 2_000 },
            numerator: 1,
            denominator: 2,
        }),
    };
    p.insert_sheet(original.clone(), 0).unwrap();
    let mut shared = original.clone();
    shared.id = SheetId::new(20).unwrap();
    shared.number = "20".into();
    shared.template = Some(SheetTemplate::StandardV1);
    shared.paper = SheetPaper {
        width: 420_000,
        height: 297_000,
        left: 17_000,
        right: 9_000,
        top: 13_000,
        bottom: 7_000,
    };
    p.insert_sheet(shared, 1).unwrap();
    let mut tiny = original;
    tiny.id = SheetId::new(30).unwrap();
    tiny.number = "30".into();
    tiny.kind = SheetKind::Cover;
    tiny.template = Some(SheetTemplate::CoverV1);
    tiny.placement = None;
    tiny.paper = SheetPaper {
        width: 1,
        height: 1,
        left: 0,
        right: 0,
        top: 0,
        bottom: 0,
    };
    p.insert_sheet(tiny, 2).unwrap();
    p
}
#[test]
fn preservation_capture_keeps_custom_paper_shared_views_tiny_sheets_and_inheritance_intent() {
    let p = varied_project();
    let mut session = Session::new(p.clone());
    let proposal = capture_legacy_templates(&session).unwrap();
    assert_eq!(session.project(), &p);
    assert!(proposal.sheets()[0].inherited);
    assert!(!proposal.sheets()[1].inherited); // Explicit standard-v1 equals the owner default.
    assert_eq!(
        proposal.document_defaults().len(),
        p.drawing_documents().len()
    );
    assert_eq!(proposal.creation_defaults().len(), 3);
    for item in proposal.sheets() {
        let old = p.sheet_frame(item.sheet).unwrap();
        let body = proposal
            .bodies()
            .find(|b| b.reference() == item.template.revision_ref())
            .unwrap();
        let layout = body.variant(item.template.variant).unwrap().layout();
        assert_eq!(layout.paper(), &p.sheet(item.sheet).unwrap().paper);
        assert_eq!(layout.border(), old.border);
        assert_eq!(layout.content(), old.content);
        assert_eq!(layout.tables().is_empty(), old.title_block_too_small);
        let captured: Vec<_> = layout
            .tables()
            .iter()
            .flat_map(|table| {
                table.cells().iter().map(move |cell| FrameRect {
                    x: table.bounds().x + cell.bounds.x,
                    y: table.bounds().y + cell.bounds.y,
                    width: cell.bounds.width,
                    height: cell.bounds.height,
                })
            })
            .collect();
        assert_eq!(
            captured,
            old.cells.iter().map(|cell| cell.bounds).collect::<Vec<_>>()
        );
    }
    let commands = proposal.commands();
    session
        .execute_batch(proposal.revision(), commands)
        .unwrap();
    assert_eq!(session.undo_len(), 1);
    assert_eq!(session.project().sheets(), p.sheets());
    assert_eq!(session.project().drawing(), p.drawing());
    assert_eq!(
        session.project().drawing_document_layout(),
        p.drawing_document_layout()
    );
    assert_eq!(session.project().template_defaults(), p.template_defaults());
    for sheet in p.sheets() {
        assert_eq!(
            session.project().sheet_frame(sheet.id).unwrap(),
            p.sheet_frame(sheet.id).unwrap()
        );
    }
    let saved = encode_project(session.project()).unwrap();
    assert_eq!(decode_project(&saved).unwrap(), *session.project());
    let mut before: Value = serde_json::from_str(&encode_project(&p).unwrap()).unwrap();
    let mut after: Value = serde_json::from_str(&saved).unwrap();
    before.as_object_mut().unwrap().remove("template_revisions");
    after.as_object_mut().unwrap().remove("template_revisions");
    assert_eq!(after, before);
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &p);
}
#[test]
fn capture_is_stable_across_reopen_lineages_and_repetition_and_rejects_stale_commit() {
    let mut session = Session::new(varied_project());
    let first = capture_legacy_templates(&session).unwrap();
    assert_eq!(
        first.sheets()[0].template.definition.to_string(),
        "efe8c6f7729ac337099733ea2598c674"
    );
    assert_eq!(
        first.sheets()[0].template.variant.to_string(),
        "3fbed27b5aa87c9a45b83dff756643ec"
    );
    let canonical = encode_project(session.project()).unwrap();
    let copied = canonical.replace(
        "\"project_id\": \"00000000000000000000000000000001\"",
        "\"project_id\": \"000000000000000000000000000000ff\"",
    );
    let copied = Session::new(decode_project(&copied).unwrap());
    let second = capture_legacy_templates(&copied).unwrap();
    assert_eq!(
        first.bodies().collect::<Vec<_>>(),
        second.bodies().collect::<Vec<_>>()
    );
    assert_eq!(first.sheets(), second.sheets());
    session
        .execute_batch(first.revision(), first.commands())
        .unwrap();
    let repeated = capture_legacy_templates(&session).unwrap();
    let token = session.revision();
    assert!(
        !session
            .execute_batch(repeated.revision(), repeated.commands())
            .unwrap()
    );
    assert_eq!(session.revision(), token);
    session.undo(session.revision()).unwrap();
    let redo = session.redo_len();
    assert!(
        session
            .execute_batch(first.revision(), first.commands())
            .is_err()
    );
    assert_eq!(session.redo_len(), redo);
    let fresh = capture_legacy_templates(&session).unwrap();
    // Re-capturing itself does not consume the redo branch or edit the project.
    assert_eq!(session.redo_len(), redo);
    assert_eq!(
        first.bodies().collect::<Vec<_>>(),
        fresh.bodies().collect::<Vec<_>>()
    );
}
#[test]
fn malformed_duplicate_unknown_and_nested_template_bodies_reject_whole_project_load() {
    let mut session = Session::new(base());
    session
        .execute(
            session.revision(),
            Command::InsertTemplateRevision(revision(1, "Saved")),
        )
        .unwrap();
    let data: Value = serde_json::from_str(&encode_project(session.project()).unwrap()).unwrap();
    let mut bad = data.clone();
    bad.as_object_mut().unwrap().remove("template_revisions");
    assert!(decode_project(&bad.to_string()).is_err());
    let mut bad = data.clone();
    let duplicate = bad["template_revisions"][0].clone();
    bad["template_revisions"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(matches!(
        decode_project(&bad.to_string()),
        Err(ProjectFileError::Model(ModelError::TemplateLibrary(
            TemplateLibraryError::DuplicateRevision(_)
        )))
    ));
    for pointer in ["/template_revisions/0", "/template_revisions/0/variants/0"] {
        let mut bad = data.clone();
        bad.pointer_mut(pointer).unwrap()["unknown"] = true.into();
        assert!(decode_project(&bad.to_string()).is_err());
    }
    let mut bad = data.clone();
    bad["template_revisions"][0]["variants"][0]["revision"] =
        TemplateRevisionId::new(99).unwrap().to_string().into();
    assert!(matches!(
        decode_project(&bad.to_string()),
        Err(ProjectFileError::TemplateBundle(_))
    ));
    let mut legacy = data;
    legacy["schema_version"] = 11.into();
    assert!(decode_project(&legacy.to_string()).is_err());
}
#[test]
fn legacy_projects_upgrade_without_implicit_capture_or_library_writes_and_reuse_rejects_bundles() {
    for text in [
        include_str!("fixtures/minimal-v2.json"),
        include_str!("fixtures/routed-conveyor-v10.json"),
        include_str!("fixtures/routed-conveyor-v11.json"),
    ] {
        let p = decode_project(text).unwrap();
        assert_eq!(p.template_revisions().len(), 0);
        assert_eq!(decode_project(&encode_project(&p).unwrap()).unwrap(), p);
    }
    let mut body = Project::new(ProjectId::new(1).unwrap());
    let root = SubsystemId::new(1).unwrap();
    body.insert_subsystem(Subsystem::new(root, "Electrical only", None).unwrap())
        .unwrap();
    body.insert_template_revision(revision(1, "Not electrical content"))
        .unwrap();
    let source = DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(1).unwrap(),
    );
    assert!(matches!(
        SubsystemRevision::new(source, root, body),
        Err(ModelError::Drawing(
            DrawingError::ReusableDrawingsUnsupported
        ))
    ));
}

#[test]
fn capture_rejects_existing_different_content_under_its_deterministic_pin() {
    let mut session = Session::new(varied_project());
    let proposal = capture_legacy_templates(&session).unwrap();
    let revision = proposal.bodies().next().unwrap();
    let variant = revision.variants().next().unwrap();
    let changed = SavedTemplateVariant::new(
        variant.reference(),
        "Conflicting body".into(),
        variant.layout().clone(),
    )
    .unwrap();
    let changed = Arc::new(TemplateRevision::new(revision.reference(), vec![changed]).unwrap());
    session
        .execute(session.revision(), Command::InsertTemplateRevision(changed))
        .unwrap();
    let before = session.project().clone();
    assert!(matches!(
        capture_legacy_templates(&session),
        Err(TemplateLibraryFileError::Catalog(
            TemplateLibraryError::RevisionConflict(_)
        ))
    ));
    assert_eq!(session.project(), &before);
}
