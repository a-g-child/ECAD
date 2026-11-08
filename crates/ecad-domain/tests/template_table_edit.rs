//! Structured edits preserve exact partitions, identities and unresolved bindings.
use ecad_domain::*;
fn id(n: u128) -> TemplateCellId {
    TemplateCellId::new(n).unwrap()
}
fn rect(x: u32, y: u32, width: u32, height: u32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
fn table(width: u32, height: u32) -> TemplateTable {
    TemplateTable::new(
        TemplateTableId::new(1).unwrap(),
        rect(0, 0, width, height),
        vec![TemplateCell {
            id: id(1),
            bounds: rect(0, 0, width, height),
            caption: "Company\nOffice".into(),
            contents: vec![TemplateTextRun::Field(
                FrameField::from_key("company.name").unwrap(),
            )],
        }],
    )
    .unwrap()
}
#[test]
fn grid_splits_retain_original_content_and_exact_remainders_then_support_unequal_rows() {
    for rows in 1..=5 {
        for columns in 1..=5 {
            if columns * rows == 1 {
                continue;
            }
            let t = table(101, 103);
            let fresh: Vec<_> = (2..=u128::from(columns * rows)).map(id).collect();
            let grid = t.split_cell(id(1), columns, rows, &fresh).unwrap();
            assert_eq!(grid.cells().len(), (columns * rows) as usize);
            assert_eq!(grid.cells()[0].caption, t.cells()[0].caption);
            assert_eq!(grid.cells()[0].contents, t.cells()[0].contents);
            assert!(
                grid.cells()[1..]
                    .iter()
                    .all(|c| c.caption.is_empty() && c.contents.is_empty())
            );
            let merged = grid
                .merge_cells(
                    &grid.cells().iter().map(|c| c.id).collect::<Vec<_>>(),
                    id(1),
                    MergeContents::Combine,
                )
                .unwrap();
            assert_eq!(merged, t);
        }
    }
    let rows = table(101, 103).split_cell(id(1), 1, 2, &[id(2)]).unwrap();
    let unequal = rows.split_cell(id(1), 3, 1, &[id(3), id(4)]).unwrap();
    assert_eq!(unequal.cells()[3].bounds, rect(0, 51, 101, 52));
    let resized = unequal.resized(rect(20, 30, 203, 207)).unwrap();
    assert_eq!(resized.cells()[3].bounds, rect(0, 102, 203, 105));
    assert_eq!(resized.cells()[0].contents, unequal.cells()[0].contents);
    assert_eq!(unequal.bounds(), rect(0, 0, 101, 103));
}
#[test]
fn merges_review_content_policy_in_reading_order_and_reject_nonrectangles_and_foreign_ids() {
    let grid = table(100, 100)
        .split_cell(id(1), 2, 2, &[id(2), id(3), id(4)])
        .unwrap();
    let mut cells = grid.cells().to_vec();
    cells[1].caption = "Project".into();
    cells[1].contents = vec![TemplateTextRun::Literal("Literal\nvalue".into())];
    cells.reverse(); // Presentation order does not control merge content order.
    let t = TemplateTable::new(grid.id(), grid.bounds(), cells).unwrap();
    let merged = t
        .merge_cells(&[id(2), id(1)], id(2), MergeContents::Combine)
        .unwrap();
    let c = merged.cells().iter().find(|c| c.id == id(2)).unwrap();
    assert_eq!(c.caption, "Company\nOffice\nProject");
    assert_eq!(
        c.contents,
        vec![
            TemplateTextRun::Field(FrameField::from_key("company.name").unwrap()),
            TemplateTextRun::Literal("\n".into()),
            TemplateTextRun::Literal("Literal\nvalue".into())
        ]
    );
    assert_eq!(merged.cells()[0].id, id(4));
    let primary = t
        .merge_cells(&[id(1), id(2)], id(2), MergeContents::KeepPrimary)
        .unwrap();
    assert_eq!(
        primary
            .cells()
            .iter()
            .find(|c| c.id == id(2))
            .unwrap()
            .caption,
        "Project"
    );
    for selection in [
        vec![id(1), id(4)],
        vec![id(1), id(2), id(3)],
        vec![id(1), id(99)],
        vec![id(1), id(1)],
    ] {
        assert!(
            t.merge_cells(&selection, id(1), MergeContents::Combine)
                .is_err()
        );
    }
    assert!(
        t.merge_cells(&[id(1), id(2)], id(3), MergeContents::Combine)
            .is_err()
    );
}
#[test]
fn seam_moves_preserve_spans_and_reject_crossing_collapsing_outer_or_absent_edges() {
    let rows = table(100, 100).split_cell(id(1), 1, 2, &[id(2)]).unwrap();
    let t = rows.split_cell(id(1), 2, 1, &[id(3)]).unwrap();
    let moved = t.move_boundary(TableAxis::X, 50, 70).unwrap();
    assert_eq!(moved.cells()[0].bounds, rect(0, 0, 70, 50));
    assert_eq!(moved.cells()[1].bounds, rect(70, 0, 30, 50));
    assert_eq!(moved.cells()[2], t.cells()[2]);
    let moved = moved.move_boundary(TableAxis::Y, 50, 30).unwrap();
    assert_eq!(moved.cells()[2].bounds, rect(0, 30, 100, 70));
    for (from, to) in [
        (0, 20),
        (100, 20),
        (40, 20),
        (50, 0),
        (50, 100),
        (50, u32::MAX),
    ] {
        assert!(t.move_boundary(TableAxis::X, from, to).is_err());
    }
    assert_eq!(t.move_boundary(TableAxis::X, 50, 50).unwrap(), t);
    let grid = table(99, 99)
        .split_cell(id(1), 3, 1, &[id(2), id(3)])
        .unwrap();
    assert!(grid.move_boundary(TableAxis::X, 33, 67).is_err());
}
#[test]
fn extreme_geometry_and_bad_split_ids_fail_without_mutating_the_source() {
    let t = table(u32::MAX, u32::MAX);
    let split = t.split_cell(id(1), 2, 1, &[id(2)]).unwrap();
    assert_eq!(split.cells()[1].bounds.width, 2_147_483_648);
    assert!(split.resized(rect(u32::MAX, 0, 2, 10)).is_err());
    assert!(split.resized(rect(0, 0, 1, 1)).is_err());
    for (columns, rows, ids) in [
        (0, 2, vec![]),
        (1, 1, vec![]),
        (2, 1, vec![id(1)]),
        (2, 2, vec![id(2), id(2), id(3)]),
        (2, 1, vec![]),
    ] {
        assert!(t.split_cell(id(1), columns, rows, &ids).is_err());
    }
    assert!(t.split_cell(id(99), 2, 1, &[id(2)]).is_err());
    assert_eq!(t.cells()[0].bounds, rect(0, 0, u32::MAX, u32::MAX));
}
