//! Numeric layout contracts and exact connected-route history, without a desktop host.

use ecad_adapters::{JsonFile, decode_project, encode_project};
use ecad_app::*;
use ecad_domain::*;
use std::collections::{BTreeMap, BTreeSet};

#[path = "layout/actions.rs"]
mod actions;
#[path = "layout/failures.rs"]
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
fn label(id: u128) -> LabelId {
    LabelId::new(id).unwrap()
}
fn line() -> PolylineId {
    PolylineId::new(1).unwrap()
}
fn view() -> ViewId {
    ViewId::new(1).unwrap()
}
fn setup() -> Project {
    let mut project = decode_project(include_str!("fixtures/routed-conveyor-v6.json")).unwrap();
    for (id, position) in [
        (1, point(0, 10)),
        (2, point(3, 20)),
        (3, point(3, 30)),
        (4, point(10, 40)),
    ] {
        project
            .insert_drawing_label(DrawingLabel {
                id: label(id),
                view: view(),
                text: format!("Label {id}"),
                position,
                appearance: Appearance::default(),
            })
            .unwrap();
    }
    project
        .insert_drawing_polyline(DrawingPolyline {
            id: line(),
            view: view(),
            points: vec![point(2, 4), point(12, 4), point(12, 9)],
            appearance: Appearance::default(),
        })
        .unwrap();
    project
}
fn commit(
    session: &mut Session,
    selection: impl IntoIterator<Item = ObjectRef>,
    operation: LayoutOperation,
) -> LayoutPlan {
    let plan = layout_plan(session.project(), selection, operation).unwrap();
    session
        .execute_batch(session.revision(), plan.commands.clone())
        .unwrap();
    plan
}

#[test]
fn mixed_translation_moves_shared_routes_once_without_intermediate_endpoint_collapse() {
    let project = setup();
    let before = project.clone();
    let mut session = Session::new(project);
    let selection = [
        ObjectRef::Anchor(anchor(1)),
        ObjectRef::Anchor(anchor(2)),
        ObjectRef::Anchor(anchor(3)),
        ObjectRef::Label(label(1)),
        ObjectRef::Polyline(line()),
    ];
    // Anchor A lands on the old B position. Sequential single-anchor commands would collapse.
    let plan = commit(
        &mut session,
        selection,
        LayoutOperation::Translate(point(10_000, 0)),
    );
    assert_eq!(plan.changes.len(), 5);
    assert_eq!(plan.affected_routes, BTreeSet::from([route(1), route(2)]));
    let drawing = session.project().drawing();
    assert_eq!(
        drawing.route(route(1)).unwrap().points,
        vec![point(10_000, 0), point(20_000, 0)]
    );
    assert_eq!(drawing.label(label(1)).unwrap().position, point(10_000, 10));
    assert_eq!(
        drawing.polyline(line()).unwrap().points,
        vec![point(10_002, 4), point(10_012, 4), point(10_012, 9)]
    );
    assert_eq!(session.undo_len(), 1);
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
fn moving_both_ends_translates_all_bends_and_retains_style_intent_and_unrelated_views() {
    let mut project = setup();
    let mut path = project.drawing().route(route(1)).unwrap().clone();
    path.points = vec![
        point(0, 0),
        point(0, 500),
        point(10_000, 500),
        point(10_000, 0),
    ];
    path.appearance.overrides.stroke_width = Some(DrawingLength::new(250).unwrap());
    project.replace_drawing_route(path.clone()).unwrap();
    let mut second = project.drawing().view(view()).unwrap().clone();
    second.id = ViewId::new(2).unwrap();
    project.insert_drawing_view(second).unwrap();
    for id in [1, 2] {
        let mut placed = project
            .drawing()
            .terminal_anchor(anchor(id))
            .unwrap()
            .clone();
        placed.id = anchor(id + 10);
        placed.view = ViewId::new(2).unwrap();
        project.insert_terminal_anchor(placed).unwrap();
    }
    let mut independent = path.clone();
    independent.id = route(10);
    independent.view = ViewId::new(2).unwrap();
    independent.anchors = [Some(anchor(11)), Some(anchor(12))];
    project.insert_drawing_route(independent.clone()).unwrap();
    let before = project.clone();
    let mut session = Session::new(project);
    commit(
        &mut session,
        [ObjectRef::Anchor(anchor(1)), ObjectRef::Anchor(anchor(2))],
        LayoutOperation::Translate(point(-20, 30)),
    );
    let moved = session.project().drawing().route(route(1)).unwrap();
    assert_eq!(
        moved.points,
        path.points
            .iter()
            .map(|p| p.translated(point(-20, 30)).unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(moved.appearance, path.appearance);
    assert_eq!(
        session.project().drawing().route(route(10)),
        Some(&independent)
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}

#[test]
fn numeric_alignment_uses_origins_preserves_other_axis_and_moves_all_route_ends_together() {
    let before = setup();
    let mut session = Session::new(before.clone());
    let selection = [
        ObjectRef::Anchor(anchor(1)),
        ObjectRef::Anchor(anchor(2)),
        ObjectRef::Anchor(anchor(3)),
        ObjectRef::Label(label(1)),
        ObjectRef::Polyline(line()),
    ];
    commit(
        &mut session,
        selection,
        LayoutOperation::Align {
            axis: LayoutAxis::Y,
            coordinate_um: 17,
        },
    );
    let drawing = session.project().drawing();
    assert_eq!(
        drawing.polyline(line()).unwrap().points,
        vec![point(2, 17), point(12, 17), point(12, 22)]
    );
    assert_eq!(drawing.label(label(1)).unwrap().position, point(0, 17));
    for path in drawing.routes() {
        assert!(path.points.iter().all(|p| p.y == 17));
    }
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}

#[test]
fn distribution_is_order_independent_deduplicated_and_uses_stable_identity_ties() {
    let project = setup();
    let a = [4, 2, 1, 3, 2].map(|id| ObjectRef::Label(label(id)));
    let b = [1, 2, 3, 4].map(|id| ObjectRef::Label(label(id)));
    let operation = LayoutOperation::Distribute(LayoutAxis::X);
    let first = layout_plan(&project, a, operation).unwrap();
    let second = layout_plan(&project, b, operation).unwrap();
    assert_eq!(first.commands, second.commands);
    assert_eq!(first.changes, second.changes);
    assert_eq!(
        first.unchanged,
        BTreeSet::from([
            ObjectRef::Label(label(1)),
            ObjectRef::Label(label(2)),
            ObjectRef::Label(label(4))
        ])
    );
    let mut session = Session::new(project.clone());
    session
        .execute_batch(session.revision(), first.commands)
        .unwrap();
    let positions: Vec<_> = session
        .project()
        .drawing()
        .labels()
        .map(|l| l.position)
        .collect();
    assert_eq!(
        positions,
        vec![point(0, 10), point(3, 20), point(6, 30), point(10, 40)]
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &project);
}

#[test]
fn distribution_handles_the_full_signed_coordinate_span_without_overflow_or_grid_rounding() {
    let mut project = setup();
    for (id, y) in [(1, i64::MIN), (2, 0), (3, 0), (4, i64::MAX)] {
        let mut item = project.drawing().label(label(id)).unwrap().clone();
        item.position.y = y;
        project.replace_drawing_label(item).unwrap();
    }
    let mut session = Session::new(project.clone());
    commit(
        &mut session,
        (1..=4).map(|id| ObjectRef::Label(label(id))),
        LayoutOperation::Distribute(LayoutAxis::Y),
    );
    let values: Vec<_> = session
        .project()
        .drawing()
        .labels()
        .map(|l| l.position.y)
        .collect();
    assert_eq!(
        values,
        vec![
            i64::MIN,
            -3_074_457_345_618_258_603,
            3_074_457_345_618_258_602,
            i64::MAX
        ]
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &project);
}

#[test]
fn layout_save_reopen_and_exact_undo_track_saved_checkpoints_without_schema_changes() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("layout.json"));
    let before = setup();
    let mut document = Document::new(before.clone(), store.clone());
    document.save().unwrap();
    let plan = layout_plan(
        document.session().project(),
        [ObjectRef::Anchor(anchor(1)), ObjectRef::Anchor(anchor(2))],
        LayoutOperation::Translate(point(100, 200)),
    )
    .unwrap();
    document
        .execute_batch(document.session().revision(), plan.commands)
        .unwrap();
    document.save().unwrap();
    let after = document.session().project().clone();
    assert_eq!(Document::open(store).unwrap().session().project(), &after);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&encode_project(&after).unwrap()).unwrap()["schema_version"],
        13
    );
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &before);
    assert!(document.session().is_dirty());
    document.redo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &after);
    assert!(!document.session().is_dirty());
}

#[test]
fn alignment_allows_wide_offsets_when_every_resulting_vertex_is_representable() {
    let mut project = setup();
    let mut item = project.drawing().polyline(line()).unwrap().clone();
    item.points = vec![point(i64::MIN, 0), point(i64::MIN + 1, 0)];
    project.replace_drawing_polyline(item).unwrap();
    let before = project.clone();
    let mut session = Session::new(project);
    commit(
        &mut session,
        [ObjectRef::Polyline(line())],
        LayoutOperation::Align {
            axis: LayoutAxis::X,
            coordinate_um: i64::MAX - 1,
        },
    );
    assert_eq!(
        session.project().drawing().polyline(line()).unwrap().points,
        vec![point(i64::MAX - 1, 0), point(i64::MAX, 0)]
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}
