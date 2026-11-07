//! Preview a snapped group move, inspect exact geometry, then commit/save/undo normally.

use ecad_adapters::{JsonFile, decode_project};
use ecad_app::{Document, MovePlacement, PreviewObject, SnapChoice, preview_move};
use ecad_domain::*;
use std::collections::BTreeSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project = decode_project(include_str!("../tests/fixtures/routed-conveyor-v6.json"))?;
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("placement.json"));
    let mut document = Document::new(project, store.clone());
    document.save()?;
    let before = document.session().project().clone();
    let selection = [1, 2, 3]
        .into_iter()
        .map(|id| AnchorId::new(id).map(ObjectRef::Anchor))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let preview = preview_move(
        document.session(),
        selection,
        ObjectRef::Anchor(AnchorId::new(1)?),
        MovePlacement::Pointer {
            position: DrawingPoint {
                x: 10_090,
                y: 1_930,
            },
            choice: SnapChoice::Best,
            settings: SnapSettings {
                tolerance_um: 200,
                terminals: true,
                endpoints: true,
                midpoints: true,
                grid: Some(DrawingGrid {
                    origin: DrawingPoint::default(),
                    spacing: DrawingLength::new(1000)?,
                }),
            },
        },
    )?;
    println!(
        "{} candidates; destination {:?}",
        preview.candidates().len(),
        preview.destination()
    );
    let routes: Vec<_> = preview
        .layout()
        .objects()
        .iter()
        .filter_map(|object| match object {
            PreviewObject::Route(route) => Some(route.clone()),
            _ => None,
        })
        .collect();
    if document.session().project() != &before || document.session().is_dirty() {
        return Err("preview changed the document".into());
    }
    document.apply_layout_preview(preview.into_layout())?;
    for route in routes {
        if document.session().project().drawing().route(route.id) != Some(&route) {
            return Err("preview and commit differ".into());
        }
    }
    document.save()?;
    let after = document.session().project().clone();
    if Document::open(store)?.session().project() != &after {
        return Err("snapped geometry changed on reopen".into());
    }
    document.undo(document.session().revision())?;
    if document.session().project() != &before {
        return Err("placement undo was not exact".into());
    }
    document.redo(document.session().revision())?;
    if document.session().project() != &after || document.session().is_dirty() {
        return Err("placement redo did not restore checkpoint".into());
    }
    println!("Transient preview, exact commit, save/reopen, and grouped undo/redo passed.");
    Ok(())
}
