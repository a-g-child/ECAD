//! Retain whole routes, drag a segment, move a shared anchor, and undo exact geometry.

use ecad_adapters::{JsonFile, decode_project};
use ecad_app::{Command, Document, RouteCommand};
use ecad_domain::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project = decode_project(include_str!("../tests/fixtures/routed-conveyor-v6.json"))?;
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("routes.json"));
    let mut document = Document::new(project, store.clone());
    document.save()?;
    let before = document.session().project().clone();
    document.execute(
        document.session().revision(),
        Command::Route(RouteCommand::Edit {
            route: RouteId::new(1)?,
            edit: RouteEdit::MoveSegment {
                index: 0,
                coordinate_um: 4_000,
            },
        }),
    )?;
    let middle = document.session().project().clone();
    document.execute(
        document.session().revision(),
        Command::Route(RouteCommand::MoveAnchor {
            anchor: AnchorId::new(2)?,
            position: DrawingPoint {
                x: 10_000,
                y: 5_000,
            },
        }),
    )?;
    let after = document.session().project().clone();
    if after.conductors().collect::<Vec<_>>() != before.conductors().collect::<Vec<_>>() {
        return Err("geometry editing changed electrical connectivity".into());
    }
    if document
        .execute(
            document.session().revision(),
            Command::SetTermination {
                conductor: ConductorId::new(1)?,
                end: End::B,
                terminal: None,
            },
        )
        .is_ok()
    {
        return Err("electrical change silently invalidated a drawn route".into());
    }
    document.save()?;
    if Document::open(store)?.session().project() != &after {
        return Err("route save/reopen changed layout".into());
    }
    document.undo(document.session().revision())?;
    if document.session().project() != &middle {
        return Err("anchor undo left extra bends".into());
    }
    document.undo(document.session().revision())?;
    if document.session().project() != &before {
        return Err("segment undo did not restore original whole routes".into());
    }
    document.redo(document.session().revision())?;
    document.redo(document.session().revision())?;
    if document.session().project() != &after || document.session().is_dirty() {
        return Err("redo did not restore saved route state".into());
    }
    println!(
        "Whole-route segment edit and shared-anchor movement preserved conductor IDs/attachments; exact undo/redo, guarded retermination, and save/reopen passed."
    );
    Ok(())
}
