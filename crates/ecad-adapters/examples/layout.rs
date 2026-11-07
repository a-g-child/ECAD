//! Numeric origin layout with simultaneous connected-route movement and exact history.

use ecad_adapters::{JsonFile, decode_project};
use ecad_app::{Document, LayoutAxis, LayoutOperation, layout_plan};
use ecad_domain::{AnchorId, DrawingPoint, ObjectRef};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let before = decode_project(include_str!("../tests/fixtures/routed-conveyor-v6.json"))?;
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("layout.json"));
    let mut document = Document::new(before.clone(), store.clone());
    document.save()?;
    let selection = [1, 2, 3].map(|id| AnchorId::new(id).map(ObjectRef::Anchor));
    let selection = selection.into_iter().collect::<Result<Vec<_>, _>>()?;
    for operation in [
        LayoutOperation::Translate(DrawingPoint { x: 10_000, y: 0 }),
        LayoutOperation::Align {
            axis: LayoutAxis::Y,
            coordinate_um: 1_250,
        },
        LayoutOperation::Distribute(LayoutAxis::X),
    ] {
        let plan = layout_plan(
            document.session().project(),
            selection.iter().copied(),
            operation,
        )?;
        println!(
            "{} origins change; {} connected routes",
            plan.changes.len(),
            plan.affected_routes.len()
        );
        document.execute_batch(document.session().revision(), plan.commands)?;
    }
    let after = document.session().project().clone();
    if before.conductors().collect::<Vec<_>>() != after.conductors().collect::<Vec<_>>() {
        return Err("layout changed electrical connectivity".into());
    }
    document.save()?;
    if Document::open(store)?.session().project() != &after {
        return Err("layout changed on reopen".into());
    }
    // Distribution is already even, so only translation and alignment enter history.
    for _ in 0..2 {
        document.undo(document.session().revision())?;
    }
    if document.session().project() != &before {
        return Err("group layout undo did not restore exact geometry".into());
    }
    for _ in 0..2 {
        document.redo(document.session().revision())?;
    }
    if document.session().project() != &after || document.session().is_dirty() {
        return Err("layout redo did not restore the saved checkpoint".into());
    }
    println!(
        "Origin layout, unchanged connectivity, save/reopen, and exact grouped undo/redo passed."
    );
    Ok(())
}
