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
    assert!(g.placed_at([280000, 200000], &paper).is_err());
    assert_eq!(g.bounds.x, 0);
}
