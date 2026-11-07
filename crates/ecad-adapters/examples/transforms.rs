//! Inspect exact rotation/mirror previews, retain connected paths, then save/undo/redo.
use ecad_adapters::{JsonFile, decode_project};
use ecad_app::{Document, LayoutOperation, PreviewObject, preview_layout};
use ecad_domain::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let before = decode_project(include_str!("../tests/fixtures/routed-conveyor-v6.json"))?;
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("transforms.json"));
    let mut document = Document::new(before.clone(), store.clone());
    document.save()?;
    let selection = [1, 2, 3]
        .into_iter()
        .map(|id| AnchorId::new(id).map(ObjectRef::Anchor))
        .collect::<Result<Vec<_>, _>>()?;
    for transform in [
        DrawingTransform::Rotate {
            pivot: DrawingPoint::default(),
            turn: QuarterTurn::Positive90,
        },
        DrawingTransform::Mirror {
            pivot: DrawingPoint { x: 1000, y: 2000 },
            axis: MirrorAxis::Horizontal,
        },
    ] {
        let preview = preview_layout(
            document.session(),
            selection.iter().copied(),
            LayoutOperation::Transform(transform),
        )?;
        let projection = preview.objects().to_vec();
        document.apply_layout_preview(preview)?;
        for object in projection {
            if let PreviewObject::Route(route) = object
                && document.session().project().drawing().route(route.id) != Some(&route)
            {
                return Err("transform preview differed from committed route".into());
            }
        }
    }
    let after = document.session().project().clone();
    if after.conductors().collect::<Vec<_>>() != before.conductors().collect::<Vec<_>>() {
        return Err("transform changed electrical data".into());
    }
    document.save()?;
    if Document::open(store)?.session().project() != &after {
        return Err("transformed geometry changed on reopen".into());
    }
    for _ in 0..2 {
        document.undo(document.session().revision())?;
    }
    if document.session().project() != &before {
        return Err("transform undo was not exact".into());
    }
    for _ in 0..2 {
        document.redo(document.session().revision())?;
    }
    if document.session().project() != &after || document.session().is_dirty() {
        return Err("transform redo did not restore checkpoint".into());
    }
    println!(
        "Rotation/mirroring preview agreement, unchanged connectivity, save/reopen, and exact undo/redo passed."
    );
    Ok(())
}
