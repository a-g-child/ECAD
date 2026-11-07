//! Coordinate limits, atomic rollback, and fixed-geometry no-op handling.
use super::*;
#[test]
fn interior_vertex_overflow_rejects_entire_selection_and_direct_anchor_transform_atomically() {
    let mut project = setup();
    let mut item = project.drawing().polyline(line()).unwrap().clone();
    item.points = vec![point(0, 0), point(i64::MIN, 0)];
    project.replace_drawing_polyline(item).unwrap();
    let mirror = DrawingTransform::Mirror {
        pivot: point(0, 0),
        axis: MirrorAxis::Vertical,
    };
    let before = project.clone();
    assert!(matches!(
        layout_plan(&project, selection(), LayoutOperation::Transform(mirror)),
        Err(LayoutError::Model(ModelError::Drawing(
            DrawingError::CoordinateOverflow
        )))
    ));
    assert_eq!(project, before);
    let mut path = project.drawing().route(route(1)).unwrap().clone();
    path.points = vec![
        point(0, 0),
        point(i64::MIN, 0),
        point(i64::MIN, 100),
        point(10000, 100),
        point(10000, 0),
    ];
    project.replace_drawing_route(path).unwrap();
    let before = project.clone();
    assert!(
        project
            .transform_terminal_anchors(BTreeSet::from([anchor(1), anchor(2)]), mirror)
            .is_err()
    );
    assert_eq!(project, before);
}
#[test]
fn a_failed_batch_restores_transformed_routes_paths_and_annotations_without_consuming_revision() {
    let before = setup();
    let mut session = Session::new(before.clone());
    let mut plan = layout_plan(&before, selection(), LayoutOperation::Transform(rotate())).unwrap();
    plan.commands
        .push(Command::RemoveDevice(DeviceId::new(999).unwrap()));
    let revision = session.revision();
    assert!(session.execute_batch(revision, plan.commands).is_err());
    assert_eq!(session.project(), &before);
    assert_eq!(session.revision(), revision);
    assert_eq!(session.undo_len(), 0);
}
#[test]
fn fixed_anchor_and_pivot_label_noops_have_no_commands_or_affected_routes_and_preserve_redo() {
    let before = setup();
    let mut session = Session::new(before.clone());
    commit(&mut session, selection(), rotate());
    session.undo(session.revision()).unwrap();
    let revision = session.revision();
    let redo = session.redo_len();
    for (target, transform) in [
        (ObjectRef::Anchor(anchor(1)), rotate()),
        (
            ObjectRef::Label(label()),
            DrawingTransform::Rotate {
                pivot: point(200, 100),
                turn: QuarterTurn::Half,
            },
        ),
    ] {
        let plan = layout_plan(
            session.project(),
            [target],
            LayoutOperation::Transform(transform),
        )
        .unwrap();
        assert!(plan.commands.is_empty());
        assert!(plan.affected_routes.is_empty());
        assert!(plan.geometry_changed.is_empty());
        assert_eq!(plan.unchanged, BTreeSet::from([target]));
        let preview =
            preview_layout(&session, [target], LayoutOperation::Transform(transform)).unwrap();
        assert!(!session.apply_layout_preview(preview).unwrap());
    }
    assert_eq!(session.revision(), revision);
    assert_eq!(session.redo_len(), redo);
    assert_eq!(session.project(), &before);
}
