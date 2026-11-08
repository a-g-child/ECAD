//! Exact border snaps and explicit layout fitting preserve cell geometry and bounds.
use ecad_domain::*;
fn rect(x: u32, y: u32, width: u32, height: u32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
#[test]
fn move_snaps_corresponding_edges_and_corners_with_inclusive_tolerance_and_stable_ties() {
    let border = rect(10, 10, 100, 100);
    let table = rect(30, 30, 20, 20);
    for (x, y, bounds, edges) in [
        (12, 32, rect(10, 32, 20, 20), vec![MarginEdge::Left]),
        (88, 32, rect(90, 32, 20, 20), vec![MarginEdge::Right]),
        (32, 12, rect(32, 10, 20, 20), vec![MarginEdge::Top]),
        (32, 88, rect(32, 90, 20, 20), vec![MarginEdge::Bottom]),
        (
            8,
            92,
            rect(10, 90, 20, 20),
            vec![MarginEdge::Left, MarginEdge::Bottom],
        ),
    ] {
        let p =
            table_border_placement(table, border, TableBorderDrag::Move { x, y }, Some(2)).unwrap();
        assert_eq!(p.bounds, bounds);
        assert_eq!(p.snapped, edges);
    }
    let free = table_border_placement(
        table,
        border,
        TableBorderDrag::Move { x: 13, y: 30 },
        Some(2),
    )
    .unwrap();
    assert_eq!(free.bounds.x, 13);
    assert!(free.snapped.is_empty());
    let bypass =
        table_border_placement(table, border, TableBorderDrag::Move { x: 12, y: 30 }, None)
            .unwrap();
    assert_eq!(bypass.bounds.x, 12);
    assert!(bypass.snapped.is_empty());
    let tie = table_border_placement(
        rect(15, 15, 80, 80),
        border,
        TableBorderDrag::Move { x: 20, y: 20 },
        Some(10),
    )
    .unwrap();
    assert_eq!(tie.bounds, rect(10, 10, 80, 80));
    assert_eq!(tie.snapped, vec![MarginEdge::Left, MarginEdge::Top]);
}
#[test]
fn resizing_snaps_only_the_dragged_side_retains_opposite_edges_and_checks_extremes() {
    let border = rect(10, 10, 100, 100);
    let table = rect(30, 30, 40, 40);
    for (edge, coordinate, bounds) in [
        (MarginEdge::Left, 8, rect(10, 30, 60, 40)),
        (MarginEdge::Right, 112, rect(30, 30, 80, 40)),
        (MarginEdge::Top, 8, rect(30, 10, 40, 60)),
        (MarginEdge::Bottom, 112, rect(30, 30, 40, 80)),
    ] {
        let p = table_border_placement(
            table,
            border,
            TableBorderDrag::Resize { edge, coordinate },
            Some(2),
        )
        .unwrap();
        assert_eq!(p.bounds, bounds);
        assert_eq!(p.snapped, vec![edge]);
        assert!(
            table_border_placement(
                table,
                border,
                TableBorderDrag::Resize { edge, coordinate },
                None
            )
            .is_err()
        );
    }
    for coordinate in [i64::MIN, i64::MAX, 0, 30] {
        assert!(
            table_border_placement(
                table,
                border,
                TableBorderDrag::Resize {
                    edge: MarginEdge::Right,
                    coordinate
                },
                Some(2)
            )
            .is_err()
        );
    }
    assert!(
        table_border_placement(
            table,
            border,
            TableBorderDrag::Move { x: i64::MAX, y: 10 },
            Some(u32::MAX)
        )
        .is_err()
    );
    let p = table_border_placement(
        rect(1, 1, 10, 10),
        rect(0, 0, u32::MAX, u32::MAX),
        TableBorderDrag::Move { x: -1, y: -1 },
        Some(1),
    )
    .unwrap();
    assert_eq!(p.bounds, rect(0, 0, 10, 10));
}
#[test]
fn uniform_margin_fitting_translates_only_outside_tables_and_intersects_content_without_scaling() {
    let old = SheetTemplate::StandardV1
        .layout(&SheetPaper::default())
        .unwrap();
    let mut paper = old.paper().clone();
    paper.left = 20000;
    paper.right = 20000;
    paper.top = 20000;
    paper.bottom = 20000;
    assert!(TemplateLayout::new(paper.clone(), old.content(), old.tables().to_vec()).is_err());
    let fitted =
        TemplateLayout::fit_inside_margins(paper, old.content(), old.tables().to_vec()).unwrap();
    assert_eq!(
        fitted.tables()[0].bounds(),
        rect(97000, 150000, 180000, 40000)
    );
    assert_eq!(fitted.tables()[0].cells(), old.tables()[0].cells());
    assert_eq!(fitted.content(), rect(20000, 20000, 257000, 135000));
    let again = TemplateLayout::fit_inside_margins(
        fitted.paper().clone(),
        fitted.content(),
        fitted.tables().to_vec(),
    )
    .unwrap();
    assert_eq!(again, fitted);
    assert_eq!(
        old.tables()[0].bounds(),
        rect(107000, 160000, 180000, 40000)
    );
}
#[test]
fn impossible_margin_fits_and_overflow_fail_while_empty_content_remains_explicit() {
    let old = SheetTemplate::StandardV1
        .layout(&SheetPaper::default())
        .unwrap();
    let mut paper = old.paper().clone();
    paper.left = 60000;
    paper.right = 60000;
    assert!(
        TemplateLayout::fit_inside_margins(paper, old.content(), old.tables().to_vec()).is_err()
    );
    assert!(
        TemplateLayout::fit_inside_margins(old.paper().clone(), rect(u32::MAX, 0, 1, 1), vec![])
            .is_err()
    );
    let empty =
        TemplateLayout::fit_inside_margins(old.paper().clone(), rect(0, 0, 0, 0), vec![]).unwrap();
    assert_eq!(empty.content(), rect(10000, 10000, 0, 0));
}
