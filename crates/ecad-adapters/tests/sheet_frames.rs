//! Bundled frames resolve portable values without changing drawings or lifecycle/history.
use ecad_adapters::{JsonFile, decode_project, encode_project};
use ecad_app::*;
use ecad_domain::*;
use serde_json::{Value, json};
fn setup() -> (Project, SheetId) {
    let mut p = decode_project(include_str!("fixtures/routed-conveyor-v9.json")).unwrap();
    let id = SheetId::new(1).unwrap();
    p.insert_sheet(
        DrawingSheet {
            kind: ecad_domain::SheetKind::Schematic,
            template: None,
            id,
            number: "E-01".into(),
            title: "Harness\nDetail".into(),
            paper: SheetPaper::default(),
            placement: None,
        },
        0,
    )
    .unwrap();
    (p, id)
}
fn value(frame: &SheetFrame, field: FrameField) -> Option<&str> {
    frame
        .cells
        .iter()
        .find(|c| c.field == field)
        .unwrap()
        .value
        .as_deref()
}
#[test]
fn frame_values_follow_metadata_owner_and_history_without_copying_source_geometry() {
    let (p, id) = setup();
    let directory = tempfile::tempdir().unwrap();
    let mut d = Document::new(p, JsonFile::new(directory.path().join("frame.json")));
    d.save().unwrap();
    let before = d.session().project().clone();
    let frame = before.sheet_frame(id).unwrap();
    assert_eq!(frame.template, SheetTemplate::StandardV1);
    assert_eq!(
        value(&frame, FrameField::SheetTitle),
        Some("Harness\nDetail")
    );
    assert_eq!(value(&frame, FrameField::DocumentSheetCount), Some("1 / 1"));
    assert_eq!(
        value(&frame, FrameField::Metadata(MetadataField::CompanyName)),
        None
    );
    let mut owner = before.drawing_documents()[0].clone();
    owner.title = "Wiring\nPackage".into();
    owner.revision = "R2".into();
    d.execute(
        d.session().revision(),
        Command::DrawingDocument(DrawingDocumentCommand::Replace(owner)),
    )
    .unwrap();
    d.execute(
        d.session().revision(),
        Command::ProjectMetadata(ProjectMetadata::new(
            [
                (
                    MetadataField::CompanyName,
                    "Company <literal>\nAddress".into(),
                ),
                (MetadataField::ProjectTitle, "Car".into()),
            ]
            .into(),
        )),
    )
    .unwrap();
    let after = d.session().project().sheet_frame(id).unwrap();
    assert_eq!(
        value(&after, FrameField::Metadata(MetadataField::CompanyName)),
        Some("Company <literal>\nAddress")
    );
    assert_eq!(
        value(&after, FrameField::DocumentTitle),
        Some("Wiring\nPackage")
    );
    assert_eq!(value(&after, FrameField::DocumentRevision), Some("R2"));
    assert_eq!(d.session().project().drawing(), before.drawing());
    d.save().unwrap();
    let reopened = Document::open(JsonFile::new(directory.path().join("frame.json"))).unwrap();
    assert_eq!(reopened.session().project().sheet_frame(id).unwrap(), after);
    d.undo(d.session().revision()).unwrap();
    d.undo(d.session().revision()).unwrap();
    assert_eq!(d.session().project(), &before);
    assert_eq!(d.session().project().sheet_frame(id).unwrap(), frame);
    d.redo(d.session().revision()).unwrap();
    d.redo(d.session().revision()).unwrap();
    assert_eq!(d.session().project().sheet_frame(id).unwrap(), after);
    assert!(!d.session().is_dirty());
}
#[test]
fn title_block_stays_in_paper_units_and_tiny_or_extreme_paper_is_safe() {
    let (mut p, id) = setup();
    let frame = p.sheet_frame(id).unwrap();
    assert_eq!(
        frame.border,
        FrameRect {
            x: 10_000,
            y: 10_000,
            width: 277_000,
            height: 190_000
        }
    );
    assert_eq!(frame.cells.len(), 8);
    let mut sheet = p.sheet(id).unwrap().clone();
    sheet.placement = Some(SheetViewPlacement {
        view: p.drawing().views().next().unwrap().id,
        paper_origin: DrawingPoint {
            x: i64::MAX,
            y: i64::MIN,
        },
        view_origin: DrawingPoint {
            x: i64::MIN,
            y: i64::MAX,
        },
        numerator: u32::MAX,
        denominator: 1,
    });
    p.replace_sheet(sheet.clone()).unwrap();
    assert_eq!(p.sheet_frame(id).unwrap(), frame);
    for (width, height, fits) in [
        (200_000, 60_000, true),
        (199_999, 60_000, false),
        (u32::MAX, u32::MAX, true),
        (20_001, 20_001, false),
    ] {
        sheet.paper.width = width;
        sheet.paper.height = height;
        p.replace_sheet(sheet.clone()).unwrap();
        let f = p.sheet_frame(id).unwrap();
        assert_eq!(f.title_block_too_small, !fits);
        assert_eq!(f.cells.is_empty(), !fits);
        for cell in f.cells {
            assert!(cell.bounds.x >= f.border.x && cell.bounds.y >= f.border.y);
            assert!(
                u64::from(cell.bounds.x) + u64::from(cell.bounds.width)
                    <= u64::from(width - sheet.paper.right)
            );
            assert!(
                u64::from(cell.bounds.y) + u64::from(cell.bounds.height)
                    <= u64::from(height - sheet.paper.bottom)
            );
        }
    }
    assert!(p.sheet_frame(SheetId::new(999).unwrap()).is_err());
}
#[test]
fn v9_upgrade_and_current_template_validation_are_strict_and_deterministic() {
    let old = include_str!("fixtures/routed-conveyor-v9.json");
    let a = decode_project(old).unwrap();
    let b = decode_project(old).unwrap();
    assert_eq!(a, b);
    let current = encode_project(&a).unwrap();
    assert_eq!(current, include_str!("fixtures/routed-conveyor-v12.json"));
    assert_eq!(decode_project(&current).unwrap(), a);
    let data: Value = serde_json::from_str(&current).unwrap();
    for bad in [
        json!(null),
        json!("standard-v2"),
        json!({"key":"standard-v1"}),
    ] {
        let mut d = data.clone();
        d["drawing_documents"][0]["default_template"] = bad;
        assert!(decode_project(&d.to_string()).is_err());
    }
    let mut d = data;
    d["drawing_documents"][0]
        .as_object_mut()
        .unwrap()
        .remove("default_template");
    assert!(decode_project(&d.to_string()).is_err());
    let mut legacy: Value = serde_json::from_str(old).unwrap();
    legacy["drawing_documents"][0]["default_template"] = json!("standard-v1");
    assert!(decode_project(&legacy.to_string()).is_err());
}
#[test]
fn closed_sheet_preview_resolves_candidate_title_and_count_without_mutation() {
    let (p, id) = setup();
    let session = Session::new(p);
    let mut sheet = session.project().sheet(id).unwrap().clone();
    sheet.id = SheetId::new(2).unwrap();
    sheet.number = "E-02".into();
    sheet.title = "Proposed".into();
    let preview = preview_sheet(&session, sheet, true).unwrap();
    assert_eq!(
        value(preview.frame(), FrameField::SheetTitle),
        Some("Proposed")
    );
    assert_eq!(
        value(preview.frame(), FrameField::DocumentSheetCount),
        Some("2 / 2")
    );
    assert_eq!(
        value(
            &session.project().sheet_frame(id).unwrap(),
            FrameField::DocumentSheetCount
        ),
        Some("1 / 1")
    );
    assert_eq!(session.project().sheets().len(), 1);
}

#[test]
fn counts_are_owner_local_and_defaults_change_appearance_without_changing_frame_geometry() {
    let (mut p, a) = setup();
    let mut sheet = p.sheet(a).unwrap().clone();
    let b = SheetId::new(2).unwrap();
    sheet.id = b;
    sheet.number = "E-02".into();
    p.insert_sheet(sheet, 1).unwrap();
    assert_eq!(
        value(&p.sheet_frame(a).unwrap(), FrameField::DocumentSheetCount),
        Some("1 / 2")
    );
    assert_eq!(
        value(&p.sheet_frame(b).unwrap(), FrameField::DocumentSheetCount),
        Some("2 / 2")
    );
    let mut owner = p.drawing_documents()[0].clone();
    owner.id = DrawingDocumentId::new(2).unwrap();
    owner.number = "DETAIL".into();
    owner.revision = "Z".into();
    let target = owner.id;
    p.insert_drawing_document(owner, 1).unwrap();
    let before = p.clone();
    let mut session = Session::new(p);
    session
        .execute(
            session.revision(),
            Command::DrawingDocument(DrawingDocumentCommand::MoveSheet {
                sheet: b,
                document: target,
                index: 0,
            }),
        )
        .unwrap();
    let p = session.project();
    assert_eq!(
        value(&p.sheet_frame(a).unwrap(), FrameField::DocumentSheetCount),
        Some("1 / 1")
    );
    assert_eq!(
        value(&p.sheet_frame(b).unwrap(), FrameField::DocumentSheetCount),
        Some("1 / 1")
    );
    assert_eq!(
        value(&p.sheet_frame(b).unwrap(), FrameField::DocumentNumber),
        Some("DETAIL")
    );
    assert_eq!(
        value(&p.sheet_frame(b).unwrap(), FrameField::DocumentRevision),
        Some("Z")
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
    let mut p = before;
    let frame = p.sheet_frame(a).unwrap();
    let mut settings = p.drawing().settings().clone();
    settings.base.font = FontFamily::new("monospace").unwrap();
    settings.base.text_height = DrawingLength::new(4000).unwrap();
    settings.base.stroke_width = DrawingLength::new(500).unwrap();
    settings.base.colour = DrawingColour {
        red: 20,
        green: 30,
        blue: 40,
    };
    p.set_drawing_settings(settings.clone()).unwrap();
    let updated = p.sheet_frame(a).unwrap();
    assert_eq!(updated.appearance, settings.base);
    assert_eq!(updated.border, frame.border);
    assert_eq!(updated.cells, frame.cells);
}
