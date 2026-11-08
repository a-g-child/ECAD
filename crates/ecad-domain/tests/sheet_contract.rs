//! Sheet paper, identity and view references are independent of schematic geometry.
use ecad_domain::*;
fn sheet(id: u128, number: &str) -> DrawingSheet {
    DrawingSheet {
        inherited_variant: None,
        kind: ecad_domain::SheetKind::Schematic,
        template: None,
        id: SheetId::new(id).unwrap(),
        number: number.into(),
        title: "Drawing\nTitle".into(),
        paper: SheetPaper::default(),
        placement: Some(SheetViewPlacement {
            view: ViewId::new(1).unwrap(),
            paper_origin: DrawingPoint { x: -10, y: 20 },
            view_origin: DrawingPoint {
                x: i64::MIN,
                y: i64::MAX,
            },
            numerator: 1,
            denominator: 2,
        }),
    }
}
#[test]
fn paper_margins_and_scale_validate_exactly_without_sum_overflow() {
    let valid = sheet(1, "1");
    valid.validate().unwrap();
    for s in [
        DrawingSheet {
            paper: SheetPaper {
                width: 0,
                ..valid.paper.clone()
            },
            ..valid.clone()
        },
        DrawingSheet {
            paper: SheetPaper {
                left: u32::MAX,
                right: u32::MAX,
                ..valid.paper.clone()
            },
            ..valid.clone()
        },
        DrawingSheet {
            paper: SheetPaper {
                top: 210000,
                bottom: 0,
                ..valid.paper.clone()
            },
            ..valid.clone()
        },
        DrawingSheet {
            title: " \n ".into(),
            ..valid.clone()
        },
        DrawingSheet {
            placement: Some(SheetViewPlacement {
                numerator: 0,
                ..valid.placement.clone().unwrap()
            }),
            ..valid.clone()
        },
        DrawingSheet {
            placement: Some(SheetViewPlacement {
                denominator: 0,
                ..valid.placement.clone().unwrap()
            }),
            ..valid.clone()
        },
    ] {
        assert!(s.validate().is_err());
    }
    DrawingSheet {
        paper: SheetPaper {
            width: 1,
            height: 1,
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        },
        placement: None,
        ..valid
    }
    .validate()
    .unwrap();
}
#[test]
fn sheet_identity_number_order_and_view_lifetime_are_independent_and_fail_atomically() {
    let mut project = Project::new(ProjectId::new(1).unwrap());
    assert!(project.insert_sheet(sheet(1, "A"), 0).is_err());
    project
        .insert_drawing_view(DrawingView {
            id: ViewId::new(1).unwrap(),
            name: "Empty view".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    project.insert_sheet(sheet(1, "A"), 0).unwrap();
    project.insert_sheet(sheet(2, "B"), 1).unwrap();
    let before = project.clone();
    assert!(project.insert_sheet(sheet(1, "C"), 2).is_err());
    assert!(project.insert_sheet(sheet(3, "A"), 2).is_err());
    assert!(project.replace_sheet(sheet(2, "A")).is_err());
    assert_eq!(project, before);
    assert!(matches!(
        project.remove_drawing_view(ViewId::new(1).unwrap()),
        Err(DrawingError::ViewInUse(_))
    ));
    assert!(project.replace_drawing(DrawingDocument::default()).is_err());
    assert_eq!(project, before);
    project.move_sheet(SheetId::new(2).unwrap(), 0).unwrap();
    assert_eq!(
        project
            .sheets()
            .iter()
            .map(|s| s.number.as_str())
            .collect::<Vec<_>>(),
        vec!["B", "A"]
    );
    assert!(project.move_sheet(SheetId::new(2).unwrap(), 2).is_err());
    project.remove_sheet(SheetId::new(1).unwrap()).unwrap();
    project.remove_sheet(SheetId::new(2).unwrap()).unwrap();
    assert!(project.drawing().view(ViewId::new(1).unwrap()).is_some());
    project
        .remove_drawing_view(ViewId::new(1).unwrap())
        .unwrap();
}
