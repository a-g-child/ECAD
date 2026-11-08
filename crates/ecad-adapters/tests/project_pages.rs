//! Creation defaults, independent source views, generated navigation and strict v11 persistence.
use ecad_adapters::{JsonFile, decode_project, encode_project};
use ecad_app::*;
use ecad_domain::*;
use serde_json::{Value, json};
fn sheet(n: u128, number: &str, kind: SheetKind) -> DrawingSheet {
    DrawingSheet {
        id: SheetId::new(n).unwrap(),
        kind,
        template: Some(DrawingTemplateDefaults::default().for_kind(kind)),
        number: number.into(),
        title: format!("Drawing {number}\nLiteral title"),
        paper: SheetPaper::default(),
        placement: None,
    }
}
fn commit(
    d: &mut Document<JsonFile>,
    dispatcher: &mut ActionDispatcher,
    input: u64,
    action: &str,
    parameters: ActionParameters,
) {
    let invocation = ActionInvocation {
        input_id: InputId::new(input).unwrap(),
        action: ActionId::new(action).unwrap(),
        source: InvocationSource::Toolbar,
        repeat: false,
        context: ActionContext::for_session(d.session()),
        parameters,
    };
    dispatcher
        .invoke(&ActionRegistry::builtins(), invocation, Some(d))
        .unwrap();
}
#[test]
fn project_properties_are_one_exact_undo_and_defaults_do_not_restyle_existing_sheets() {
    let directory = tempfile::tempdir().unwrap();
    let mut p = Project::new(ProjectId::new(1).unwrap());
    p.insert_sheet(sheet(1, "1", SheetKind::Cover), 0).unwrap();
    let before = p.clone();
    let frame = p.sheet_frame(SheetId::new(1).unwrap()).unwrap();
    let mut d = Document::new(p, JsonFile::new(directory.path().join("properties.json")));
    let templates = DrawingTemplateDefaults {
        cover: SheetTemplate::StandardV1,
        ..DrawingTemplateDefaults::default()
    };
    let parameters = ActionParameters::ProjectProperties {
        metadata: ProjectMetadata::new(
            [(MetadataField::CompanyName, "Portable\nCompany".into())].into(),
        ),
        templates: templates.clone(),
    };
    let mut dispatcher = ActionDispatcher::default();
    commit(
        &mut d,
        &mut dispatcher,
        1,
        "document.metadata.edit",
        parameters.clone(),
    );
    assert_eq!(d.session().undo_len(), 1);
    assert_eq!(d.session().project().template_defaults(), &templates);
    assert_eq!(
        d.session()
            .project()
            .sheet_frame(SheetId::new(1).unwrap())
            .unwrap()
            .template,
        frame.template
    );
    d.save().unwrap();
    let saved = d.session().project().clone();
    assert_eq!(
        Document::open(JsonFile::new(directory.path().join("properties.json")))
            .unwrap()
            .session()
            .project(),
        &saved
    );
    d.undo(d.session().revision()).unwrap();
    assert_eq!(d.session().project(), &before);
    d.redo(d.session().revision()).unwrap();
    assert!(!d.session().is_dirty());
    d.execute(
        d.session().revision(),
        Command::ProjectMetadata(ProjectMetadata::default()),
    )
    .unwrap();
    d.undo(d.session().revision()).unwrap();
    let revision = d.session().revision();
    commit(
        &mut d,
        &mut dispatcher,
        2,
        "document.metadata.edit",
        parameters,
    );
    assert_eq!(d.session().revision(), revision);
    assert_eq!(d.session().redo_len(), 1);
}
#[test]
fn fresh_schematic_view_and_sheet_commit_as_one_operation_with_exact_save_history_and_cancel() {
    let directory = tempfile::tempdir().unwrap();
    let p = decode_project(include_str!("fixtures/routed-conveyor-v10.json")).unwrap();
    let before = p.clone();
    let owner = p.drawing_documents()[0].id;
    let view = DrawingView {
        id: ViewId::new(42).unwrap(),
        name: "Independent".into(),
        overrides: StylePatch::default(),
    };
    let mut page = sheet(42, "42", SheetKind::Schematic);
    page.placement = Some(SheetViewPlacement {
        view: view.id,
        paper_origin: DrawingPoint {
            x: 20_000,
            y: 60_000,
        },
        view_origin: DrawingPoint::default(),
        numerator: 1,
        denominator: 1,
    });
    let mut d = Document::new(p, JsonFile::new(directory.path().join("sheet.json")));
    let draft =
        preview_sheet_in_document(d.session(), page.clone(), true, owner, Some(view.clone()))
            .unwrap();
    assert_eq!(d.session().project(), &before); // Dropping a proposal creates neither view nor sheet.
    let mut dispatcher = ActionDispatcher::default();
    commit(
        &mut d,
        &mut dispatcher,
        1,
        "drawing.sheets.edit",
        ActionParameters::SheetPreview(draft),
    );
    assert_eq!(d.session().undo_len(), 1);
    assert_eq!(
        d.session().project().drawing().views().len(),
        before.drawing().views().len() + 1
    );
    assert_eq!(d.session().project().sheet(page.id), Some(&page));
    assert_eq!(
        d.session().project().drawing().routes().count(),
        before.drawing().routes().count()
    );
    d.save().unwrap();
    let saved = d.session().project().clone();
    assert_eq!(
        Document::open(JsonFile::new(directory.path().join("sheet.json")))
            .unwrap()
            .session()
            .project(),
        &saved
    );
    d.undo(d.session().revision()).unwrap();
    assert_eq!(d.session().project(), &before);
    d.redo(d.session().revision()).unwrap();
    assert_eq!(d.session().project(), &saved);
    assert!(!d.session().is_dirty());
    let mut bad = page.clone();
    bad.id = SheetId::new(43).unwrap();
    bad.number = "43".into();
    bad.kind = SheetKind::Cover;
    assert!(preview_sheet_in_document(d.session(), bad, true, owner, Some(view)).is_err());
    assert_eq!(d.session().project(), &saved);
}
#[test]
fn drawing_lists_naturally_order_stable_targets_and_paginate_by_actual_list_sheet_capacities() {
    let mut p = Project::new(ProjectId::new(1).unwrap());
    for (i, number) in ["10", "2", "1.10", "1.2", "A20", "A3"]
        .into_iter()
        .enumerate()
    {
        p.insert_sheet(
            sheet(
                i as u128 + 1,
                number,
                if i == 0 {
                    SheetKind::Cover
                } else {
                    SheetKind::Schematic
                },
            ),
            p.sheets().len(),
        )
        .unwrap();
    }
    let mut list = sheet(100, "LIST1", SheetKind::DrawingList);
    list.paper.height = 80_000;
    p.insert_sheet(list.clone(), p.sheets().len()).unwrap();
    let first = p.drawing_list(list.id).unwrap().unwrap();
    assert_eq!(first.capacity, 2);
    assert_eq!(first.remaining, 4);
    assert_eq!(
        first
            .rows
            .iter()
            .map(|r| r.number.as_str())
            .collect::<Vec<_>>(),
        ["1.2", "1.10"]
    );
    for n in 101..=102 {
        let mut next = list.clone();
        next.id = SheetId::new(n).unwrap();
        next.number = format!("LIST{n}");
        p.insert_sheet(next, p.sheets().len()).unwrap();
    }
    let mut rows = Vec::new();
    for n in 100..=102 {
        let page = p.drawing_list(SheetId::new(n).unwrap()).unwrap().unwrap();
        assert_eq!(page.remaining, 0);
        rows.extend(page.rows);
    }
    assert_eq!(
        rows.iter().map(|r| r.number.as_str()).collect::<Vec<_>>(),
        ["1.2", "1.10", "2", "10", "A3", "A20"]
    );
    let target = rows[0].sheet;
    let mut renamed = p.sheet(target).unwrap().clone();
    renamed.number = "0".into();
    renamed.title = "Changed\nTitle".into();
    p.replace_sheet(renamed).unwrap();
    let first = p.drawing_list(list.id).unwrap().unwrap();
    assert_eq!(first.rows[0].sheet, target);
    assert_eq!(first.rows[0].number, "0");
    assert_eq!(first.rows[0].title, "Changed\nTitle");
    let mut tiny = p.sheet(list.id).unwrap().clone();
    tiny.paper.width = 20_001;
    tiny.paper.height = 20_001;
    p.replace_sheet(tiny).unwrap();
    let too_small = p.drawing_list(list.id).unwrap().unwrap();
    assert_eq!(too_small.capacity, 0);
    assert!(too_small.rows.is_empty());
    assert_eq!(too_small.remaining, 2);
    let text = encode_project(&p).unwrap();
    assert!(!text.contains("remaining"));
    assert_eq!(decode_project(&text).unwrap(), p);
}
#[test]
fn legacy_loading_preserves_appearance_and_v11_rejects_unknown_templates_roles_and_source_on_cover()
{
    for old in [
        include_str!("fixtures/routed-conveyor-v8.json"),
        include_str!("fixtures/routed-conveyor-v9.json"),
        include_str!("fixtures/routed-conveyor-v10.json"),
    ] {
        let p = decode_project(old).unwrap();
        assert_eq!(p.template_defaults(), &DrawingTemplateDefaults::default());
        assert_eq!(decode_project(&encode_project(&p).unwrap()).unwrap(), p);
    }
    let mut p = Project::new(ProjectId::new(1).unwrap());
    p.insert_sheet(sheet(1, "1", SheetKind::Cover), 0).unwrap();
    let data: Value = serde_json::from_str(&encode_project(&p).unwrap()).unwrap();
    for (pointer, value) in [
        ("/template_defaults/cover", json!("unknown")),
        ("/sheets/0/kind", json!("other")),
        ("/sheets/0/template", json!("unavailable")),
    ] {
        let mut bad = data.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(decode_project(&bad.to_string()).is_err());
    }
    let mut bad = data;
    bad.as_object_mut().unwrap().remove("template_defaults");
    assert!(decode_project(&bad.to_string()).is_err());
    let mut p = decode_project(include_str!("fixtures/routed-conveyor-v10.json")).unwrap();
    let mut bad = sheet(1, "1", SheetKind::Cover);
    bad.placement = Some(SheetViewPlacement {
        view: ViewId::new(1).unwrap(),
        paper_origin: DrawingPoint::default(),
        view_origin: DrawingPoint::default(),
        numerator: 1,
        denominator: 1,
    });
    let before = p.clone();
    assert!(p.insert_sheet(bad, 0).is_err());
    assert_eq!(p, before);
}

#[test]
fn reusable_electrical_bodies_cannot_silently_discard_custom_project_template_defaults() {
    let mut body = Project::new(ProjectId::new(1).unwrap());
    let root = SubsystemId::new(10).unwrap();
    body.insert_subsystem(Subsystem::new(root, "Reusable", None).unwrap())
        .unwrap();
    let source = DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(2).unwrap(),
    );
    assert!(SubsystemRevision::new(source, root, body.clone()).is_ok());
    body.replace_template_defaults(DrawingTemplateDefaults {
        cover: SheetTemplate::StandardV1,
        ..DrawingTemplateDefaults::default()
    });
    assert!(matches!(
        SubsystemRevision::new(source, root, body),
        Err(ModelError::Drawing(
            DrawingError::ReusableDrawingsUnsupported
        ))
    ));
}
