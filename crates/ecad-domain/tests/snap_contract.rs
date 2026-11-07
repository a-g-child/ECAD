//! Exact document-space snap rules independent of a renderer, electrical model, or host.
use ecad_domain::*;
use std::collections::BTreeSet;

fn point(x: i64, y: i64) -> DrawingPoint {
    DrawingPoint { x, y }
}
fn view() -> ViewId {
    ViewId::new(1).unwrap()
}
fn anchor(id: u128) -> AnchorId {
    AnchorId::new(id).unwrap()
}
fn setup() -> DrawingDocument {
    let mut drawing = DrawingDocument::default();
    drawing
        .insert_view(DrawingView {
            id: view(),
            name: "Snapping".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    drawing
}
fn placed(drawing: &mut DrawingDocument, id: u128, position: DrawingPoint) {
    drawing
        .insert_terminal_anchor(TerminalAnchor {
            id: anchor(id),
            view: view(),
            terminal: TerminalId::new(id).unwrap(),
            position,
        })
        .unwrap();
}
fn line(drawing: &mut DrawingDocument, id: u128, points: Vec<DrawingPoint>) {
    drawing
        .insert_polyline(DrawingPolyline {
            id: PolylineId::new(id).unwrap(),
            view: view(),
            points,
            appearance: Appearance::default(),
        })
        .unwrap();
}
fn query(position: DrawingPoint, tolerance_um: u32) -> SnapQuery {
    SnapQuery {
        view: view(),
        position,
        settings: SnapSettings {
            tolerance_um,
            terminals: true,
            endpoints: true,
            midpoints: true,
            grid: Some(DrawingGrid {
                origin: point(0, 0),
                spacing: DrawingLength::new(10).unwrap(),
            }),
        },
        excluded: BTreeSet::new(),
    }
}

#[test]
fn source_priority_precedes_distance_and_distance_precedes_stable_typed_identity() {
    let mut drawing = setup();
    for (id, position) in [(3, point(-3, 0)), (1, point(4, 0)), (2, point(3, 0))] {
        placed(&mut drawing, id, position);
    }
    line(&mut drawing, 10, vec![point(0, 0), point(10, 0)]);
    let before = drawing.clone();
    let candidates = drawing.snap_candidates(&query(point(0, 0), 10)).unwrap();
    assert_eq!(
        candidates.iter().map(|c| c.target).collect::<Vec<_>>(),
        vec![
            SnapTarget::Terminal(anchor(2)),
            SnapTarget::Terminal(anchor(3)),
            SnapTarget::Terminal(anchor(1)),
            SnapTarget::PolylineEndpoint {
                polyline: PolylineId::new(10).unwrap(),
                end: SnapEnd::First
            },
            SnapTarget::PolylineEndpoint {
                polyline: PolylineId::new(10).unwrap(),
                end: SnapEnd::Last
            },
            SnapTarget::PolylineMidpoint {
                polyline: PolylineId::new(10).unwrap(),
                segment: 0
            },
            SnapTarget::Grid
        ]
    );
    assert_eq!(candidates[0].distance_squared_um, 9);
    for _ in 0..20 {
        assert_eq!(
            drawing.snap_candidates(&query(point(0, 0), 10)).unwrap(),
            candidates
        );
    }
    assert_eq!(drawing, before);
}

#[test]
fn tolerance_is_an_inclusive_circle_and_zero_requires_an_exact_point() {
    let mut drawing = setup();
    placed(&mut drawing, 1, point(3, 4));
    placed(&mut drawing, 2, point(4, 4));
    placed(&mut drawing, 3, point(0, 0));
    let mut q = query(point(0, 0), 5);
    q.settings.grid = None;
    assert_eq!(
        drawing
            .snap_candidates(&q)
            .unwrap()
            .iter()
            .map(|c| c.target)
            .collect::<Vec<_>>(),
        vec![
            SnapTarget::Terminal(anchor(3)),
            SnapTarget::Terminal(anchor(1))
        ]
    );
    q.settings.tolerance_um = 0;
    assert_eq!(drawing.snap_candidates(&q).unwrap().len(), 1);
    q.settings = SnapSettings::default();
    assert!(drawing.snap_candidates(&q).unwrap().is_empty());
}

#[test]
fn exclusions_are_representation_scoped_and_excluded_anchors_cannot_reappear_as_route_endpoints() {
    let mut drawing = setup();
    placed(&mut drawing, 1, point(0, 0));
    placed(&mut drawing, 2, point(10, 0));
    let route = RouteId::new(1).unwrap();
    drawing
        .insert_route(DrawingRoute {
            id: route,
            view: view(),
            conductor: ConductorId::new(1).unwrap(),
            anchors: [Some(anchor(1)), Some(anchor(2))],
            points: vec![point(0, 0), point(10, 0)],
            mode: RouteMode::Orthogonal,
            appearance: Appearance::default(),
        })
        .unwrap();
    let other = ViewId::new(2).unwrap();
    drawing
        .insert_view(DrawingView {
            id: other,
            name: "Other".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    drawing
        .insert_terminal_anchor(TerminalAnchor {
            id: anchor(3),
            view: other,
            terminal: TerminalId::new(1).unwrap(),
            position: point(0, 0),
        })
        .unwrap();
    let mut q = query(point(0, 0), 10);
    q.settings.midpoints = false;
    q.settings.grid = None;
    q.excluded.insert(ObjectRef::Anchor(anchor(1)));
    assert_eq!(
        drawing
            .snap_candidates(&q)
            .unwrap()
            .iter()
            .map(|c| c.target)
            .collect::<Vec<_>>(),
        vec![
            SnapTarget::Terminal(anchor(2)),
            SnapTarget::RouteEndpoint {
                route,
                end: SnapEnd::Last
            }
        ]
    );
    q.excluded.insert(ObjectRef::Route(route));
    q.excluded.insert(ObjectRef::Anchor(anchor(2)));
    assert!(drawing.snap_candidates(&q).unwrap().is_empty());
    q.view = ViewId::new(99).unwrap();
    assert_eq!(
        drawing.snap_candidates(&q),
        Err(DrawingError::MissingView(q.view))
    );
}

#[test]
fn coincident_targets_remain_distinct_for_explicit_cycling_and_distance_orders_source_peers() {
    let mut drawing = setup();
    line(&mut drawing, 1, vec![point(4, 0), point(0, 0)]);
    line(&mut drawing, 2, vec![point(0, 0), point(8, 0)]);
    let mut q = query(point(0, 0), 10);
    q.settings.terminals = false;
    q.settings.midpoints = false;
    let candidates = drawing.snap_candidates(&q).unwrap();
    assert_eq!(
        candidates[0].target,
        SnapTarget::PolylineEndpoint {
            polyline: PolylineId::new(1).unwrap(),
            end: SnapEnd::Last
        }
    );
    assert_eq!(
        candidates[1].target,
        SnapTarget::PolylineEndpoint {
            polyline: PolylineId::new(2).unwrap(),
            end: SnapEnd::First
        }
    );
    assert_eq!(candidates.last().unwrap().target, SnapTarget::Grid);
    q.excluded
        .insert(ObjectRef::Polyline(PolylineId::new(1).unwrap()));
    assert_eq!(drawing.snap_candidates(&q).unwrap().len(), 3);
}

#[test]
fn grids_handle_negative_half_ties_offset_origins_and_signed_range_boundaries() {
    let drawing = setup();
    let mut q = query(point(-5, -5), 10);
    assert_eq!(
        drawing.snap_candidates(&q).unwrap()[0].position,
        point(-10, -10)
    );
    q.position = point(8, 8);
    q.settings.grid.as_mut().unwrap().origin = point(3, 3);
    assert_eq!(
        drawing.snap_candidates(&q).unwrap()[0].position,
        point(3, 3)
    );
    q.position = point(i64::MIN, i64::MIN);
    q.settings.grid.as_mut().unwrap().origin = point(i64::MAX, i64::MAX);
    assert_eq!(
        drawing.snap_candidates(&q).unwrap()[0].position,
        point(i64::MIN + 5, i64::MIN + 5)
    );
    q.position = point(i64::MAX, i64::MAX);
    q.settings.grid = Some(DrawingGrid {
        origin: point(i64::MIN, i64::MIN),
        spacing: DrawingLength::new(3).unwrap(),
    });
    q.settings.tolerance_um = 0;
    assert_eq!(drawing.snap_candidates(&q).unwrap()[0].position, q.position);
}

#[test]
fn midpoint_and_distance_math_remain_exact_at_full_coordinate_and_tolerance_spans() {
    let mut drawing = setup();
    line(
        &mut drawing,
        1,
        vec![point(i64::MIN, i64::MIN), point(i64::MAX, i64::MAX)],
    );
    line(&mut drawing, 2, vec![point(-2, 0), point(-1, -1)]);
    let mut q = query(point(0, 0), 3);
    q.settings.grid = None;
    let midpoints: Vec<_> = drawing
        .snap_candidates(&q)
        .unwrap()
        .into_iter()
        .filter(|c| matches!(c.target, SnapTarget::PolylineMidpoint { .. }))
        .collect();
    assert_eq!(midpoints[0].position, point(-1, -1));
    assert_eq!(midpoints[1].position, point(-2, -1));
    placed(&mut drawing, 1, point(i64::from(u32::MAX), 0));
    placed(
        &mut drawing,
        2,
        point(i64::from(u32::MAX), i64::from(u32::MAX)),
    );
    q.settings = SnapSettings {
        terminals: true,
        tolerance_um: u32::MAX,
        ..SnapSettings::default()
    };
    let candidates = drawing.snap_candidates(&q).unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates[0].distance_squared_um,
        u128::from(u32::MAX).pow(2)
    );
}
