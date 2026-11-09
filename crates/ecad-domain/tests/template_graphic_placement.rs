//! Pointer primitive bounds and whole-object translation remain deterministic and exact.
use ecad_domain::*;
#[test]
fn reverse_drag_retains_ordered_line_and_movement_keeps_all_local_content() {
    let paper = SheetPaper::default();
    let id = TemplateGraphicId::new(1).unwrap();
    let line = TemplateGraphic::between(
        id,
        TemplatePrimitive::Line,
        [50000, 40000],
        [10000, 40000],
        &paper,
    )
    .unwrap();
    assert_eq!(
        line.bounds,
        FrameRect {
            x: 10000,
            y: 40000,
            width: 40000,
            height: 0
        }
    );
    assert_eq!(
        line.shape,
        TemplateGraphicShape::Polyline(vec![[40000, 0], [0, 0]])
    );
    let moved = line.placed_at([0, 1000], &paper).unwrap();
    assert_eq!(moved.shape, line.shape);
    assert_eq!(moved.id, line.id);
    for kind in [
        TemplatePrimitive::Text,
        TemplatePrimitive::Rectangle,
        TemplatePrimitive::Ellipse,
    ] {
        let g = TemplateGraphic::between(id, kind, [50000, 40000], [10000, 20000], &paper).unwrap();
        assert_eq!(
            g.bounds,
            FrameRect {
                x: 10000,
                y: 20000,
                width: 40000,
                height: 20000
            }
        );
        let moved = g.placed_at([20000, 50000], &paper).unwrap();
        assert_eq!(moved.shape, g.shape);
        assert_eq!(moved.font, g.font);
    }
}
#[test]
fn collapsed_negative_extreme_and_outside_geometry_reject_without_clamping() {
    let paper = SheetPaper::default();
    let id = TemplateGraphicId::new(1).unwrap();
    for kind in [
        TemplatePrimitive::Text,
        TemplatePrimitive::Line,
        TemplatePrimitive::Rectangle,
        TemplatePrimitive::Ellipse,
    ] {
        for (a, b) in [
            ([0, 0], [0, 0]),
            ([-1, 0], [10, 10]),
            ([0, 0], [i64::MAX, 10]),
            ([0, 0], [300000, 210000]),
        ] {
            assert!(TemplateGraphic::between(id, kind, a, b, &paper).is_err());
        }
    }
    let g = TemplateGraphic::between(id, TemplatePrimitive::Text, [0, 0], [40000, 20000], &paper)
        .unwrap();
    assert!(g.placed_at([-1, 0], &paper).is_err());
    let moved = g.placed_at([280000, 200000], &paper).unwrap();
    assert_eq!(moved.bounds.width, g.bounds.width);
    assert_eq!(moved.shape, g.shape);
    assert!(g.placed_at([i64::from(paper.width), 0], &paper).is_err());
    assert_eq!(g.bounds.x, 0);
}

#[test]
fn multi_vertex_paths_preserve_order_closure_zero_extent_axes_and_checked_limits() {
    let paper = SheetPaper::default();
    let id = TemplateGraphicId::new(1).unwrap();
    let points = [
        [50000, 40000],
        [10000, 40000],
        [10000, 20000],
        [50000, 40000],
    ];
    let g = TemplateGraphic::from_vertices(id, &points, &paper).unwrap();
    assert_eq!(
        g.bounds,
        FrameRect {
            x: 10000,
            y: 20000,
            width: 40000,
            height: 20000
        }
    );
    assert_eq!(
        g.shape,
        TemplateGraphicShape::Polyline(vec![[40000, 20000], [0, 20000], [0, 0], [40000, 20000]])
    );
    assert_eq!(g.placed_at([0, 0], &paper).unwrap().shape, g.shape);
    assert_eq!(
        TemplateGraphic::from_vertices(id, &[[0, 0], [0, 100], [0, 200]], &paper)
            .unwrap()
            .bounds
            .width,
        0
    );
    for points in [
        vec![],
        vec![[0, 0]],
        vec![[0, 0], [0, 0]],
        vec![[-1, 0], [1, 0]],
        vec![[0, 0], [i64::MAX, 0]],
        vec![[0, 0], [300000, 0]],
    ] {
        assert!(TemplateGraphic::from_vertices(id, &points, &paper).is_err());
    }
    let points: Vec<_> = (0..4096).map(|i| [i, 0]).collect();
    assert!(TemplateGraphic::from_vertices(id, &points, &paper).is_ok());
    let mut oversized = points;
    oversized.push([4096, 0]);
    assert!(TemplateGraphic::from_vertices(id, &oversized, &paper).is_err());
}
