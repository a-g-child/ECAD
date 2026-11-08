//! Logical document identity/order/membership are independent of electrical and drawing content.
use ecad_domain::*;
fn doc(id: u128, number: &str) -> ProjectDrawingDocument {
    ProjectDrawingDocument {
        id: DrawingDocumentId::new(id).unwrap(),
        number: number.into(),
        title: format!("Package {number}\nTitle"),
        revision: "A".into(),
    }
}
fn sheet(id: u128) -> DrawingSheet {
    DrawingSheet {
        id: SheetId::new(id).unwrap(),
        number: format!("D-{id}"),
        title: "Drawing".into(),
        paper: SheetPaper::default(),
        placement: None,
    }
}
#[test]
fn initial_package_identity_is_deterministic_and_metadata_changes_validate_atomically() {
    let mut p = Project::new(ProjectId::new(123).unwrap());
    let initial = p.drawing_documents()[0].clone();
    assert_eq!(initial, ProjectDrawingDocument::initial(p.id()));
    assert_eq!(initial.id.to_string(), p.id().to_string());
    assert!(p.remove_drawing_document(initial.id).is_err());
    p.insert_drawing_document(doc(2, "2"), 1).unwrap();
    let before = p.clone();
    assert!(p.insert_drawing_document(doc(2, "3"), 2).is_err());
    assert!(p.insert_drawing_document(doc(3, "1"), 2).is_err());
    assert!(
        p.replace_drawing_document(ProjectDrawingDocument {
            title: " \n ".into(),
            ..doc(2, "2")
        })
        .is_err()
    );
    assert_eq!(p, before);
    let old = p
        .replace_drawing_document(ProjectDrawingDocument {
            number: "POWER".into(),
            revision: "B\nDraft".into(),
            ..doc(2, "2")
        })
        .unwrap();
    assert_eq!(old, doc(2, "2"));
    p.validate().unwrap();
}
#[test]
fn sheets_move_between_owners_and_documents_without_renumbering_or_implicit_global_reparenting() {
    let mut p = Project::new(ProjectId::new(1).unwrap());
    let a = p.drawing_documents()[0].id;
    let b = DrawingDocumentId::new(2).unwrap();
    let c = DrawingDocumentId::new(3).unwrap();
    p.insert_drawing_document(doc(2, "2"), 1).unwrap();
    p.insert_drawing_document(doc(3, "3"), 2).unwrap();
    p.insert_sheet_in_document(sheet(1), a, 0).unwrap();
    p.insert_sheet_in_document(sheet(2), a, 1).unwrap();
    p.insert_sheet_in_document(sheet(3), b, 0).unwrap();
    p.insert_sheet_in_document(sheet(4), c, 0).unwrap();
    let source = p
        .sheets()
        .iter()
        .map(|s| (s.id, s.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        p.move_sheet_to_document(SheetId::new(2).unwrap(), b, 0)
            .unwrap(),
        (a, 1)
    );
    assert_eq!(
        p.sheets_in_document(b)
            .unwrap()
            .iter()
            .map(|s| s.number.as_str())
            .collect::<Vec<_>>(),
        vec!["D-2", "D-3"]
    );
    assert!(p.remove_drawing_document(b).is_err());
    p.move_drawing_document(c, 0).unwrap();
    assert_eq!(p.sheets()[0].number, "D-4");
    let before = p.clone();
    assert!(p.move_sheet(SheetId::new(3).unwrap(), 0).is_err());
    assert_eq!(p, before);
    for s in p.sheets() {
        assert_eq!(Some(s), source.get(&s.id));
    }
    p.validate().unwrap();
}
#[test]
fn complete_layout_rejects_missing_duplicate_or_unknown_membership_without_partial_changes() {
    let mut p = Project::new(ProjectId::new(1).unwrap());
    p.insert_sheet(sheet(1), 0).unwrap();
    p.insert_sheet(sheet(2), 1).unwrap();
    let before = p.clone();
    let good = p.drawing_document_layout();
    let mut duplicate = good.clone();
    duplicate[0].sheets.push(SheetId::new(1).unwrap());
    let mut missing = good.clone();
    missing[0].sheets.pop();
    let mut unknown = good.clone();
    unknown[0].sheets.push(SheetId::new(999).unwrap());
    let mut same_doc = good.clone();
    same_doc.push(good[0].clone());
    let mut same_number = good.clone();
    same_number.push(DrawingDocumentLayout {
        document: doc(2, "1"),
        sheets: vec![],
    });
    for bad in [vec![], duplicate, missing, unknown, same_doc, same_number] {
        assert!(p.replace_drawing_document_layout(bad).is_err());
        assert_eq!(p, before);
    }
    let layout = vec![
        DrawingDocumentLayout {
            document: doc(2, "2"),
            sheets: vec![SheetId::new(2).unwrap()],
        },
        DrawingDocumentLayout {
            document: good[0].document.clone(),
            sheets: vec![SheetId::new(1).unwrap()],
        },
    ];
    let old = p.replace_drawing_document_layout(layout.clone()).unwrap();
    assert_eq!(old, good);
    assert_eq!(p.drawing_document_layout(), layout);
    p.replace_drawing_document_layout(old).unwrap();
    assert_eq!(p, before);
}
