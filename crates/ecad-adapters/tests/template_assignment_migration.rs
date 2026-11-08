//! Saved-reference migration preserves layout and intent, rejects dangling pins, and undoes exactly.
use ecad_adapters::*;
use ecad_app::*;
use ecad_domain::*;
use serde_json::{Value, json};
fn legacy() -> Project {
    let mut p = decode_project(include_str!("fixtures/routed-conveyor-v12.json")).unwrap();
    let view = p.drawing().views().next().unwrap().id;
    for (id, paper, template, kind) in [
        (1, SheetPaper::default(), None, SheetKind::Schematic),
        (
            2,
            SheetPaper {
                width: 420_000,
                height: 297_000,
                left: 17_000,
                right: 9_000,
                top: 13_000,
                bottom: 7_000,
            },
            None,
            SheetKind::Schematic,
        ),
        (
            3,
            SheetPaper::default(),
            Some(SheetTemplate::StandardV1),
            SheetKind::Cover,
        ),
        (
            4,
            SheetPaper {
                width: 1,
                height: 1,
                left: 0,
                right: 0,
                top: 0,
                bottom: 0,
            },
            Some(SheetTemplate::CoverV1),
            SheetKind::Cover,
        ),
    ] {
        p.insert_sheet(
            DrawingSheet {
                id: SheetId::new(id).unwrap(),
                kind,
                template,
                inherited_variant: None,
                number: id.to_string(),
                title: format!("Drawing {id}\nLiteral"),
                paper,
                placement: (kind == SheetKind::Schematic).then_some(SheetViewPlacement {
                    view,
                    paper_origin: DrawingPoint {
                        x: 19_000,
                        y: 23_000,
                    },
                    view_origin: DrawingPoint { x: -91, y: 2_000 },
                    numerator: 1,
                    denominator: 2,
                }),
            },
            p.sheets().len(),
        )
        .unwrap();
    }
    p
}
#[test]
fn migration_keeps_exact_frames_shared_geometry_and_inherited_paper_variants_with_one_undo() {
    let original = legacy();
    let mut session = Session::new(original.clone());
    let plan = preview_template_assignment_migration(&session).unwrap();
    assert_eq!(session.project(), &original);
    session
        .execute_batch(plan.revision(), plan.commands())
        .unwrap();
    let migrated = session.project().clone();
    assert_eq!(session.undo_len(), 1);
    assert_eq!(migrated.drawing(), original.drawing());
    assert_eq!(migrated.metadata(), original.metadata());
    assert!(migrated.sheets()[0].template.is_none());
    assert!(migrated.sheets()[1].template.is_none());
    assert!(migrated.sheets()[1].inherited_variant.is_some());
    assert!(matches!(
        migrated.sheets()[2].template,
        Some(SheetTemplate::Saved(_))
    ));
    for sheet in original.sheets() {
        let before = original.sheet_frame(sheet.id).unwrap();
        let after = migrated.sheet_frame(sheet.id).unwrap();
        assert_eq!(after.border, before.border);
        assert_eq!(after.content, before.content);
        assert_eq!(after.cells, before.cells);
        assert_eq!(after.appearance, before.appearance);
        assert_eq!(after.title_block_too_small, before.title_block_too_small);
        assert_eq!(migrated.sheet(sheet.id).unwrap().placement, sheet.placement);
        assert_eq!(migrated.sheet(sheet.id).unwrap().paper, sheet.paper);
    }
    let owner = migrated.drawing_documents()[0].default_template;
    let SheetTemplate::Saved(pin) = owner else {
        panic!("saved default")
    };
    assert!(
        migrated
            .template_revision(pin.revision_ref())
            .unwrap()
            .variants()
            .len()
            >= 2
    );
    assert_eq!(
        decode_project(&encode_project(&migrated).unwrap()).unwrap(),
        migrated
    );
    assert_eq!(migrate_template_assignments(&migrated).unwrap(), migrated);
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &original);
    assert!(
        session
            .execute_batch(plan.revision(), plan.commands())
            .is_err()
    );
    session.redo(session.revision()).unwrap();
    assert_eq!(session.project(), &migrated);
}
#[test]
fn migrated_loading_is_clean_observes_original_bytes_and_writes_only_on_explicit_save() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.json");
    let original = encode_project(&legacy()).unwrap();
    std::fs::write(&path, &original).unwrap();
    let store = JsonFile::new(&path).with_saved_templates();
    let mut doc = Document::open(store.clone()).unwrap();
    assert!(!doc.session().is_dirty());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    let migrated = doc.session().project().clone();
    doc.save().unwrap();
    let reopened = Document::open(store).unwrap();
    assert_eq!(reopened.session().project(), &migrated);
    assert!(!reopened.session().is_dirty());
    let saved = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&saved).unwrap()["schema_version"],
        13
    );
    // A failed observed-version save retains migrated model/checkpoint/history.
    std::fs::write(&path, "external change").unwrap();
    assert!(doc.save().is_err());
    assert_eq!(doc.session().project(), &migrated);
    assert!(!doc.session().is_dirty());
}
#[test]
fn in_use_guards_cover_defaults_documents_and_explicit_sheets_and_release_only_after_reassignment()
{
    let mut p = migrate_template_assignments(&legacy()).unwrap();
    let before = p.clone();
    let mut used = vec![];
    for t in [
        p.template_defaults().schematic,
        p.template_defaults().cover,
        p.template_defaults().drawing_list,
        p.drawing_documents()[0].default_template,
    ] {
        let SheetTemplate::Saved(pin) = t else {
            panic!("saved")
        };
        used.push(pin.revision_ref());
    }
    for sheet in p.sheets() {
        if let Some(SheetTemplate::Saved(pin)) = sheet.template {
            used.push(pin.revision_ref());
        }
    }
    for pin in used {
        assert!(matches!(
            p.remove_template_revision(pin),
            Err(TemplateLibraryError::InUse(_))
        ));
        assert_eq!(p, before);
    }
    let SheetTemplate::Saved(list) = p.template_defaults().drawing_list else {
        panic!("saved")
    };
    let mut defaults = p.template_defaults().clone();
    defaults.drawing_list = defaults.schematic;
    p.replace_template_defaults(defaults).unwrap();
    assert!(p.remove_template_revision(list.revision_ref()).is_ok());
}
#[test]
fn bad_saved_references_paper_tampering_inheritance_and_transfer_fail_without_mutation() {
    let mut p = migrate_template_assignments(&legacy()).unwrap();
    let before = p.clone();
    let mut sheet = p.sheets()[0].clone();
    sheet.paper.width += 1;
    assert!(p.replace_sheet(sheet).is_err());
    assert_eq!(p, before);
    let mut assignments = p.template_assignments();
    let row = &mut assignments.sheets[0];
    row.inherited_variant = Some(TemplateVariantId::new(1).unwrap());
    assert!(p.replace_template_assignments(assignments).is_err());
    assert_eq!(p, before);
    let mut default = p.template_defaults().clone();
    default.cover = SheetTemplate::Saved(TemplateVariantRef {
        definition: TemplateDefinitionId::new(1).unwrap(),
        revision: TemplateRevisionId::new(1).unwrap(),
        variant: TemplateVariantId::new(1).unwrap(),
    });
    assert!(p.replace_template_defaults(default).is_err());
    assert_eq!(p, before);
    let mut owner = p.drawing_documents()[0].clone();
    owner.id = DrawingDocumentId::new(99).unwrap();
    owner.number = "99".into();
    owner.default_template = p.template_defaults().drawing_list;
    p.insert_drawing_document(owner.clone(), 1).unwrap();
    let before = p.clone();
    assert!(
        p.move_sheet_to_document(SheetId::new(2).unwrap(), owner.id, 0)
            .is_err()
    );
    assert_eq!(p, before);
}
#[test]
fn retained_tiny_layout_reopens_but_cannot_be_used_for_new_sheets_and_undo_can_restore_it() {
    let project = migrate_template_assignments(&legacy()).unwrap();
    let tiny = project.sheet(SheetId::new(4).unwrap()).unwrap().clone();
    let SheetTemplate::Saved(pin) = tiny.template.unwrap() else {
        panic!("saved")
    };
    let body = project.saved_template(pin).unwrap();
    assert!(!body.allows_new_assignments());
    let text = encode_template_variant(body).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&text).unwrap()["schema_version"],
        2
    );
    assert_eq!(decode_template_variant(&text).unwrap(), *body);
    let mut session = Session::new(project.clone());
    let mut new = tiny.clone();
    new.id = SheetId::new(50).unwrap();
    new.number = "50".into();
    assert!(preview_sheet(&session, new, true).is_err());
    session
        .execute(
            session.revision(),
            Command::Sheet(SheetCommand::Remove(tiny.id)),
        )
        .unwrap();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &project);
    assert!(preview_sheet(&session, tiny, false).is_ok());
}
#[test]
fn v13_rejects_unknown_pin_fields_dangling_variants_and_legacy_schema_saved_references() {
    let p = migrate_template_assignments(&legacy()).unwrap();
    let data: Value = serde_json::from_str(&encode_project(&p).unwrap()).unwrap();
    for (pointer, value) in [
        (
            "/template_defaults/schematic/saved/variant",
            json!("00000000000000000000000000000001"),
        ),
        ("/template_defaults/schematic/saved/revision", json!("0")),
        (
            "/sheets/0/inherited_variant",
            json!("00000000000000000000000000000001"),
        ),
    ] {
        let mut bad = data.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(decode_project(&bad.to_string()).is_err(), "{pointer}");
    }
    let mut bad = data.clone();
    bad["template_defaults"]["schematic"]["saved"]["unknown"] = true.into();
    assert!(decode_project(&bad.to_string()).is_err());
    let mut bad = data;
    bad["schema_version"] = 12.into();
    assert!(decode_project(&bad.to_string()).is_err());
}

#[test]
fn arbitrary_saved_cells_render_literal_and_composed_multiline_contents_without_legacy_assumptions()
{
    let mut p = migrate_template_assignments(&legacy()).unwrap();
    let pin = TemplateVariantRef {
        definition: TemplateDefinitionId::new(900).unwrap(),
        revision: TemplateRevisionId::new(901).unwrap(),
        variant: TemplateVariantId::new(902).unwrap(),
    };
    let table = TemplateTable::new(
        TemplateTableId::new(90).unwrap(),
        FrameRect {
            x: 10_000,
            y: 10_000,
            width: 100_000,
            height: 30_000,
        },
        vec![TemplateCell {
            id: TemplateCellId::new(91).unwrap(),
            bounds: FrameRect {
                x: 0,
                y: 0,
                width: 100_000,
                height: 30_000,
            },
            caption: "Custom\ncaption".into(),
            contents: vec![
                TemplateTextRun::Literal("Title:\n".into()),
                TemplateTextRun::Field(FrameField::SheetTitle),
                TemplateTextRun::Literal("\nOptional address: ".into()),
                TemplateTextRun::Field(FrameField::Metadata(MetadataField::CompanyAddress)),
            ],
        }],
    )
    .unwrap();
    let layout = TemplateLayout::new(
        SheetPaper::default(),
        FrameRect {
            x: 10_000,
            y: 50_000,
            width: 100_000,
            height: 100_000,
        },
        vec![table],
    )
    .unwrap();
    let body = SavedTemplateVariant::new(pin, "Custom saved layout".into(), layout).unwrap();
    p.insert_template_revision(std::sync::Arc::new(
        TemplateRevision::new(pin.revision_ref(), vec![body]).unwrap(),
    ))
    .unwrap();
    let mut assignments = p.template_assignments();
    assignments.sheets[0].template = Some(SheetTemplate::Saved(pin));
    assignments.sheets[0].inherited_variant = None;
    p.replace_template_assignments(assignments).unwrap();
    let frame = p.sheet_frame(SheetId::new(1).unwrap()).unwrap();
    assert_eq!(frame.cells.len(), 1);
    assert_eq!(frame.cells[0].caption, "Custom\ncaption");
    assert_eq!(frame.cells[0].field, None);
    assert_eq!(
        frame.cells[0].value.as_deref(),
        Some("Title:\nDrawing 1\nLiteral\nOptional address: ")
    );
    assert_eq!(decode_project(&encode_project(&p).unwrap()).unwrap(), p);
}
