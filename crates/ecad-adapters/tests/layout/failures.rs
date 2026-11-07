//! Invalid layout selection/geometry and localized rollback regressions.
use super::*;

#[test]
fn unsupported_missing_empty_mixed_view_and_insufficient_selections_are_explicit_errors() {
    let mut project = setup();
    let op = LayoutOperation::Translate(point(1, 1));
    assert_eq!(
        layout_plan(&project, [], op).err(),
        Some(LayoutError::EmptySelection)
    );
    for target in [
        ObjectRef::Route(route(1)),
        ObjectRef::Device(DeviceId::new(1).unwrap()),
        ObjectRef::View(view()),
    ] {
        assert_eq!(
            layout_plan(&project, [ObjectRef::Label(label(1)), target], op).err(),
            Some(LayoutError::UnsupportedTarget(target))
        );
    }
    assert!(matches!(
        layout_plan(&project, [ObjectRef::Label(label(999))], op),
        Err(LayoutError::Model(ModelError::Drawing(
            DrawingError::MissingLabel(_)
        )))
    ));
    assert_eq!(
        layout_plan(
            &project,
            [ObjectRef::Label(label(1)); 3],
            LayoutOperation::Distribute(LayoutAxis::X)
        )
        .err(),
        Some(LayoutError::TooFewObjects)
    );
    let second = ViewId::new(2).unwrap();
    project
        .insert_drawing_view(DrawingView {
            id: second,
            name: "Other".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    let mut item = project.drawing().label(label(2)).unwrap().clone();
    item.view = second;
    project.replace_drawing_label(item).unwrap();
    assert_eq!(
        layout_plan(
            &project,
            [ObjectRef::Label(label(1)), ObjectRef::Label(label(2))],
            op
        )
        .err(),
        Some(LayoutError::MixedViews)
    );
}

#[test]
fn overflow_at_origins_and_interior_vertices_or_collapsed_routes_never_changes_the_source() {
    let mut project = setup();
    let mut line = project.drawing().polyline(line()).unwrap().clone();
    line.points[1].x = i64::MAX;
    project.replace_drawing_polyline(line).unwrap();
    let before = project.clone();
    assert!(matches!(
        layout_plan(
            &project,
            [
                ObjectRef::Polyline(super::line()),
                ObjectRef::Label(label(1))
            ],
            LayoutOperation::Translate(point(1, 0))
        ),
        Err(LayoutError::Model(ModelError::Drawing(
            DrawingError::CoordinateOverflow
        )))
    ));
    assert_eq!(project, before);
    let mut item = project.drawing().label(label(1)).unwrap().clone();
    item.position.x = i64::MIN;
    project.replace_drawing_label(item).unwrap();
    assert!(
        layout_plan(
            &project,
            [ObjectRef::Label(label(1))],
            LayoutOperation::Translate(point(-1, 0))
        )
        .is_err()
    );
    let before = project.clone();
    assert!(
        layout_plan(
            &project,
            [ObjectRef::Anchor(anchor(1)), ObjectRef::Anchor(anchor(2))],
            LayoutOperation::Align {
                axis: LayoutAxis::X,
                coordinate_um: 10_000
            }
        )
        .is_err()
    );
    assert_eq!(project, before);
}

#[test]
fn direct_group_move_checks_route_interior_overflow_before_changing_any_anchor() {
    let mut project = setup();
    let mut path = project.drawing().route(route(1)).unwrap().clone();
    path.points = vec![
        point(0, 0),
        point(i64::MAX, 0),
        point(i64::MAX, 1),
        point(10_000, 1),
        point(10_000, 0),
    ];
    project.replace_drawing_route(path).unwrap();
    let before = project.clone();
    assert!(
        project
            .move_terminal_anchors(BTreeMap::from([
                (anchor(1), point(1, 0)),
                (anchor(2), point(10_001, 0))
            ]))
            .is_err()
    );
    assert_eq!(project, before);
}

#[test]
fn differential_endpoint_move_collapsing_intermediate_geometry_returns_error_without_panic() {
    let mut project = setup();
    let before = project.clone();
    assert!(
        project
            .move_terminal_anchors(BTreeMap::from([
                (anchor(1), point(10_000, 0)),
                (anchor(2), point(15_000, 0))
            ]))
            .is_err()
    );
    assert_eq!(project, before);
}

#[test]
fn malformed_multi_anchor_snapshots_cannot_omit_routes_duplicate_records_or_change_bindings() {
    let mut project = setup();
    let baseline = project
        .drawing()
        .route_layout([anchor(1), anchor(2)])
        .unwrap();
    assert_eq!(baseline.routes.len(), 2);
    let mut variants = Vec::new();
    let mut bad = baseline.clone();
    bad.anchors.push(bad.anchors[0].clone());
    variants.push(bad);
    let mut bad = baseline.clone();
    bad.routes.pop();
    variants.push(bad);
    let mut bad = baseline.clone();
    bad.routes.push(bad.routes[0].clone());
    variants.push(bad);
    let mut bad = baseline.clone();
    bad.anchors[0].terminal = TerminalId::new(301).unwrap();
    variants.push(bad);
    let mut bad = baseline.clone();
    bad.routes[0].appearance.overrides.stroke_width = Some(DrawingLength::new(999).unwrap());
    variants.push(bad);
    let mut bad = baseline.clone();
    bad.routes[0].conductor = ConductorId::new(2).unwrap();
    variants.push(bad);
    let mut bad = baseline;
    bad.anchors.clear();
    variants.push(bad);
    for bad in variants {
        let before = project.clone();
        assert!(project.restore_route_layout(bad).is_err());
        assert_eq!(project, before);
    }
}

#[test]
fn failed_batch_restores_multi_anchor_geometry_and_noop_layout_retains_redo_and_revision() {
    let before = setup();
    let mut session = Session::new(before.clone());
    let selection = [
        ObjectRef::Anchor(anchor(1)),
        ObjectRef::Anchor(anchor(2)),
        ObjectRef::Label(label(1)),
    ];
    let mut plan = layout_plan(
        &before,
        selection,
        LayoutOperation::Translate(point(100, 200)),
    )
    .unwrap();
    plan.commands
        .push(Command::RemoveDevice(DeviceId::new(999).unwrap()));
    let revision = session.revision();
    assert!(session.execute_batch(revision, plan.commands).is_err());
    assert_eq!(session.project(), &before);
    assert_eq!(session.revision(), revision);
    commit(
        &mut session,
        selection,
        LayoutOperation::Translate(point(100, 200)),
    );
    session.undo(session.revision()).unwrap();
    let revision = session.revision();
    let redo = session.redo_len();
    let plan = layout_plan(
        session.project(),
        selection,
        LayoutOperation::Translate(point(0, 0)),
    )
    .unwrap();
    assert!(plan.commands.is_empty());
    assert_eq!(plan.unchanged.len(), 3);
    assert!(!session.execute_batch(revision, plan.commands).unwrap());
    assert_eq!(session.revision(), revision);
    assert_eq!(session.redo_len(), redo);
    assert_eq!(session.project(), &before);
}
