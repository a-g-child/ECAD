//! Whole-path orthogonal group transforms with exact previews, connections, and localized history.
use ecad_adapters::{JsonFile, decode_project, encode_project};
use ecad_app::*;
use ecad_domain::*;
use std::collections::BTreeSet;
#[path = "transforms/actions.rs"]
mod actions;
#[path = "transforms/failures.rs"]
mod failures;
fn point(x: i64, y: i64) -> DrawingPoint {
    DrawingPoint { x, y }
}
fn anchor(id: u128) -> AnchorId {
    AnchorId::new(id).unwrap()
}
fn route(id: u128) -> RouteId {
    RouteId::new(id).unwrap()
}
fn label() -> LabelId {
    LabelId::new(1).unwrap()
}
fn line() -> PolylineId {
    PolylineId::new(1).unwrap()
}
fn view() -> ViewId {
    ViewId::new(1).unwrap()
}
fn rotate() -> DrawingTransform {
    DrawingTransform::Rotate {
        pivot: point(0, 0),
        turn: QuarterTurn::Positive90,
    }
}
fn setup() -> Project {
    let mut project = decode_project(include_str!("fixtures/routed-conveyor-v6.json")).unwrap();
    project
        .insert_drawing_polyline(DrawingPolyline {
            id: line(),
            view: view(),
            points: vec![point(0, 0), point(100, 0), point(100, 200)],
            appearance: Appearance::default(),
        })
        .unwrap();
    project
        .insert_drawing_label(DrawingLabel {
            id: label(),
            view: view(),
            position: point(200, 100),
            text: "Keep text upright".into(),
            appearance: Appearance::default(),
        })
        .unwrap();
    let mut path = project.drawing().route(route(1)).unwrap().clone();
    path.points = vec![
        point(0, 0),
        point(0, 500),
        point(5000, 500),
        point(10000, 500),
        point(10000, 0),
    ];
    path.appearance.overrides.stroke_width = Some(DrawingLength::new(250).unwrap());
    project.replace_drawing_route(path).unwrap();
    project
}
fn selection() -> BTreeSet<ObjectRef> {
    [
        ObjectRef::Anchor(anchor(1)),
        ObjectRef::Anchor(anchor(2)),
        ObjectRef::Anchor(anchor(3)),
        ObjectRef::Label(label()),
        ObjectRef::Polyline(line()),
    ]
    .into_iter()
    .collect()
}
fn commit(
    session: &mut Session,
    selection: impl IntoIterator<Item = ObjectRef>,
    transform: DrawingTransform,
) -> LayoutPreview {
    let preview =
        preview_layout(session, selection, LayoutOperation::Transform(transform)).unwrap();
    session.apply_layout_preview(preview.clone()).unwrap();
    preview
}
fn assert_projection(project: &Project, preview: &LayoutPreview) {
    for record in preview.objects() {
        match record {
            PreviewObject::Anchor(a) => {
                assert_eq!(project.drawing().terminal_anchor(a.id), Some(a))
            }
            PreviewObject::Route(r) => assert_eq!(project.drawing().route(r.id), Some(r)),
            PreviewObject::Label(l) => assert_eq!(project.drawing().label(l.id), Some(l)),
            PreviewObject::Polyline(l) => assert_eq!(project.drawing().polyline(l.id), Some(l)),
        }
    }
}
#[test]
fn group_rotation_preserves_every_internal_bend_binding_style_and_exact_preview_record() {
    let before = setup();
    let mut session = Session::new(before.clone());
    let preview = commit(&mut session, selection(), rotate());
    assert_projection(session.project(), &preview);
    assert_eq!(preview.geometry_changed().len(), 4); // Anchor 1 remains at the pivot; the path's origin also stays fixed.
    assert_eq!(preview.changes().len(), 3);
    assert_eq!(
        preview.affected_routes(),
        &BTreeSet::from([route(1), route(2)])
    );
    for original in before.drawing().routes() {
        let mut expected = original.clone();
        expected.points = original
            .points
            .iter()
            .map(|p| rotate().point(*p).unwrap())
            .collect();
        assert_eq!(
            session.project().drawing().route(original.id),
            Some(&expected)
        );
    }
    assert_eq!(
        session.project().drawing().polyline(line()).unwrap().points,
        vec![point(0, 0), point(0, 100), point(-200, 100)]
    );
    assert_eq!(
        session.project().drawing().label(label()).unwrap().text,
        "Keep text upright"
    );
    assert_eq!(
        session.project().conductors().collect::<Vec<_>>(),
        before.conductors().collect::<Vec<_>>()
    );
    assert_eq!(session.undo_len(), 1);
    let after = session.project().clone();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    session.redo(session.revision()).unwrap();
    assert_eq!(session.project(), &after);
}
#[test]
fn mirror_transforms_paths_without_swapping_route_end_order_or_electrical_bindings() {
    let before = setup();
    let mut session = Session::new(before.clone());
    let mirror = DrawingTransform::Mirror {
        pivot: point(25, 30),
        axis: MirrorAxis::Vertical,
    };
    let preview = commit(&mut session, selection(), mirror);
    assert_projection(session.project(), &preview);
    for original in before.drawing().routes() {
        let moved = session.project().drawing().route(original.id).unwrap();
        assert_eq!(moved.anchors, original.anchors);
        assert_eq!(moved.conductor, original.conductor);
        assert_eq!(moved.appearance, original.appearance);
        assert_eq!(
            moved.points,
            original
                .points
                .iter()
                .map(|p| mirror.point(*p).unwrap())
                .collect::<Vec<_>>()
        );
    }
    commit(&mut session, selection(), mirror);
    assert_eq!(session.project(), &before);
    session.undo(session.revision()).unwrap();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}
#[test]
fn rotating_a_path_about_its_first_vertex_is_an_edit_even_when_its_origin_is_unchanged() {
    let before = setup();
    let mut session = Session::new(before.clone());
    let preview = commit(&mut session, [ObjectRef::Polyline(line())], rotate());
    assert!(preview.changes().is_empty());
    assert!(preview.unchanged().is_empty());
    assert_eq!(
        preview.geometry_changed(),
        &BTreeSet::from([ObjectRef::Polyline(line())])
    );
    assert_eq!(session.undo_len(), 1);
    assert_ne!(session.project(), &before);
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}
#[test]
fn a_self_loop_rotates_completely_even_when_its_only_anchor_remains_at_the_pivot() {
    let mut project = setup();
    project.remove_drawing_route(route(1)).unwrap();
    let conductor = ConductorId::new(3).unwrap();
    project
        .insert_conductor(Conductor::new(
            conductor,
            Some(TerminalId::new(101).unwrap()),
            Some(TerminalId::new(101).unwrap()),
        ))
        .unwrap();
    let path = DrawingRoute {
        id: route(3),
        view: view(),
        conductor,
        anchors: [Some(anchor(1)), Some(anchor(1))],
        points: vec![
            point(0, 0),
            point(500, 0),
            point(500, 500),
            point(0, 500),
            point(0, 0),
        ],
        mode: RouteMode::Orthogonal,
        appearance: Appearance::default(),
    };
    project.insert_drawing_route(path.clone()).unwrap();
    let before = project.clone();
    let mut session = Session::new(project);
    let preview = commit(&mut session, [ObjectRef::Anchor(anchor(1))], rotate());
    assert!(preview.changes().is_empty());
    assert!(preview.geometry_changed().is_empty());
    assert_eq!(preview.unchanged().len(), 1);
    assert_eq!(preview.affected_routes(), &BTreeSet::from([route(3)]));
    assert_eq!(session.undo_len(), 1);
    assert_eq!(
        session.project().drawing().route(route(3)).unwrap().points,
        path.points
            .iter()
            .map(|p| rotate().point(*p).unwrap())
            .collect::<Vec<_>>()
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}
#[test]
fn partially_selected_connections_adjust_locally_and_other_views_are_unchanged() {
    let mut project = setup();
    let second = ViewId::new(2).unwrap();
    project
        .insert_drawing_view(DrawingView {
            id: second,
            name: "Independent".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    for id in [1, 2] {
        let mut a = project
            .drawing()
            .terminal_anchor(anchor(id))
            .unwrap()
            .clone();
        a.id = anchor(id + 10);
        a.view = second;
        project.insert_terminal_anchor(a).unwrap();
    }
    let mut independent = project.drawing().route(route(1)).unwrap().clone();
    independent.id = route(10);
    independent.view = second;
    independent.anchors = [Some(anchor(11)), Some(anchor(12))];
    project.insert_drawing_route(independent.clone()).unwrap();
    let before = project.clone();
    let mut session = Session::new(project);
    let preview = commit(
        &mut session,
        [ObjectRef::Anchor(anchor(1)), ObjectRef::Anchor(anchor(2))],
        rotate(),
    );
    assert_projection(session.project(), &preview);
    let local = session.project().drawing().route(route(2)).unwrap();
    assert_eq!(
        local.points,
        vec![point(0, 10000), point(20000, 10000), point(20000, 0)]
    );
    assert_eq!(
        session.project().drawing().route(route(10)),
        Some(&independent)
    );
    session.project().validate().unwrap();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}
#[test]
fn repeated_quarter_turns_preserve_collinear_vertices_and_return_exactly_to_original_geometry() {
    let before = setup();
    let mut session = Session::new(before.clone());
    for _ in 0..4 {
        commit(&mut session, selection(), rotate());
    }
    assert_eq!(session.project(), &before);
    for _ in 0..4 {
        session.undo(session.revision()).unwrap();
    }
    assert_eq!(session.project(), &before);
}
#[test]
fn transformed_v6_geometry_saves_reopens_and_restores_the_saved_checkpoint_on_redo() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("transform.json"));
    let before = setup();
    let mut document = Document::new(before.clone(), store.clone());
    document.save().unwrap();
    let preview = preview_layout(
        document.session(),
        selection(),
        LayoutOperation::Transform(rotate()),
    )
    .unwrap();
    document.apply_layout_preview(preview.clone()).unwrap();
    document.save().unwrap();
    let reopened = Document::open(store).unwrap();
    assert_projection(reopened.session().project(), &preview);
    let data: serde_json::Value =
        serde_json::from_str(&encode_project(reopened.session().project()).unwrap()).unwrap();
    assert_eq!(data["schema_version"], 6);
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &before);
    assert!(document.session().is_dirty());
    document.redo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), reopened.session().project());
    assert!(!document.session().is_dirty());
}

#[test]
fn a_free_electrical_end_is_not_implicitly_carried_by_a_selected_terminal_transform() {
    let mut project = setup();
    let conductor = ConductorId::new(3).unwrap();
    project
        .insert_conductor(Conductor::new(
            conductor,
            Some(TerminalId::new(101).unwrap()),
            None,
        ))
        .unwrap();
    let free = point(1500, 2500);
    project
        .insert_drawing_route(DrawingRoute {
            id: route(3),
            view: view(),
            conductor,
            anchors: [Some(anchor(1)), None],
            points: vec![point(0, 0), free],
            mode: RouteMode::Free,
            appearance: Appearance::default(),
        })
        .unwrap();
    let before = project.clone();
    let mut session = Session::new(project);
    let stationary = preview_layout(
        &session,
        [ObjectRef::Anchor(anchor(1))],
        LayoutOperation::Transform(rotate()),
    )
    .unwrap();
    assert!(!session.apply_layout_preview(stationary).unwrap());
    let transform = DrawingTransform::Rotate {
        pivot: point(0, 10000),
        turn: QuarterTurn::Positive90,
    };
    commit(&mut session, [ObjectRef::Anchor(anchor(1))], transform);
    let path = session.project().drawing().route(route(3)).unwrap();
    assert_eq!(path.points, vec![point(10000, 10000), free]);
    assert_eq!(path.anchors, [Some(anchor(1)), None]);
    assert_eq!(
        session
            .project()
            .conductor(conductor)
            .unwrap()
            .endpoint(End::B),
        None
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}
