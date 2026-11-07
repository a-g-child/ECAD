//! Creation snapshots, per-property inheritance, and exact intent-aware history.

#[path = "styles/persistence.rs"]
mod persistence;
#[path = "styles/support.rs"]
mod support;
use ecad_adapters::{JsonFile, decode_project, encode_project};
use ecad_app::{
    Command, Document, DrawingCommand, PropertyEdit, SelectionValue, inspect_properties,
    property_edit_plan, property_patch_plan,
};
use ecad_domain::*;
use std::collections::BTreeMap;
use support::*;

#[test]
fn inheritance_resolves_each_property_through_object_view_named_and_project_layers() {
    let mut session = setup();
    let mut scope = session.project().drawing().view(view()).unwrap().clone();
    scope.overrides.text_height = Some(length(4_000));
    scope.overrides.stroke_width = Some(length(750));
    run(&mut session, DrawingCommand::ReplaceView(scope));
    let red = DrawingColour {
        red: 255,
        green: 0,
        blue: 0,
    };
    set(
        &mut session,
        label(1),
        PropertyId::DrawingColour,
        PropertyValue::Colour(red),
    );
    for (target, property, value, origin) in [
        (
            label(1),
            PropertyId::DrawingColour,
            PropertyValue::Colour(red),
            PropertyOrigin::Object,
        ),
        (
            label(1),
            PropertyId::TextHeight,
            PropertyValue::Length(length(4_000)),
            PropertyOrigin::View(view()),
        ),
        (
            label(1),
            PropertyId::FontFamily,
            PropertyValue::Text("Named family".into()),
            PropertyOrigin::NamedStyle(text_style()),
        ),
        (
            label(2),
            PropertyId::DrawingColour,
            PropertyValue::Colour(DrawingColour::default()),
            PropertyOrigin::Project,
        ),
        (
            path(),
            PropertyId::StrokeWidth,
            PropertyValue::Length(length(750)),
            PropertyOrigin::View(view()),
        ),
    ] {
        let resolved = session.project().property(target, property).unwrap();
        assert_eq!(resolved.value, value);
        assert_eq!(resolved.origin, Some(origin));
    }
}

#[test]
fn creation_presets_change_only_future_objects_and_undo_restores_the_exact_captured_binding() {
    let mut session = setup();
    let first = session
        .project()
        .drawing()
        .label(LabelId::new(1).unwrap())
        .unwrap()
        .clone();
    let mut settings = session.project().drawing().settings().clone();
    settings.text_creation.overrides.text_height = Some(length(5_000));
    let before = session.project().clone();
    let commands = vec![
        Command::Drawing(DrawingCommand::Settings(settings)),
        Command::Drawing(DrawingCommand::CreateLabel {
            id: LabelId::new(3).unwrap(),
            view: view(),
            text: "New".into(),
            position: DrawingPoint::default(),
        }),
    ];
    session.execute_batch(session.revision(), commands).unwrap();
    assert_eq!(session.project().drawing().label(first.id), Some(&first));
    assert_eq!(
        session
            .project()
            .property(label(3), PropertyId::TextHeight)
            .unwrap()
            .origin,
        Some(PropertyOrigin::Object)
    );
    assert_eq!(
        session
            .project()
            .property(label(3), PropertyId::TextHeight)
            .unwrap()
            .value,
        PropertyValue::Length(length(5_000))
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    session.redo(session.revision()).unwrap();
    assert_eq!(
        session
            .project()
            .drawing()
            .label(LabelId::new(3).unwrap())
            .unwrap()
            .appearance
            .overrides
            .text_height,
        Some(length(5_000))
    );
}

#[test]
fn explicit_equal_values_have_distinct_intent_and_reset_undo_restores_it_exactly() {
    let mut session = setup();
    let before = session.project().clone();
    let history = session.undo_len();
    let plan = property_edit_plan(
        &before,
        [label(1)],
        BTreeMap::from([(PropertyId::TextHeight, PropertyValue::Length(length(3_000)))]),
    )
    .unwrap();
    assert_eq!(plan.changes.len(), 1);
    assert_eq!(plan.changes[0].before, plan.changes[0].after);
    assert_ne!(plan.changes[0].before_state, plan.changes[0].after_state);
    session
        .execute_batch(session.revision(), plan.commands)
        .unwrap();
    assert_eq!(session.undo_len(), history + 1);
    assert_eq!(
        session
            .project()
            .property(label(1), PropertyId::TextHeight)
            .unwrap()
            .origin,
        Some(PropertyOrigin::Object)
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    session.redo(session.revision()).unwrap();
    let changed = DrawingStyle::new(
        text_style(),
        "Device labels",
        DrawingRole::Text,
        StylePatch {
            text_height: Some(length(6_000)),
            ..StylePatch::default()
        },
    )
    .unwrap();
    run(&mut session, DrawingCommand::ReplaceStyle(changed));
    assert_eq!(
        session
            .project()
            .property(label(1), PropertyId::TextHeight)
            .unwrap()
            .value,
        PropertyValue::Length(length(3_000))
    );
    assert_eq!(
        session
            .project()
            .property(label(2), PropertyId::TextHeight)
            .unwrap()
            .value,
        PropertyValue::Length(length(6_000))
    );
    let explicit = session.project().clone();
    let plan = property_patch_plan(
        &explicit,
        [label(1)],
        BTreeMap::from([(PropertyId::TextHeight, PropertyEdit::Reset)]),
    )
    .unwrap();
    session
        .execute_batch(session.revision(), plan.commands)
        .unwrap();
    assert_eq!(
        session
            .project()
            .property(label(1), PropertyId::TextHeight)
            .unwrap()
            .origin,
        Some(PropertyOrigin::NamedStyle(text_style()))
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &explicit);
}

#[test]
fn mixed_graphical_and_electrical_selections_edit_only_applicable_subsets_and_expose_origins() {
    let mut session = setup();
    set(
        &mut session,
        label(1),
        PropertyId::DrawingColour,
        PropertyValue::Colour(DrawingColour::default()),
    );
    let electrical = ObjectRef::Device(DeviceId::new(1).unwrap());
    let selection = [label(1), label(2), path(), electrical];
    let inspector = inspect_properties(session.project(), selection).unwrap();
    let row = &inspector.properties[&PropertyId::DrawingColour];
    assert!(matches!(row.value, SelectionValue::Uniform(_)));
    assert_eq!(row.origins[&label(1)], PropertyOrigin::Object);
    assert_eq!(row.origins[&label(2)], PropertyOrigin::Project);
    assert_eq!(row.inapplicable, [electrical].into());
    let plan = property_edit_plan(
        session.project(),
        selection,
        BTreeMap::from([
            (
                PropertyId::FontFamily,
                PropertyValue::Text("Project font request".into()),
            ),
            (PropertyId::StrokeWidth, PropertyValue::Length(length(800))),
            (
                PropertyId::DrawingColour,
                PropertyValue::Colour(DrawingColour {
                    red: 0,
                    green: 0,
                    blue: 255,
                }),
            ),
        ]),
    )
    .unwrap();
    assert_eq!(plan.changes.len(), 6);
    assert_eq!(plan.inapplicable.len(), 6);
    let before = session.project().clone();
    session
        .execute_batch(session.revision(), plan.commands)
        .unwrap();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}

#[test]
fn shared_updates_have_focused_persistence_and_do_not_rewrite_dependent_objects() {
    let mut session = setup();
    let before: serde_json::Value =
        serde_json::from_str(&encode_project(session.project()).unwrap()).unwrap();
    let changed = DrawingStyle::new(
        text_style(),
        "Device labels",
        DrawingRole::Text,
        StylePatch {
            text_height: Some(length(7_000)),
            ..StylePatch::default()
        },
    )
    .unwrap();
    run(&mut session, DrawingCommand::ReplaceStyle(changed));
    let after: serde_json::Value =
        serde_json::from_str(&encode_project(session.project()).unwrap()).unwrap();
    assert_eq!(before["drawing"]["labels"], after["drawing"]["labels"]);
    assert_eq!(
        before["drawing"]["polylines"],
        after["drawing"]["polylines"]
    );
    assert_ne!(before["drawing"]["styles"], after["drawing"]["styles"]);
    assert_eq!(
        session
            .project()
            .property(label(1), PropertyId::FontFamily)
            .unwrap()
            .origin,
        Some(PropertyOrigin::Project)
    );
}

#[test]
fn graphical_identity_units_and_whole_geometry_are_validated_without_electrical_changes() {
    let mut session = setup();
    let electrical = session.project().devices().cloned().collect::<Vec<_>>();
    let before = session.project().clone();
    assert!(DrawingLength::new(0).is_err());
    assert!(FontFamily::new("  ").is_err());
    assert!(
        session
            .execute(
                session.revision(),
                Command::SetProperty {
                    target: path(),
                    property: PropertyId::DrawingVertices,
                    value: PropertyValue::Points(vec![DrawingPoint::default(); 2])
                }
            )
            .is_err()
    );
    assert_eq!(session.project(), &before);
    set(
        &mut session,
        label(1),
        PropertyId::DrawingPosition,
        PropertyValue::Point(DrawingPoint {
            x: -7_000,
            y: 12_000,
        }),
    );
    assert_eq!(
        session.project().devices().cloned().collect::<Vec<_>>(),
        electrical
    );
    for target in [label(1), path(), ObjectRef::View(view())] {
        for property in PROPERTY_IDS {
            if property.applies_to(target) {
                let value = session.project().property(target, property).unwrap();
                assert!(property.descriptor().accepts_value(&value.value));
            }
        }
    }
    assert_eq!(
        PropertyId::TextHeight.descriptor().ownership,
        PropertyOwnership::Presentation
    );
    assert_ne!(
        PropertyId::TextHeight.descriptor().key,
        PropertyId::StrokeWidth.descriptor().key
    );
}

#[test]
fn stale_wrong_role_missing_reference_and_in_use_changes_leave_state_unchanged() {
    let mut session = setup();
    let before = session.project().clone();
    for command in [
        DrawingCommand::RemoveStyle(text_style()),
        DrawingCommand::RemoveView(view()),
        DrawingCommand::ReplaceStyle(
            DrawingStyle::new(
                text_style(),
                "Wrong kind",
                DrawingRole::Stroke,
                StylePatch::default(),
            )
            .unwrap(),
        ),
    ] {
        assert!(
            session
                .execute(session.revision(), Command::Drawing(command))
                .is_err()
        );
        assert_eq!(session.project(), &before);
    }
    for style in [Some(stroke_style()), Some(StyleId::new(999).unwrap())] {
        assert!(
            session
                .execute(
                    session.revision(),
                    Command::SetProperty {
                        target: label(1),
                        property: PropertyId::DrawingStyle,
                        value: PropertyValue::Style(style)
                    }
                )
                .is_err()
        );
        assert_eq!(session.project(), &before);
    }
    assert!(
        session
            .execute(
                session.revision(),
                Command::SetProperty {
                    target: label(1),
                    property: PropertyId::DrawingView,
                    value: PropertyValue::Object(ObjectRef::View(ViewId::new(999).unwrap()))
                }
            )
            .is_err()
    );
    assert_eq!(session.project(), &before);
    let mut settings = session.project().drawing().settings().clone();
    settings.text_creation.style = Some(stroke_style());
    assert!(
        session
            .execute(
                session.revision(),
                Command::Drawing(DrawingCommand::Settings(settings))
            )
            .is_err()
    );
    assert_eq!(session.project(), &before);
}

#[test]
fn intent_changes_and_settings_roll_back_atomically_and_noops_preserve_redo() {
    let mut session = setup();
    let before = session.project().clone();
    let mut settings = before.drawing().settings().clone();
    settings.base.text_height = length(9_000);
    let commands = vec![
        Command::Drawing(DrawingCommand::Settings(settings)),
        Command::SetProperty {
            target: label(1),
            property: PropertyId::TextHeight,
            value: PropertyValue::Length(length(3_000)),
        },
        Command::Drawing(DrawingCommand::RemoveView(view())),
    ];
    assert!(session.execute_batch(session.revision(), commands).is_err());
    assert_eq!(session.project(), &before);
    set(
        &mut session,
        label(1),
        PropertyId::TextHeight,
        PropertyValue::Length(length(3_000)),
    );
    session.undo(session.revision()).unwrap();
    let redo = session.redo_len();
    let revision = session.revision();
    let reset = property_patch_plan(
        session.project(),
        [label(1)],
        BTreeMap::from([(PropertyId::TextHeight, PropertyEdit::Reset)]),
    )
    .unwrap();
    assert!(!session.execute_batch(revision, reset.commands).unwrap());
    assert_eq!(session.redo_len(), redo);
    assert_eq!(session.revision(), revision);
    assert!(
        property_patch_plan(
            session.project(),
            [ObjectRef::Device(DeviceId::new(1).unwrap())],
            BTreeMap::from([(PropertyId::Designation, PropertyEdit::Reset)])
        )
        .is_err()
    );
}
