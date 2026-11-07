//! Real file IO with saved checkpoints, history branching, and session epochs.

use ecad_adapters::{JsonFile, decode_project};
use ecad_app::{Command, EditError, ProjectStore, Session};
use ecad_domain::{ConductorId, Device, DeviceId, End, Project};
use std::num::NonZeroUsize;

fn device(value: u128) -> Command {
    Command::InsertDevice(
        Device::new(DeviceId::new(value).unwrap(), format!("-X{value}"), []).unwrap(),
    )
}

#[test]
fn successful_save_preserves_revision_and_history_and_undo_redo_tracks_checkpoint() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("project.json"));
    let mut session = Session::new(Project::new(ecad_domain::ProjectId::new(1).unwrap()));
    assert!(session.is_dirty());
    session.save(&store).unwrap();
    assert!(!session.is_dirty());
    session.execute(session.revision(), device(1)).unwrap();
    assert!(session.is_dirty());
    session.undo(session.revision()).unwrap();
    assert!(!session.is_dirty());
    session.redo(session.revision()).unwrap();
    assert!(session.is_dirty());
    let revision = session.revision();
    let history = (session.undo_len(), session.redo_len());
    session.save(&store).unwrap();
    assert_eq!(session.revision(), revision);
    assert_eq!((session.undo_len(), session.redo_len()), history);
    assert!(!session.is_dirty());
    session.undo(session.revision()).unwrap();
    assert!(session.is_dirty());
    session.redo(session.revision()).unwrap();
    assert!(!session.is_dirty());
    assert_eq!(store.load().unwrap(), *session.project());
}

#[test]
fn failed_save_does_not_move_checkpoint_or_discard_edits() {
    let directory = tempfile::tempdir().unwrap();
    let good = JsonFile::new(directory.path().join("project.json"));
    let bad = JsonFile::new(directory.path()); // Replacing a directory must fail.
    let mut session = Session::new(Project::new(ecad_domain::ProjectId::new(1).unwrap()));
    session.save(&good).unwrap();
    assert!(session.save(&bad).is_err());
    assert!(!session.is_dirty());
    session.execute(session.revision(), device(1)).unwrap();
    let revision = session.revision();
    assert!(session.save(&bad).is_err());
    assert!(session.is_dirty());
    assert_eq!(session.revision(), revision);
    assert_eq!(session.undo_len(), 1);
    assert_eq!(
        good.load().unwrap(),
        Project::new(ecad_domain::ProjectId::new(1).unwrap())
    );
    session.undo(revision).unwrap();
    assert!(!session.is_dirty());
}

#[test]
fn divergent_branch_never_inherits_the_discarded_branch_saved_status() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("project.json"));
    let mut session = Session::new(Project::new(ecad_domain::ProjectId::new(1).unwrap()));
    session.execute(session.revision(), device(1)).unwrap();
    session.save(&store).unwrap();
    session.undo(session.revision()).unwrap();
    session.execute(session.revision(), device(2)).unwrap();
    assert!(session.is_dirty());
    assert_eq!(session.redo_len(), 0);
    assert_ne!(store.load().unwrap(), *session.project());
}

#[test]
fn reopened_documents_start_clean_with_empty_history_and_a_new_epoch() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("project.json"));
    let mut first = Session::new(Project::new(ecad_domain::ProjectId::new(1).unwrap()));
    first.save(&store).unwrap();
    let old = first.revision();
    let mut reopened = Session::open(&store).unwrap();
    assert!(!reopened.is_dirty());
    assert_eq!(reopened.undo_len(), 0);
    assert_eq!(reopened.redo_len(), 0);
    assert_eq!(reopened.revision().get(), old.get());
    assert_ne!(reopened.revision().epoch(), old.epoch());
    assert!(matches!(
        reopened.execute(old, device(1)),
        Err(EditError::StaleRevision { .. })
    ));
    assert!(!reopened.is_dirty());
}

#[test]
fn compound_rollback_and_no_ops_preserve_a_clean_checkpoint_and_redo() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("project.json"));
    let original = decode_project(include_str!("fixtures/minimal-v4.json")).unwrap();
    store.save(&original).unwrap();
    let mut session = Session::open(&store).unwrap();
    session.execute(session.revision(), device(3)).unwrap();
    session.undo(session.revision()).unwrap();
    let revision = session.revision();
    assert!(
        session
            .execute_batch(revision, vec![device(4), device(1)])
            .is_err()
    );
    assert!(!session.is_dirty());
    assert_eq!(session.project(), &original);
    assert_eq!(session.redo_len(), 1);
    assert!(
        !session
            .execute_batch(
                revision,
                vec![Command::SetTermination {
                    conductor: ConductorId::new(1001).unwrap(),
                    end: End::B,
                    terminal: Some(ecad_domain::TerminalId::new(201).unwrap())
                }]
            )
            .unwrap()
    );
    assert!(!session.is_dirty());
    assert_eq!(session.redo_len(), 1);
}

#[test]
fn checkpoint_is_not_falsely_reached_after_history_eviction() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("project.json"));
    let mut session = Session::with_history_limit(
        Project::new(ecad_domain::ProjectId::new(1).unwrap()),
        NonZeroUsize::new(1).unwrap(),
    );
    session.save(&store).unwrap();
    session.execute(session.revision(), device(1)).unwrap();
    session.execute(session.revision(), device(2)).unwrap();
    session.undo(session.revision()).unwrap();
    assert!(!session.undo(session.revision()).unwrap());
    assert!(session.is_dirty());
    assert_eq!(session.project().devices().count(), 1);
}

#[test]
fn net_zero_batch_is_conservatively_dirty_until_undone_or_saved() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("project.json"));
    let mut session = Session::new(Project::new(ecad_domain::ProjectId::new(1).unwrap()));
    session.save(&store).unwrap();
    session
        .execute_batch(
            session.revision(),
            vec![device(1), Command::RemoveDevice(DeviceId::new(1).unwrap())],
        )
        .unwrap();
    assert_eq!(
        session.project(),
        &Project::new(ecad_domain::ProjectId::new(1).unwrap())
    );
    assert!(session.is_dirty());
    session.undo(session.revision()).unwrap();
    assert!(!session.is_dirty());
}
