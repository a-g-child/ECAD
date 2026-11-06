//! Regression contracts for history identity, stale edits, and failed-operation atomicity.

use ecad_app::{Command, EditError, Session};
use ecad_domain::{
    Conductor, ConductorId, Device, DeviceId, End, ModelError, Project, Terminal, TerminalId,
};
use std::num::NonZeroUsize;

fn device(id: u128) -> Device {
    Device::new(
        DeviceId::new(id).unwrap(),
        format!("-X{id}"),
        [Terminal::new(TerminalId::new(id).unwrap(), "1").unwrap()],
    )
    .unwrap()
}

fn execute(session: &mut Session, command: Command) {
    assert!(session.execute(session.revision(), command).unwrap());
}

#[test]
fn undo_redo_restores_exact_device_terminal_and_conductor_identities() {
    let mut session = Session::new(Project::default());
    execute(&mut session, Command::InsertDevice(device(1)));
    execute(&mut session, Command::InsertDevice(device(2)));
    let id = ConductorId::new(1).unwrap();
    execute(
        &mut session,
        Command::InsertConductor(Conductor::new(
            id,
            Some(TerminalId::new(1).unwrap()),
            Some(TerminalId::new(2).unwrap()),
        )),
    );
    let connected = session.project().clone();
    execute(&mut session, Command::RemoveConductor(id));
    execute(
        &mut session,
        Command::RemoveDevice(DeviceId::new(2).unwrap()),
    );
    let removed = session.project().clone();
    assert!(session.undo(session.revision()).unwrap());
    assert!(session.undo(session.revision()).unwrap());
    assert_eq!(session.project(), &connected);
    assert!(session.redo(session.revision()).unwrap());
    assert!(session.redo(session.revision()).unwrap());
    assert_eq!(session.project(), &removed);
}

#[test]
fn rejected_edit_preserves_model_revision_and_redo() {
    let mut session = Session::new(Project::default());
    execute(&mut session, Command::InsertDevice(device(1)));
    execute(&mut session, Command::InsertDevice(device(2)));
    session.undo(session.revision()).unwrap();
    let original = session.project().clone();
    let revision = session.revision();
    let counts = (session.undo_len(), session.redo_len());
    assert!(matches!(
        session.execute(revision, Command::InsertDevice(device(1))),
        Err(EditError::Model(ModelError::DuplicateDevice(_)))
    ));
    assert_eq!(session.project(), &original);
    assert_eq!(session.revision(), revision);
    assert_eq!((session.undo_len(), session.redo_len()), counts);
    assert!(session.redo(revision).unwrap());
    assert!(
        session
            .project()
            .device(DeviceId::new(2).unwrap())
            .is_some()
    );
}

#[test]
fn no_op_does_not_erase_redo_or_advance_revision() {
    let mut session = Session::new(Project::default());
    let id = ConductorId::new(1).unwrap();
    execute(
        &mut session,
        Command::InsertConductor(Conductor::new(id, None, None)),
    );
    execute(&mut session, Command::InsertDevice(device(1)));
    session.undo(session.revision()).unwrap();
    let revision = session.revision();
    assert!(
        !session
            .execute(
                revision,
                Command::SetTermination {
                    conductor: id,
                    end: End::A,
                    terminal: None
                }
            )
            .unwrap()
    );
    assert_eq!(session.revision(), revision);
    assert_eq!(session.redo_len(), 1);
    assert!(session.redo(revision).unwrap());
}

#[test]
fn new_edit_after_undo_discards_only_the_abandoned_redo_branch() {
    let mut session = Session::new(Project::default());
    execute(&mut session, Command::InsertDevice(device(1)));
    execute(&mut session, Command::InsertDevice(device(2)));
    session.undo(session.revision()).unwrap();
    execute(&mut session, Command::InsertDevice(device(3)));
    assert_eq!(session.redo_len(), 0);
    assert!(!session.redo(session.revision()).unwrap());
    session.undo(session.revision()).unwrap();
    assert!(
        session
            .project()
            .device(DeviceId::new(1).unwrap())
            .is_some()
    );
    assert!(
        session
            .project()
            .device(DeviceId::new(2).unwrap())
            .is_none()
    );
    assert!(
        session
            .project()
            .device(DeviceId::new(3).unwrap())
            .is_none()
    );
}

#[test]
fn undo_advances_revision_so_pre_undo_commands_cannot_replay() {
    let mut session = Session::new(Project::default());
    execute(&mut session, Command::InsertDevice(device(1)));
    let old = session.revision();
    session.undo(old).unwrap();
    let current = session.revision();
    assert!(current.get() > old.get());
    assert!(matches!(
        session.execute(old, Command::InsertDevice(device(2))),
        Err(EditError::StaleRevision { .. })
    ));
    assert_eq!(session.revision(), current);
    assert_eq!(session.redo_len(), 1);
    assert!(matches!(
        session.undo(old),
        Err(EditError::StaleRevision { .. })
    ));
    assert!(matches!(
        session.redo(old),
        Err(EditError::StaleRevision { .. })
    ));
}

#[test]
fn history_limit_evicts_oldest_operation_without_deleting_document_objects() {
    let mut session =
        Session::with_history_limit(Project::default(), NonZeroUsize::new(2).unwrap());
    for id in 1..=3 {
        execute(&mut session, Command::InsertDevice(device(id)));
    }
    assert_eq!(session.undo_len(), 2);
    session.undo(session.revision()).unwrap();
    session.undo(session.revision()).unwrap();
    assert!(!session.undo(session.revision()).unwrap());
    assert_eq!(session.project().devices().count(), 1);
    assert!(
        session
            .project()
            .device(DeviceId::new(1).unwrap())
            .is_some()
    );
    session.redo(session.revision()).unwrap();
    session.redo(session.revision()).unwrap();
    assert_eq!(session.project().devices().count(), 3);
}
