//! Route schema, strict continuity, and local-history persistence regressions.

use super::*;
use serde_json::{Value, json};

#[test]
fn route_layout_roundtrips_and_exact_undo_tracks_saved_checkpoints() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("routes.json"));
    let mut document = Document::new(setup().project().clone(), store.clone());
    document.save().unwrap();
    let before = document.session().project().clone();
    document
        .execute(
            document.session().revision(),
            Command::Route(RouteCommand::MoveAnchor {
                anchor: anchor(2),
                position: point(10_000, 5_000),
            }),
        )
        .unwrap();
    document.save().unwrap();
    assert_eq!(
        Document::open(store).unwrap().session().project(),
        document.session().project()
    );
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &before);
    assert!(document.session().is_dirty());
    document.redo(document.session().revision()).unwrap();
    assert!(!document.session().is_dirty());
}

#[test]
fn malformed_references_geometry_duplicates_and_wrong_electrical_continuity_are_rejected() {
    let text = encode_project(setup().project()).unwrap();
    assert_eq!(text, include_str!("../fixtures/routed-conveyor-v10.json"));
    let data: Value = serde_json::from_str(&text).unwrap();
    for (pointer, value) in [
        (
            "/drawing/anchors/0/terminal",
            json!("00000000000000000000000000000999"),
        ),
        (
            "/drawing/routes/0/conductor",
            json!("00000000000000000000000000000999"),
        ),
        ("/drawing/routes/0/anchors/0", json!(null)),
        (
            "/drawing/routes/0/anchors/0",
            json!("00000000000000000000000000000003"),
        ),
        (
            "/drawing/routes/0/points_um",
            json!([[0, 0], [1, 1], [10000, 0]]),
        ),
        ("/drawing/routes/0/mode", json!("unknown")),
        ("/schema_version", json!(5)),
    ] {
        let mut invalid = data.clone();
        *invalid.pointer_mut(pointer).unwrap() = value;
        assert!(
            decode_project(&invalid.to_string()).is_err(),
            "accepted {pointer}"
        );
    }
    for field in ["anchors", "routes"] {
        let mut invalid = data.clone();
        let duplicate = invalid["drawing"][field][0].clone();
        invalid["drawing"][field]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        assert!(decode_project(&invalid.to_string()).is_err());
    }
    let mut reordered = data;
    for field in ["anchors", "routes"] {
        reordered["drawing"][field]
            .as_array_mut()
            .unwrap()
            .reverse();
    }
    assert_eq!(
        encode_project(&decode_project(&reordered.to_string()).unwrap()).unwrap(),
        text
    );
}

#[test]
fn duplicate_same_view_representations_cross_view_anchors_and_anchor_device_deletion_are_guarded() {
    let mut project = setup().project().clone();
    let before = project.clone();
    let mut duplicate = project.drawing().route(route(1)).unwrap().clone();
    duplicate.id = route(99);
    assert!(project.insert_drawing_route(duplicate).is_err());
    assert_eq!(project, before);
    let second = ViewId::new(2).unwrap();
    project
        .insert_drawing_view(DrawingView {
            id: second,
            name: "Second".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    let mut invalid = project.drawing().route(route(1)).unwrap().clone();
    invalid.view = second;
    invalid.id = route(99);
    let before = project.clone();
    assert!(project.insert_drawing_route(invalid).is_err());
    assert_eq!(project, before);
    let loose = DeviceId::new(50).unwrap();
    let terminal = TerminalId::new(500).unwrap();
    project
        .insert_device(
            Device::new(loose, "-Loose", [Terminal::new(terminal, "1").unwrap()]).unwrap(),
        )
        .unwrap();
    project
        .insert_terminal_anchor(TerminalAnchor {
            id: anchor(50),
            view: view(),
            terminal,
            position: point(1, 1),
        })
        .unwrap();
    assert!(matches!(
        project.remove_device(loose),
        Err(ModelError::Route(RouteError::DeviceAnchored(_)))
    ));
}

#[test]
fn old_v5_drawings_upgrade_without_inventing_routes_and_free_paths_retain_explicit_free_ends() {
    let text = include_str!("../fixtures/styled-overview-v5.json");
    let mut project = decode_project(text).unwrap();
    assert_eq!(project.drawing().routes().len(), 0);
    assert_eq!(project.drawing().terminal_anchors().len(), 0);
    let wire = ConductorId::new(90).unwrap();
    project
        .insert_conductor(Conductor::new(wire, None, None))
        .unwrap();
    let free = DrawingRoute {
        id: route(90),
        view: ViewId::new(1).unwrap(),
        conductor: wire,
        anchors: [None, None],
        points: vec![point(1, 2), point(3, 4)],
        mode: RouteMode::Free,
        appearance: Appearance::default(),
    };
    project.insert_drawing_route(free.clone()).unwrap();
    assert!(
        free.edited(&RouteEdit::MoveSegment {
            index: 0,
            coordinate_um: 10
        })
        .is_err()
    );
    assert_eq!(
        decode_project(&encode_project(&project).unwrap()).unwrap(),
        project
    );
}

#[test]
fn adopting_changed_internal_connections_requires_explicit_resolution_of_existing_view_routes() {
    use ecad_adapters::RandomIds;
    use ecad_app::{Session, adoption_plan, instance_commands};
    use std::sync::Arc;
    let body = decode_project(include_str!("../fixtures/sensor-module-v3.json")).unwrap();
    let source = DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(1).unwrap(),
    );
    let revision = Arc::new(
        SubsystemRevision::new(source, SubsystemId::new(10).unwrap(), body.clone()).unwrap(),
    );
    let mut project = Project::new(ProjectId::new(123).unwrap());
    project.insert_definition(Arc::clone(&revision)).unwrap();
    project
        .insert_drawing_view(DrawingView {
            id: view(),
            name: "Instance layout".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    let ids = RandomIds::instance_ids(&revision, InstanceLimits::default()).unwrap();
    let root = ids.subsystems[&revision.root()];
    let mut session = Session::new(project);
    let commands = instance_commands(
        session.project(),
        source,
        ids.clone(),
        None,
        "Sensor",
        InstanceLimits::default(),
    )
    .unwrap();
    session.execute_batch(session.revision(), commands).unwrap();
    for (id, local, x) in [(1, 201, 0), (2, 301, 10_000)] {
        run(
            &mut session,
            RouteCommand::InsertAnchor(TerminalAnchor {
                id: anchor(id),
                view: view(),
                terminal: ids.terminals[&TerminalId::new(local).unwrap()],
                position: point(x, 0),
            }),
        );
    }
    run(
        &mut session,
        RouteCommand::Insert(DrawingRoute {
            id: route(1),
            view: view(),
            conductor: ids.conductors[&ConductorId::new(2).unwrap()],
            anchors: [Some(anchor(1)), Some(anchor(2))],
            points: vec![point(0, 0), point(10_000, 0)],
            mode: RouteMode::Orthogonal,
            appearance: Appearance::default(),
        }),
    );
    let mut changed = body;
    changed
        .set_termination(ConductorId::new(2).unwrap(), End::B, None)
        .unwrap();
    let next = DefinitionRef::new(source.definition(), DefinitionRevisionId::new(2).unwrap());
    let revision =
        Arc::new(SubsystemRevision::new(next, SubsystemId::new(10).unwrap(), changed).unwrap());
    session
        .execute(
            session.revision(),
            Command::InsertDefinition(Arc::clone(&revision)),
        )
        .unwrap();
    let before = session.project().clone();
    let mapping = RandomIds::instance_ids(&revision, InstanceLimits::default()).unwrap();
    assert!(matches!(
        adoption_plan(
            session.project(),
            root,
            next,
            mapping.clone(),
            InstanceLimits::default()
        ),
        Err(ModelError::Route(RouteError::AttachmentConflict(_)))
    ));
    assert_eq!(session.project(), &before);
    run(&mut session, RouteCommand::Remove(route(1)));
    let plan = adoption_plan(
        session.project(),
        root,
        next,
        mapping,
        InstanceLimits::default(),
    )
    .unwrap();
    session.execute(session.revision(), plan.command).unwrap();
    session.project().validate().unwrap();
    session.undo(session.revision()).unwrap();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}

#[test]
fn route_action_requests_existing_conductor_and_uses_separate_route_creation_defaults() {
    use ecad_app::*;
    let mut project = setup().project().clone();
    project.remove_drawing_route(route(1)).unwrap();
    let mut settings = project.drawing().settings().clone();
    settings.route_creation.overrides.stroke_width = Some(DrawingLength::new(450).unwrap());
    settings.stroke_creation.overrides.stroke_width = Some(DrawingLength::new(900).unwrap());
    project.set_drawing_settings(settings).unwrap();
    assert_eq!(
        decode_project(&encode_project(&project).unwrap()).unwrap(),
        project
    );
    let directory = tempfile::tempdir().unwrap();
    let mut document = Document::new(
        project,
        JsonFile::new(directory.path().join("action-route.json")),
    );
    document.save().unwrap();
    let before = document.session().project().clone();
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    let context = ActionContext {
        view: Some(view()),
        ..ActionContext::for_session(document.session())
    };
    let invocation = ActionInvocation {
        input_id: InputId::new(1).unwrap(),
        action: ActionId::new("drawing.route.create").unwrap(),
        source: InvocationSource::Keyboard,
        repeat: false,
        context,
        parameters: ActionParameters::None,
    };
    let ActionOutcome::Request(request) = dispatcher
        .invoke(&registry, invocation, Some(&mut document))
        .unwrap()
    else {
        panic!("no route placement request");
    };
    assert_eq!(request.kind, ActionRequestKind::RoutePlacement);
    assert_eq!(document.session().project(), &before);
    let reply = ActionInvocation {
        input_id: InputId::new(2).unwrap(),
        action: request.action,
        source: InvocationSource::Palette,
        repeat: false,
        context: request.context,
        parameters: ActionParameters::Route {
            id: route(1),
            conductor: ConductorId::new(1).unwrap(),
            anchors: [Some(anchor(1)), Some(anchor(2))],
            points: vec![point(0, 0), point(10_000, 0)],
            mode: RouteMode::Orthogonal,
        },
    };
    assert!(matches!(
        dispatcher.invoke(&registry, reply, Some(&mut document)),
        Ok(ActionOutcome::Edited(true))
    ));
    assert_eq!(
        document
            .session()
            .project()
            .drawing()
            .route(route(1))
            .unwrap()
            .appearance
            .overrides
            .stroke_width,
        Some(DrawingLength::new(450).unwrap())
    );
    assert_eq!(document.session().project().conductors().len(), 2);
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &before);
    assert!(!document.session().is_dirty());
}
