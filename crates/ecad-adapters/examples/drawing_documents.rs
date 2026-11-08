//! Logical packages group sheets without changing shared source geometry or electrical identity.
use ecad_adapters::{JsonFile, RandomIds, decode_project};
use ecad_app::{Command, Document, DrawingDocumentCommand};
use ecad_domain::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let file = JsonFile::new(directory.path().join("documents.json"));
    let mut p = decode_project(include_str!("../tests/fixtures/routed-conveyor-v8.json"))?;
    let initial = p.drawing_documents()[0].id;
    for id in 1..=2 {
        p.insert_sheet_in_document(
            DrawingSheet {
                id: SheetId::new(id)?,
                number: format!("D-{id}"),
                title: format!("Harness detail {id}"),
                paper: SheetPaper::default(),
                placement: Some(SheetViewPlacement {
                    view: ViewId::new(1)?,
                    paper_origin: DrawingPoint {
                        x: 20_000,
                        y: 60_000,
                    },
                    view_origin: DrawingPoint::default(),
                    numerator: 1,
                    denominator: 1,
                }),
            },
            initial,
            (id - 1) as usize,
        )?;
    }
    let before = p.clone();
    let mut d = Document::new(p, file.clone());
    let id = RandomIds::drawing_document()?;
    d.execute(
        d.session().revision(),
        Command::DrawingDocument(DrawingDocumentCommand::Insert {
            document: ProjectDrawingDocument {
                id,
                number: "HARNESS".into(),
                title: "Harness drawings".into(),
                revision: "A".into(),
                default_template: ecad_domain::SheetTemplate::StandardV1,
            },
            index: 1,
        }),
    )?;
    d.execute(
        d.session().revision(),
        Command::DrawingDocument(DrawingDocumentCommand::MoveSheet {
            sheet: SheetId::new(1)?,
            document: id,
            index: 0,
        }),
    )?;
    d.save()?;
    assert_eq!(
        Document::open(file)?.session().project(),
        d.session().project()
    );
    assert_eq!(d.session().project().drawing(), before.drawing());
    d.undo(d.session().revision())?;
    d.undo(d.session().revision())?;
    assert_eq!(d.session().project(), &before);
    println!(
        "Two drawing documents: explicit sheet ownership, preserved shared views, exact undo and v10 round trip passed."
    );
    Ok(())
}
