//! Whole-route editing, local anchor propagation, exact history, and electrical guards.

#[path = "routes/persistence.rs"]
mod persistence;
#[path = "routes/support.rs"]
mod support;
use ecad_adapters::{JsonFile, decode_project, encode_project};
use ecad_app::{Command, Document, RouteCommand, inspect_properties, property_edit_plan};
use ecad_domain::*;
use std::collections::BTreeMap;
use support::*;

#[test]
fn segment_drag_and_waypoint_edits_retain_whole_identity_and_have_exact_undo() {
    let mut session = setup();
    let before = session.project().clone();
    run(
        &mut session,
        RouteCommand::Edit {
            route: route(1),
            edit: RouteEdit::MoveSegment {
                index: 0,
                coordinate_um: 4_000,
            },
        },
    );
    let changed = session.project().drawing().route(route(1)).unwrap();
    assert_eq!(
        changed.points,
        [
            point(0, 0),
            point(0, 4_000),
            point(10_000, 4_000),
            point(10_000, 0)
        ]
    );
    assert_eq!(changed.anchors, [Some(anchor(1)), Some(anchor(2))]);
    assert_eq!(changed.conductor, ConductorId::new(1).unwrap());
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    run(
        &mut session,
        RouteCommand::Edit {
            route: route(1),
            edit: RouteEdit::InsertVertex {
                index: 1,
                position: point(5_000, 0),
            },
        },
    );
    run(
        &mut session,
        RouteCommand::Edit {
            route: route(1),
            edit: RouteEdit::RemoveVertex { index: 1 },
        },
    );
    assert_eq!(session.project(), &before);
    session.undo(session.revision()).unwrap();
    assert_eq!(
        session
            .project()
            .drawing()
            .route(route(1))
            .unwrap()
            .points
            .len(),
        3
    );
}

#[test]
fn moving_shared_anchor_adjusts_only_touching_routes_and_undo_restores_original_bends() {
    let mut session = setup();
    let before = session.project().clone();
    let history = session.undo_len();
    run(
        &mut session,
        RouteCommand::MoveAnchor {
            anchor: anchor(2),
            position: point(10_000, 5_000),
        },
    );
    assert_eq!(session.undo_len(), history + 1);
    assert_eq!(
        session.project().drawing().route(route(1)).unwrap().points,
        [point(0, 0), point(0, 5_000), point(10_000, 5_000)]
    );
    assert_eq!(
        session.project().drawing().route(route(2)).unwrap().points,
        [point(10_000, 5_000), point(20_000, 5_000), point(20_000, 0)]
    );
    assert_eq!(
        session.project().conductors().collect::<Vec<_>>(),
        before.conductors().collect::<Vec<_>>()
    );
    session.project().validate().unwrap();
    let after = session.project().clone();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    session.redo(session.revision()).unwrap();
    assert_eq!(session.project(), &after);
}

#[test]
fn views_have_independent_geometry_without_duplicate_electrical_terminals_or_conductors() {
    let mut session = setup();
    let second = ViewId::new(2).unwrap();
    session
        .execute(
            session.revision(),
            Command::Drawing(ecad_app::DrawingCommand::InsertView(DrawingView {
                id: second,
                name: "Other view".into(),
                overrides: StylePatch::default(),
            })),
        )
        .unwrap();
    for (id, terminal, position) in [(4, 101, point(0, 10_000)), (5, 201, point(10_000, 10_000))] {
        run(
            &mut session,
            RouteCommand::InsertAnchor(TerminalAnchor {
                id: anchor(id),
                view: second,
                terminal: TerminalId::new(terminal).unwrap(),
                position,
            }),
        );
    }
    let independent = DrawingRoute {
        id: route(3),
        view: second,
        conductor: ConductorId::new(1).unwrap(),
        anchors: [Some(anchor(4)), Some(anchor(5))],
        points: vec![point(0, 10_000), point(10_000, 10_000)],
        mode: RouteMode::Orthogonal,
        appearance: Appearance::default(),
    };
    run(&mut session, RouteCommand::Insert(independent.clone()));
    run(
        &mut session,
        RouteCommand::MoveAnchor {
            anchor: anchor(2),
            position: point(10_000, 5_000),
        },
    );
    assert_eq!(
        session.project().drawing().route(route(3)),
        Some(&independent)
    );
    assert_eq!(
        session
            .project()
            .devices()
            .flat_map(Device::terminals)
            .count(),
        3
    );
    assert_eq!(session.project().conductors().len(), 2);
    assert_eq!(
        session
            .project()
            .drawing()
            .conductor_routes(ConductorId::new(1).unwrap())
            .count(),
        2
    );
}

#[test]
fn invalid_geometry_attached_vertex_moves_and_collapsed_anchor_moves_reject_without_mutation() {
    let mut session = setup();
    let before = session.project().clone();
    for edit in [
        RouteEdit::MoveVertex {
            index: 0,
            position: point(1, 1),
        },
        RouteEdit::RemoveVertex { index: 0 },
        RouteEdit::InsertVertex {
            index: 1,
            position: point(5_000, 5_000),
        },
        RouteEdit::MoveSegment {
            index: usize::MAX,
            coordinate_um: 0,
        },
    ] {
        assert!(
            session
                .execute(
                    session.revision(),
                    Command::Route(RouteCommand::Edit {
                        route: route(1),
                        edit
                    })
                )
                .is_err()
        );
        assert_eq!(session.project(), &before);
    }
    assert!(
        session
            .execute(
                session.revision(),
                Command::Route(RouteCommand::MoveAnchor {
                    anchor: anchor(1),
                    position: point(10_000, 0)
                })
            )
            .is_err()
    );
    assert_eq!(session.project(), &before);
    let mut forged = before.drawing().anchor_layout(anchor(2)).unwrap();
    forged.routes.pop();
    assert!(
        session
            .execute(
                session.revision(),
                Command::Route(RouteCommand::RestoreAnchorLayout(forged))
            )
            .is_err()
    );
    assert_eq!(session.project(), &before);
}

#[test]
fn electrical_retermination_and_deletion_require_explicit_route_resolution() {
    let mut session = setup();
    let before = session.project().clone();
    assert!(
        session
            .execute(
                session.revision(),
                Command::SetTermination {
                    conductor: ConductorId::new(1).unwrap(),
                    end: End::B,
                    terminal: None
                }
            )
            .is_err()
    );
    assert!(
        session
            .execute(
                session.revision(),
                Command::RemoveConductor(ConductorId::new(2).unwrap())
            )
            .is_err()
    );
    assert!(
        session
            .execute(
                session.revision(),
                Command::Route(RouteCommand::RemoveAnchor(anchor(2)))
            )
            .is_err()
    );
    assert!(
        session
            .execute(
                session.revision(),
                Command::Drawing(ecad_app::DrawingCommand::RemoveView(view()))
            )
            .is_err()
    );
    assert_eq!(session.project(), &before);
    let mut changed = before.drawing().route(route(1)).unwrap().clone();
    changed.anchors[1] = None;
    session
        .execute_batch(
            session.revision(),
            vec![
                Command::Route(RouteCommand::Remove(route(1))),
                Command::SetTermination {
                    conductor: ConductorId::new(1).unwrap(),
                    end: End::B,
                    terminal: None,
                },
                Command::Route(RouteCommand::Insert(changed)),
            ],
        )
        .unwrap();
    session.project().validate().unwrap();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}

#[test]
fn route_style_and_whole_vertices_use_shared_property_history_and_anchor_positions_require_layout_edits()
 {
    let mut session = setup();
    let selection = [ObjectRef::Route(route(1)), ObjectRef::Anchor(anchor(1))];
    let inspector = inspect_properties(session.project(), selection).unwrap();
    assert_eq!(
        inspector.properties[&PropertyId::StrokeWidth]
            .applicable
            .len(),
        1
    );
    assert!(
        inspector.properties[&PropertyId::DrawingPosition]
            .editable
            .is_empty()
    );
    assert_eq!(
        inspector.properties[&PropertyId::RouteConductor]
            .read_only
            .len(),
        1
    );
    let before = session.project().clone();
    let plan = property_edit_plan(
        &before,
        selection,
        BTreeMap::from([(
            PropertyId::StrokeWidth,
            PropertyValue::Length(DrawingLength::new(700).unwrap()),
        )]),
    )
    .unwrap();
    session
        .execute_batch(session.revision(), plan.commands)
        .unwrap();
    assert_eq!(
        session
            .project()
            .property(ObjectRef::Route(route(1)), PropertyId::StrokeWidth)
            .unwrap()
            .origin,
        Some(PropertyOrigin::Object)
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    assert!(
        property_edit_plan(
            &before,
            [ObjectRef::Route(route(1))],
            BTreeMap::from([(
                PropertyId::DrawingVertices,
                PropertyValue::Points(vec![point(1, 1), point(10_000, 0)])
            )])
        )
        .is_err()
    );
}

#[test]
fn failed_batches_roll_back_exact_local_layout_and_noop_moves_preserve_redo() {
    let mut session = setup();
    let before = session.project().clone();
    let revision = session.revision();
    assert!(
        session
            .execute_batch(
                revision,
                vec![
                    Command::Route(RouteCommand::MoveAnchor {
                        anchor: anchor(2),
                        position: point(10_000, 5_000)
                    }),
                    Command::RemoveDevice(DeviceId::new(999).unwrap())
                ]
            )
            .is_err()
    );
    assert_eq!(session.project(), &before);
    assert_eq!(session.revision(), revision);
    run(
        &mut session,
        RouteCommand::MoveAnchor {
            anchor: anchor(2),
            position: point(10_000, 5_000),
        },
    );
    session.undo(session.revision()).unwrap();
    let redo = session.redo_len();
    let revision = session.revision();
    assert!(
        !session
            .execute(
                revision,
                Command::Route(RouteCommand::MoveAnchor {
                    anchor: anchor(2),
                    position: point(10_000, 0)
                })
            )
            .unwrap()
    );
    assert_eq!(session.redo_len(), redo);
    assert_eq!(session.revision(), revision);
}

#[test]
fn repeated_anchor_moves_do_not_accumulate_elbows_and_preserve_collinear_waypoints() {
    let mut session = setup();
    run(
        &mut session,
        RouteCommand::Edit {
            route: route(1),
            edit: RouteEdit::InsertVertex {
                index: 1,
                position: point(5_000, 0),
            },
        },
    );
    let before = session.project().clone();
    for _ in 0..20 {
        run(
            &mut session,
            RouteCommand::MoveAnchor {
                anchor: anchor(2),
                position: point(10_000, 5_000),
            },
        );
        run(
            &mut session,
            RouteCommand::MoveAnchor {
                anchor: anchor(2),
                position: point(10_000, 0),
            },
        );
        assert_eq!(session.project(), &before);
    }
    for _ in 0..40 {
        session.undo(session.revision()).unwrap();
    }
    assert_eq!(session.project(), &before);
}

#[test]
fn one_anchor_at_both_route_ends_keeps_a_single_indexed_route_and_exact_loop_undo() {
    let mut session = setup();
    session
        .execute(
            session.revision(),
            Command::InsertConductor(Conductor::new(
                ConductorId::new(3).unwrap(),
                Some(TerminalId::new(101).unwrap()),
                Some(TerminalId::new(101).unwrap()),
            )),
        )
        .unwrap();
    run(
        &mut session,
        RouteCommand::Insert(DrawingRoute {
            id: route(3),
            view: view(),
            conductor: ConductorId::new(3).unwrap(),
            anchors: [Some(anchor(1)), Some(anchor(1))],
            points: vec![
                point(0, 0),
                point(5_000, 0),
                point(5_000, 5_000),
                point(0, 5_000),
                point(0, 0),
            ],
            mode: RouteMode::Orthogonal,
            appearance: Appearance::default(),
        }),
    );
    let before = session.project().clone();
    assert_eq!(
        before
            .drawing()
            .anchor_layout(anchor(1))
            .unwrap()
            .routes
            .len(),
        2
    );
    run(
        &mut session,
        RouteCommand::MoveAnchor {
            anchor: anchor(1),
            position: point(-5_000, -5_000),
        },
    );
    session.project().validate().unwrap();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}
