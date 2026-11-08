//! Structured template invariants and exact preservation of legacy frame geometry.
use ecad_domain::*;
fn bounds(x: u32, y: u32, width: u32, height: u32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
fn cell(id: u128, b: FrameRect) -> TemplateCell {
    TemplateCell {
        id: TemplateCellId::new(id).unwrap(),
        bounds: b,
        caption: "Caption\nsecond line".into(),
        contents: vec![TemplateTextRun::Literal(" value\n\n".into())],
    }
}
fn table(cells: Vec<TemplateCell>) -> Result<TemplateTable, TemplateError> {
    TemplateTable::new(
        TemplateTableId::new(1).unwrap(),
        bounds(10_000, 10_000, 100, 100),
        cells,
    )
}
#[test]
fn unequal_row_divisions_and_rectangular_spans_are_real_partitions() {
    let cells = vec![
        cell(1, bounds(0, 0, 100, 40)),
        cell(2, bounds(0, 40, 25, 60)),
        cell(3, bounds(25, 40, 75, 60)),
    ];
    let t = table(cells.clone()).unwrap();
    assert_eq!(t.cells(), cells);
    let spanned = vec![
        cell(1, bounds(0, 0, 25, 100)),
        cell(2, bounds(25, 0, 75, 40)),
        cell(3, bounds(25, 40, 75, 60)),
    ];
    assert!(table(spanned).is_ok());
}
#[test]
fn partition_rejects_overlap_gap_duplicate_and_overflow_even_when_area_matches() {
    assert_eq!(table(vec![]), Err(TemplateError::Gap));
    assert_eq!(
        table(vec![cell(1, bounds(0, 0, 100, 99))]),
        Err(TemplateError::Gap)
    );
    assert_eq!(
        table(vec![
            cell(1, bounds(0, 0, 50, 100)),
            cell(2, bounds(49, 0, 50, 100))
        ]),
        Err(TemplateError::Overlap)
    );
    assert_eq!(
        table(vec![
            cell(1, bounds(0, 0, 50, 100)),
            cell(1, bounds(50, 0, 50, 100))
        ]),
        Err(TemplateError::DuplicateCell(
            TemplateCellId::new(1).unwrap()
        ))
    );
    for b in [
        bounds(0, 0, 0, 100),
        bounds(1, 0, 100, 100),
        bounds(u32::MAX, 0, u32::MAX, 100),
    ] {
        assert_eq!(table(vec![cell(1, b)]), Err(TemplateError::Bounds));
    }
    assert_eq!(
        TemplateTable::new(
            TemplateTableId::new(1).unwrap(),
            bounds(u32::MAX, 0, 2, 2),
            vec![]
        ),
        Err(TemplateError::Bounds)
    );
    // The largest representable extent uses widened area arithmetic.
    let b = bounds(0, 0, u32::MAX, u32::MAX);
    assert!(TemplateTable::new(TemplateTableId::new(1).unwrap(), b, vec![cell(1, b)]).is_ok());
}
#[test]
fn layout_validates_owner_containment_and_global_cell_identity() {
    let t = table(vec![cell(1, bounds(0, 0, 100, 100))]).unwrap();
    let paper = SheetPaper::default();
    let content = bounds(15_000, 15_000, 100_000, 100_000);
    let duplicate = TemplateTable::new(
        TemplateTableId::new(2).unwrap(),
        t.bounds(),
        t.cells().to_vec(),
    )
    .unwrap();
    assert!(matches!(
        TemplateLayout::new(paper.clone(), content, vec![t.clone(), duplicate]),
        Err(TemplateError::DuplicateCell(_))
    ));
    assert!(matches!(
        TemplateLayout::new(paper.clone(), content, vec![t.clone(), t.clone()]),
        Err(TemplateError::DuplicateTable(_))
    ));
    assert_eq!(
        TemplateLayout::new(paper.clone(), bounds(0, 0, 1, 1), vec![]),
        Err(TemplateError::Bounds)
    );
    let outside = TemplateTable::new(t.id(), bounds(0, 0, 100, 100), t.cells().to_vec()).unwrap();
    assert_eq!(
        TemplateLayout::new(paper, content, vec![outside]),
        Err(TemplateError::Bounds)
    );
}
#[test]
fn typed_multiline_runs_use_explicit_context_and_preserve_missing_fields() {
    let mut c = cell(1, bounds(0, 0, 100, 100));
    c.contents = vec![
        TemplateTextRun::Literal("Project: ".into()),
        TemplateTextRun::Field(FrameField::SheetTitle),
        TemplateTextRun::Literal("\nCompany: ".into()),
        TemplateTextRun::Field(FrameField::Metadata(MetadataField::CompanyName)),
        TemplateTextRun::Field(FrameField::Metadata(MetadataField::CompanyName)),
    ];
    let result =
        c.resolve(|key| (key == FrameField::SheetTitle).then(|| "Line 1\n\n Line 3".into()));
    assert_eq!(result.value, "Project: Line 1\n\n Line 3\nCompany: ");
    assert_eq!(
        result.missing,
        [FrameField::Metadata(MetadataField::CompanyName)]
    );
    assert_eq!(c.resolve(|_| Some(String::new())).missing, []);
    for field in MetadataField::ALL {
        assert_eq!(
            FrameField::from_key(field.key()),
            Some(FrameField::Metadata(field))
        );
    }
    assert_eq!(FrameField::from_key("future.field"), None);
}
#[test]
fn legacy_frames_keep_exact_block_positions_cells_and_content_regions() {
    let paper = SheetPaper {
        left: 7_000,
        right: 11_000,
        top: 13_000,
        bottom: 17_000,
        ..SheetPaper::default()
    };
    for (template, block, content_height, cell_width, cell_height) in [
        (
            SheetTemplate::StandardV1,
            bounds(106_000, 153_000, 180_000, 40_000),
            130_000,
            90_000,
            10_000,
        ),
        (
            SheetTemplate::CoverV1,
            bounds(56_500, 63_000, 180_000, 80_000),
            40_000,
            90_000,
            20_000,
        ),
        (
            SheetTemplate::DrawingListV1,
            bounds(106_000, 173_000, 180_000, 20_000),
            150_000,
            45_000,
            10_000,
        ),
    ] {
        let l = template.layout(&paper).unwrap();
        assert_eq!(l.border(), bounds(7_000, 13_000, 279_000, 180_000));
        assert_eq!(l.content(), bounds(12_000, 18_000, 269_000, content_height));
        assert_eq!(l.tables()[0].bounds(), block);
        assert_eq!(l.tables()[0].cells().len(), 8);
        let first = &l.tables()[0].cells()[0];
        assert_eq!(first.bounds, bounds(0, 0, cell_width, cell_height));
        assert_eq!(
            first.contents,
            [TemplateTextRun::Field(FrameField::Metadata(
                MetadataField::CompanyName
            ))]
        );
        assert_eq!(template.layout(&paper).unwrap(), l);
    }
}
#[test]
fn tiny_and_extreme_valid_legacy_paper_do_not_panic_or_normalise() {
    for paper in [
        SheetPaper {
            width: 1,
            height: 1,
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        },
        SheetPaper {
            width: u32::MAX,
            height: u32::MAX,
            left: u32::MAX - 1,
            right: 0,
            top: u32::MAX - 1,
            bottom: 0,
        },
    ] {
        for template in SheetTemplate::ALL {
            let l = template.layout(&paper).unwrap();
            assert_eq!(l.paper(), &paper);
            assert!(l.tables().is_empty());
            assert_eq!(l.content().width, 0);
            assert_eq!(l.content().height, 0);
        }
    }
    assert!(
        SheetTemplate::StandardV1
            .layout(&SheetPaper {
                width: 0,
                ..SheetPaper::default()
            })
            .is_err()
    );
}
