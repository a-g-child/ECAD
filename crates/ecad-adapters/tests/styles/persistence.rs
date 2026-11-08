//! Portable intent/settings persistence and explicit compatibility boundaries.

use super::*;
use serde_json::{Value, json};

#[test]
fn save_reopen_preserves_equal_override_intent_defaults_and_document_checkpoint_history() {
    let session = setup();
    assert_eq!(
        encode_project(session.project()).unwrap(),
        include_str!("../fixtures/styled-overview-v12.json")
    );
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("styled.json"));
    let mut document = Document::new(session.project().clone(), store.clone());
    document.save().unwrap();
    let before = document.session().project().clone();
    let plan = property_edit_plan(
        &before,
        [label(1)],
        BTreeMap::from([(PropertyId::TextHeight, PropertyValue::Length(length(3_000)))]),
    )
    .unwrap();
    document
        .execute_batch(document.session().revision(), plan.commands)
        .unwrap();
    document.save().unwrap();
    let reopened = Document::open(store).unwrap();
    assert_eq!(reopened.session().project(), document.session().project());
    assert_eq!(
        reopened
            .session()
            .project()
            .property_state(label(1), PropertyId::TextHeight)
            .unwrap(),
        PropertyState::Override(Some(PropertyValue::Length(length(3_000))))
    );
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &before);
    assert!(document.session().is_dirty());
    document.redo(document.session().revision()).unwrap();
    assert!(!document.session().is_dirty());
}

#[test]
fn v2_v3_v4_upgrade_preserves_electrical_identity_and_gives_portable_empty_drawing_defaults() {
    for text in [
        include_str!("../fixtures/minimal-v2.json"),
        include_str!("../fixtures/minimal-v3.json"),
        include_str!("../fixtures/minimal-v4.json"),
    ] {
        let project = decode_project(text).unwrap();
        assert_eq!(project.drawing(), &DrawingDocument::default());
        let current = encode_project(&project).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&current).unwrap()["schema_version"],
            12
        );
        assert_eq!(decode_project(&current).unwrap(), project);
    }
}

#[test]
fn strict_drawing_schema_rejects_unknowns_duplicates_zero_lengths_missing_refs_and_wrong_roles() {
    let data: Value = serde_json::from_str(&encode_project(setup().project()).unwrap()).unwrap();
    for pointer in [
        "/drawing",
        "/drawing/settings",
        "/drawing/settings/base",
        "/drawing/views/0",
        "/drawing/styles/0/values",
        "/drawing/labels/0/appearance",
        "/drawing/polylines/0",
    ] {
        let mut invalid = data.clone();
        invalid.pointer_mut(pointer).unwrap()["unknown"] = json!(true);
        assert!(decode_project(&invalid.to_string()).is_err());
    }
    for array in ["styles", "views", "labels", "polylines"] {
        let mut invalid = data.clone();
        let repeated = invalid["drawing"][array][0].clone();
        invalid["drawing"][array]
            .as_array_mut()
            .unwrap()
            .push(repeated);
        assert!(decode_project(&invalid.to_string()).is_err());
    }
    for (pointer, value) in [
        ("/drawing/settings/base/text_height_um", json!(0)),
        ("/drawing/styles/0/values/text_height_um", json!(0)),
        ("/drawing/styles/0/values/stroke_width_um", json!(500)),
        (
            "/drawing/labels/0/view",
            json!("00000000000000000000000000000999"),
        ),
        (
            "/drawing/labels/0/appearance/style",
            json!("00000000000000000000000000000002"),
        ),
        (
            "/drawing/settings/stroke_creation/style",
            json!("00000000000000000000000000000999"),
        ),
        ("/drawing/settings/base/colour", json!([256, 0, 0])),
        ("/drawing/polylines/0/points_um", json!([[1.5, 0], [2, 0]])),
    ] {
        let mut invalid = data.clone();
        *invalid.pointer_mut(pointer).unwrap() = value;
        assert!(
            decode_project(&invalid.to_string()).is_err(),
            "accepted {pointer}"
        );
    }
    let mut invalid = data.clone();
    invalid["schema_version"] = json!(13);
    assert!(decode_project(&invalid.to_string()).is_err());
    let mut invalid = data.clone();
    invalid.as_object_mut().unwrap().remove("drawing");
    assert!(decode_project(&invalid.to_string()).is_err());
    let mut invalid = data;
    invalid["schema_version"] = json!(4);
    assert!(decode_project(&invalid.to_string()).is_err());
}

#[test]
fn drawing_arrays_canonicalize_and_reusable_electrical_bodies_reject_drawing_loss() {
    let project = setup().project().clone();
    let text = encode_project(&project).unwrap();
    let mut data: Value = serde_json::from_str(&text).unwrap();
    for array in ["styles", "views", "labels", "polylines"] {
        data["drawing"][array].as_array_mut().unwrap().reverse();
    }
    assert_eq!(
        encode_project(&decode_project(&data.to_string()).unwrap()).unwrap(),
        text
    );
    let mut body = decode_project(include_str!("../fixtures/sensor-module-v3.json")).unwrap();
    body.replace_drawing(project.drawing().clone()).unwrap();
    let source = DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(1).unwrap(),
    );
    assert!(matches!(
        SubsystemRevision::new(source, SubsystemId::new(10).unwrap(), body),
        Err(ModelError::Drawing(
            DrawingError::ReusableDrawingsUnsupported
        ))
    ));
}

#[test]
fn project_fallback_changes_follow_existing_inheritance_while_creation_overrides_remain_captured() {
    let mut session = setup();
    run(
        &mut session,
        DrawingCommand::ReplaceStyle(
            DrawingStyle::new(
                text_style(),
                "Sparse",
                DrawingRole::Text,
                StylePatch::default(),
            )
            .unwrap(),
        ),
    );
    assert_eq!(
        session
            .project()
            .property(label(1), PropertyId::TextHeight)
            .unwrap()
            .origin,
        Some(PropertyOrigin::Project)
    );
    let mut settings = session.project().drawing().settings().clone();
    settings.text_creation.overrides.text_height = Some(length(8_000));
    settings.base.text_height = length(9_000);
    run(&mut session, DrawingCommand::Settings(settings));
    assert_eq!(
        session
            .project()
            .property(label(1), PropertyId::TextHeight)
            .unwrap()
            .value,
        PropertyValue::Length(length(9_000))
    );
    run(
        &mut session,
        DrawingCommand::CreateLabel {
            id: LabelId::new(3).unwrap(),
            view: view(),
            text: "New".into(),
            position: DrawingPoint::default(),
        },
    );
    assert_eq!(
        session
            .project()
            .property(label(3), PropertyId::TextHeight)
            .unwrap()
            .value,
        PropertyValue::Length(length(8_000))
    );
    let mut settings = session.project().drawing().settings().clone();
    settings.base.text_height = length(10_000);
    run(&mut session, DrawingCommand::Settings(settings));
    assert_eq!(
        session
            .project()
            .property(label(1), PropertyId::TextHeight)
            .unwrap()
            .value,
        PropertyValue::Length(length(10_000))
    );
    assert_eq!(
        session
            .project()
            .property(label(3), PropertyId::TextHeight)
            .unwrap()
            .value,
        PropertyValue::Length(length(8_000))
    );
}

#[test]
fn creation_presets_alone_keep_referenced_styles_alive_and_failed_creation_leaves_no_partial_view()
{
    let mut session = ecad_app::Session::new(Project::new(ProjectId::new(1).unwrap()));
    run(
        &mut session,
        DrawingCommand::InsertStyle(
            DrawingStyle::new(
                text_style(),
                "For future labels",
                DrawingRole::Text,
                StylePatch::default(),
            )
            .unwrap(),
        ),
    );
    let mut settings = session.project().drawing().settings().clone();
    settings.text_creation.style = Some(text_style());
    run(&mut session, DrawingCommand::Settings(settings));
    let before = session.project().clone();
    assert!(
        session
            .execute(
                session.revision(),
                Command::Drawing(DrawingCommand::RemoveStyle(text_style()))
            )
            .is_err()
    );
    assert_eq!(session.project(), &before);
    let commands = vec![
        Command::Drawing(DrawingCommand::InsertView(DrawingView {
            id: view(),
            name: "Temporary".into(),
            overrides: StylePatch::default(),
        })),
        Command::Drawing(DrawingCommand::CreateLabel {
            id: LabelId::new(1).unwrap(),
            view: view(),
            text: "  ".into(),
            position: DrawingPoint::default(),
        }),
    ];
    assert!(session.execute_batch(session.revision(), commands).is_err());
    assert_eq!(session.project(), &before);
}
