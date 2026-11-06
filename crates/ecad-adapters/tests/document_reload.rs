//! Replacement validation, explicit discard, and old-projection invalidation.

mod common;

use common::{device, fixture, seed};
use ecad_adapters::{ProjectFileError, encode_project};
use ecad_app::{Document, DocumentError, EditError, ReloadPolicy};

#[test]
fn clean_reload_adopts_external_edits_and_clears_history_with_a_new_epoch() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let mut document = Document::open(seed(&path)).unwrap();
    document
        .execute(document.session().revision(), device(3))
        .unwrap();
    document.save().unwrap();
    let old = document.session().revision();
    let content = std::fs::read_to_string(&path)
        .unwrap()
        .replace("-X1", "-Y1");
    std::fs::write(&path, content).unwrap();
    document.reload(old, ReloadPolicy::RequireClean).unwrap();
    assert_eq!(
        document
            .session()
            .project()
            .devices()
            .next()
            .unwrap()
            .designation(),
        "-Y1"
    );
    assert_eq!(document.session().undo_len(), 0);
    assert_eq!(document.session().redo_len(), 0);
    assert!(!document.session().is_dirty());
    assert!(!document.has_external_changes().unwrap());
    assert_ne!(document.session().revision().epoch(), old.epoch());
    assert!(matches!(
        document.execute(old, device(4)),
        Err(EditError::StaleRevision { .. })
    ));
}

#[test]
fn dirty_reload_requires_explicit_discard_and_only_then_replaces_the_model() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let mut document = Document::open(seed(&path)).unwrap();
    document
        .execute(document.session().revision(), device(3))
        .unwrap();
    let revision = document.session().revision();
    assert!(matches!(
        document.reload(revision, ReloadPolicy::RequireClean),
        Err(DocumentError::UnsavedChanges)
    ));
    assert_eq!(document.session().project().devices().count(), 3);
    assert_eq!(document.session().revision(), revision);
    document
        .reload(revision, ReloadPolicy::DiscardLocalChanges)
        .unwrap();
    assert_eq!(document.session().project(), &fixture());
    assert!(!document.session().is_dirty());
    assert_eq!(document.session().undo_len(), 0);
}

#[test]
fn invalid_replacement_never_discards_edits_or_advances_the_observed_file_token() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let mut document = Document::open(seed(&path)).unwrap();
    document
        .execute(document.session().revision(), device(3))
        .unwrap();
    let revision = document.session().revision();
    let local = document.session().project().clone();
    std::fs::write(&path, "{broken JSON").unwrap();
    assert!(document.has_external_changes().unwrap());
    assert!(matches!(
        document.reload(revision, ReloadPolicy::DiscardLocalChanges),
        Err(DocumentError::Store(_))
    ));
    assert_eq!(document.session().project(), &local);
    assert_eq!(document.session().revision(), revision);
    assert_eq!(document.session().undo_len(), 1);
    assert!(document.session().is_dirty());
    assert!(matches!(
        document.save(),
        Err(ProjectFileError::ExternalChange(_))
    ));
    assert_eq!(std::fs::read_to_string(path).unwrap(), "{broken JSON");
}

#[test]
fn different_project_identity_cannot_replace_this_document_even_with_discard() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let mut document = Document::open(seed(&path)).unwrap();
    let revision = document.session().revision();
    let replacement = encode_project(&fixture()).unwrap().replace(
        "\"project_id\": \"00000000000000000000000000000001\"",
        "\"project_id\": \"00000000000000000000000000000002\"",
    );
    std::fs::write(&path, &replacement).unwrap();
    assert!(matches!(
        document.reload(revision, ReloadPolicy::DiscardLocalChanges),
        Err(DocumentError::DifferentProject { .. })
    ));
    assert_eq!(document.session().project(), &fixture());
    assert_eq!(document.session().revision(), revision);
    assert!(matches!(
        document.save(),
        Err(ProjectFileError::ExternalChange(_))
    ));
    assert_eq!(std::fs::read_to_string(path).unwrap(), replacement);
}

#[test]
fn stale_discard_request_does_not_discard_newer_edits() {
    let directory = tempfile::tempdir().unwrap();
    let mut document = Document::open(seed(&directory.path().join("project.json"))).unwrap();
    let old = document.session().revision();
    document
        .execute(document.session().revision(), device(3))
        .unwrap();
    assert!(matches!(
        document.reload(old, ReloadPolicy::DiscardLocalChanges),
        Err(DocumentError::Edit(EditError::StaleRevision { .. }))
    ));
    assert!(document.session().is_dirty());
    assert_eq!(document.session().project().devices().count(), 3);
}

#[test]
fn missing_replacement_keeps_the_live_document_and_history_intact() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let mut document = Document::open(seed(&path)).unwrap();
    document
        .execute(document.session().revision(), device(3))
        .unwrap();
    let revision = document.session().revision();
    std::fs::remove_file(&path).unwrap();
    assert!(matches!(
        document.reload(revision, ReloadPolicy::DiscardLocalChanges),
        Err(DocumentError::Store(_))
    ));
    assert_eq!(document.session().revision(), revision);
    assert_eq!(document.session().undo_len(), 1);
    assert!(document.session().is_dirty());
}
