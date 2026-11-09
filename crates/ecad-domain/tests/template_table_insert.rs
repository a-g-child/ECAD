//! Band insertion at outside/inside boundaries preserves spans, typed contents and exact extents.
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
fn base() -> TemplateTable {
    TemplateTable::new(
        TemplateTableId::new(1).unwrap(),
        rect(100, 100, 100, 100),
        vec![
            TemplateCell {
                picture: None,
                text_styles: Default::default(),
                id: id(1),
                bounds: rect(0, 0, 50, 100),
                caption: "Spanning\ncompany".into(),
                contents: vec![TemplateTextRun::Field(
                    FrameField::from_key("company.name").unwrap(),
                )],
            },
            TemplateCell {
                picture: None,
                text_styles: Default::default(),
                id: id(2),
                bounds: rect(50, 0, 50, 50),
                caption: "Upper".into(),
                contents: vec![],
            },
            TemplateCell {
                picture: None,
                text_styles: Default::default(),
                id: id(3),
                bounds: rect(50, 50, 50, 50),
                caption: "Lower".into(),
                contents: vec![],
            },
        ],
    )
    .unwrap()
}
#[test]
fn interior_insertion_extends_spans_and_allocates_only_uncovered_blank_cells() {
    let t = base();
    assert_eq!(t.insertion_cell_count(TableAxis::Y, 50).unwrap(), 1);
    let added = t
        .insert_band(TableAxis::Y, 50, 10, TableGrowthAnchor::End, &[id(4)])
        .unwrap();
    assert_eq!(added.bounds(), rect(100, 90, 100, 110));
    assert_eq!(added.cells()[0].bounds, rect(0, 0, 50, 110));
    assert_eq!(added.cells()[0].contents, t.cells()[0].contents);
    assert_eq!(added.cells()[1], t.cells()[1]);
    assert_eq!(added.cells()[2].bounds, rect(50, 60, 50, 50));
    assert_eq!(added.cells()[3].bounds, rect(50, 50, 50, 10));
    assert!(added.cells()[3].caption.is_empty() && added.cells()[3].contents.is_empty());
    let column = t
        .insert_band(
            TableAxis::X,
            50,
            10,
            TableGrowthAnchor::Start,
            &[id(4), id(5)],
        )
        .unwrap();
    assert_eq!(column.bounds(), rect(100, 100, 110, 100));
    assert_eq!(column.cells()[3].bounds, rect(50, 0, 10, 50));
    assert_eq!(column.cells()[4].bounds, rect(50, 50, 10, 50));
    assert_eq!(t, base());
}
#[test]
fn outer_insertions_copy_adjacent_divisions_and_keep_the_chosen_edge_fixed() {
    let t = base();
    for at in [0, 100] {
        let row = t
            .insert_band(
                TableAxis::Y,
                at,
                10,
                TableGrowthAnchor::End,
                &[id(4), id(5)],
            )
            .unwrap();
        assert_eq!(row.bounds().y + row.bounds().height, 200);
        assert_eq!(row.cells()[3].bounds, rect(0, at, 50, 10));
        assert_eq!(row.cells()[4].bounds, rect(50, at, 50, 10));
    }
    assert_eq!(t.insertion_cell_count(TableAxis::X, 0).unwrap(), 1);
    assert_eq!(t.insertion_cell_count(TableAxis::X, 100).unwrap(), 2);
}
#[test]
fn invalid_boundaries_anchor_space_overflow_and_identity_reuse_are_atomic_errors() {
    let t = base();
    for (at, size, ids) in [
        (25, 10, vec![id(4)]),
        (101, 10, vec![id(4)]),
        (50, 0, vec![id(4)]),
        (50, 101, vec![id(4)]),
        (100, 10, vec![id(4), id(4)]),
        (50, 10, vec![id(1)]),
        (50, 10, vec![]),
        (50, u32::MAX, vec![id(4)]),
    ] {
        assert!(
            t.insert_band(TableAxis::Y, at, size, TableGrowthAnchor::End, &ids)
                .is_err()
        );
    }
    assert_eq!(t, base());
}
