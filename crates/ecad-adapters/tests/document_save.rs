//! Content-based conflicts, first-save safety, and atomic lifecycle checkpoints.

mod common;

use common::{device, fixture, seed};
use ecad_adapters::{JsonFile, ProjectFileError};
use ecad_app::{Document, ProjectStore, VersionedProjectStore};

#[test]
fn new_document_saves_and_tracks_its_observed_revision_without_changing_history() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("new.json"));
    let mut document = Document::new(fixture(), store.clone());
    assert!(document.session().is_dirty());
    assert!(!document.has_external_changes().unwrap());
    document.save().unwrap();
    assert!(!document.session().is_dirty());
    document
        .execute(document.session().revision(), device(3))
        .unwrap();
    let revision = document.session().revision();
    document.save().unwrap();
    assert_eq!(document.session().revision(), revision);
    assert_eq!(document.session().undo_len(), 1);
    assert!(!document.session().is_dirty());
    assert_eq!(store.load().unwrap(), *document.session().project());
    document.undo(revision).unwrap();
    assert!(document.session().is_dirty());
    document.redo(document.session().revision()).unwrap();
    assert!(!document.session().is_dirty());
}

#[test]
fn same_length_external_edit_is_detected_and_never_overwritten() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let store = seed(&path);
    let mut document = Document::open(store).unwrap();
    document
        .execute(document.session().revision(), device(3))
        .unwrap();
    let local = document.session().project().clone();
    let revision = document.session().revision();
    let original = std::fs::read_to_string(&path).unwrap();
    let edited = original.replace("-X1", "-Y1");
    assert_eq!(original.len(), edited.len());
    std::fs::write(&path, &edited).unwrap();
    assert!(document.has_external_changes().unwrap());
    assert!(matches!(
        document.save(),
        Err(ProjectFileError::ExternalChange(_))
    ));
    assert_eq!(document.session().project(), &local);
    assert_eq!(document.session().revision(), revision);
    assert_eq!(document.session().undo_len(), 1);
    assert!(document.session().is_dirty());
    assert_eq!(std::fs::read_to_string(path).unwrap(), edited);
}

#[test]
fn deleted_destination_is_not_silently_recreated_by_an_open_document() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let mut document = Document::open(seed(&path)).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(document.has_external_changes().unwrap());
    assert!(matches!(
        document.save(),
        Err(ProjectFileError::ExternalChange(_))
    ));
    assert!(!path.exists());
    assert!(!document.session().is_dirty());
}

#[test]
fn first_save_and_save_as_do_not_overwrite_unobserved_destinations() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("existing.json");
    let store = seed(&path);
    let original = std::fs::read(&path).unwrap();
    let mut unsaved = Document::new(fixture(), store.clone());
    assert!(matches!(
        unsaved.save(),
        Err(ProjectFileError::ExternalChange(_))
    ));
    assert!(unsaved.session().is_dirty());
    assert_eq!(std::fs::read(&path).unwrap(), original);

    let source_path = directory.path().join("source.json");
    let mut document = Document::open(seed(&source_path)).unwrap();
    document
        .execute(document.session().revision(), device(3))
        .unwrap();
    assert!(matches!(
        document.save_as(store),
        Err(ProjectFileError::ExternalChange(_))
    ));
    assert!(document.session().is_dirty());
    document.save().unwrap(); // Still bound to source, not the failed save-as.
    assert_eq!(
        JsonFile::new(source_path).load().unwrap().devices().count(),
        3
    );
    assert_eq!(std::fs::read(path).unwrap(), original);
}

#[test]
fn save_as_preserves_lineage_and_rebinds_only_after_successful_creation() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("source.json");
    let destination = directory.path().join("copy.json");
    let mut document = Document::open(seed(&source)).unwrap();
    let lineage = document.session().project().id();
    document
        .execute(document.session().revision(), device(3))
        .unwrap();
    document.save_as(JsonFile::new(&destination)).unwrap();
    assert!(!document.session().is_dirty());
    assert_eq!(document.session().project().id(), lineage);
    assert_eq!(document.session().undo_len(), 1);
    std::fs::write(&source, "external original change").unwrap();
    assert!(!document.has_external_changes().unwrap());
    document
        .execute(document.session().revision(), device(4))
        .unwrap();
    document.save().unwrap();
    assert_eq!(
        JsonFile::new(destination).load().unwrap().devices().count(),
        4
    );
    assert_eq!(
        std::fs::read_to_string(source).unwrap(),
        "external original change"
    );
}

#[test]
fn identical_bytes_at_another_path_do_not_authorize_a_save() {
    let directory = tempfile::tempdir().unwrap();
    let first = seed(&directory.path().join("first.json"));
    let second = seed(&directory.path().join("second.json"));
    let token = first.load_snapshot().unwrap().version;
    assert!(matches!(
        second.save_if_unchanged(&fixture(), Some(&token)),
        Err(ProjectFileError::ExternalChange(_))
    ));
}

#[test]
fn competing_open_documents_cannot_silently_overwrite_each_other() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let store = seed(&path);
    let mut first = Document::open(store.clone()).unwrap();
    let mut second = Document::open(store).unwrap();
    first
        .execute(first.session().revision(), device(3))
        .unwrap();
    first.save().unwrap();
    second
        .execute(second.session().revision(), device(4))
        .unwrap();
    assert!(matches!(
        second.save(),
        Err(ProjectFileError::ExternalChange(_))
    ));
    assert!(second.session().is_dirty());
    assert_eq!(
        JsonFile::new(path).load().unwrap(),
        *first.session().project()
    );
}
