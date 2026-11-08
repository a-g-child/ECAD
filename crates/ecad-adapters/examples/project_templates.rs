//! Offline preservation capture and project bundling with exact one-step undo/reopen.
use ecad_adapters::{JsonFile, capture_legacy_templates};
use ecad_app::Document;
use ecad_domain::{Project, ProjectId};
use std::error::Error;
fn main() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("project.json"));
    let initial = Project::new(ProjectId::new(1)?);
    let mut document = Document::new(initial.clone(), store.clone());
    document.save()?;
    let capture = capture_legacy_templates(document.session())?;
    let count = capture.bodies().len();
    document.execute_batch(capture.revision(), capture.commands())?;
    document.save()?;
    let reopened = Document::open(store)?;
    assert_eq!(reopened.session().project(), document.session().project());
    document.undo(document.session().revision())?;
    assert_eq!(document.session().project(), &initial);
    document.redo(document.session().revision())?;
    assert!(!document.session().is_dirty());
    println!(
        "Captured {count} legacy default layouts, bundled/reopened them offline, and verified exact undo."
    );
    Ok(())
}
