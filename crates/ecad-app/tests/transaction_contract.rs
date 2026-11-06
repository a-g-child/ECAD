//! Compound-operation regression contracts: dependency ordering and atomic rollback.

use ecad_app::{Command, EditError, Session};
use ecad_domain::{Conductor, ConductorId, Device, DeviceId, End, Project, Terminal, TerminalId};
use std::num::NonZeroUsize;

fn device(value: u128) -> Command {
    Command::InsertDevice(
        Device::new(
            DeviceId::new(value).unwrap(),
            format!("-X{value}"),
            [Terminal::new(TerminalId::new(value).unwrap(), "1").unwrap()],
        )
        .unwrap(),
    )
}

fn conductor() -> Command {
    Command::InsertConductor(Conductor::new(
        ConductorId::new(1).unwrap(),
        Some(TerminalId::new(1).unwrap()),
        Some(TerminalId::new(2).unwrap()),
    ))
}

fn assembly() -> Vec<Command> {
    vec![device(1), device(2), conductor()]
}

#[test]
fn dependent_multi_object_creation_is_one_complete_undo_and_redo() {
    let mut session = Session::new(Project::default());
    assert!(
        session
            .execute_batch(session.revision(), assembly())
            .unwrap()
    );
    let complete = session.project().clone();
    assert_eq!(complete.devices().count(), 2);
    assert_eq!(complete.conductors().count(), 1);
    assert_eq!(session.undo_len(), 1);
    assert_eq!(session.revision().get(), 1);
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &Project::default());
    session.redo(session.revision()).unwrap();
    assert_eq!(session.project(), &complete);
}

#[test]
fn failure_at_each_batch_position_restores_model_revision_and_history() {
    for position in 0..=3 {
        let mut session = Session::new(Project::default());
        session.execute(session.revision(), device(99)).unwrap();
        session.execute(session.revision(), device(98)).unwrap();
        session.undo(session.revision()).unwrap();
        let original = session.project().clone();
        let revision = session.revision();
        let history = (session.undo_len(), session.redo_len());
        let mut commands = assembly();
        commands.insert(position, device(99));
        assert!(matches!(
            session.execute_batch(revision, commands),
            Err(EditError::Model(_))
        ));
        assert_eq!(session.project(), &original, "failure position {position}");
        assert_eq!(session.revision(), revision);
        assert_eq!((session.undo_len(), session.redo_len()), history);
        assert!(session.redo(revision).unwrap());
    }
}

#[test]
fn rollback_restores_deleted_devices_and_connections_in_dependency_order() {
    let mut session = Session::new(Project::default());
    session
        .execute_batch(session.revision(), assembly())
        .unwrap();
    let original = session.project().clone();
    let revision = session.revision();
    let result = session.execute_batch(
        revision,
        vec![
            Command::RemoveConductor(ConductorId::new(1).unwrap()),
            Command::RemoveDevice(DeviceId::new(2).unwrap()),
            device(1), // Duplicate: both preceding removals must be reversed.
        ],
    );
    assert!(matches!(result, Err(EditError::Model(_))));
    assert_eq!(session.project(), &original);
    assert_eq!(session.revision(), revision);
    assert_eq!(session.undo_len(), 1);
    session.project().validate().unwrap();
}

#[test]
fn empty_and_all_no_op_batches_preserve_revision_and_redo() {
    let mut session = Session::new(Project::default());
    session
        .execute_batch(session.revision(), assembly())
        .unwrap();
    session.execute(session.revision(), device(3)).unwrap();
    session.undo(session.revision()).unwrap();
    let revision = session.revision();
    assert!(!session.execute_batch(revision, vec![]).unwrap());
    let unchanged = Command::SetTermination {
        conductor: ConductorId::new(1).unwrap(),
        end: End::A,
        terminal: Some(TerminalId::new(1).unwrap()),
    };
    assert!(
        !session
            .execute_batch(revision, vec![unchanged.clone(), unchanged])
            .unwrap()
    );
    assert_eq!(session.revision(), revision);
    assert_eq!(session.undo_len(), 1);
    assert_eq!(session.redo_len(), 1);
}

#[test]
fn mixed_no_ops_and_edits_restore_the_original_state() {
    let mut session = Session::new(Project::default());
    session
        .execute_batch(session.revision(), assembly())
        .unwrap();
    let original = session.project().clone();
    let set = |terminal| Command::SetTermination {
        conductor: ConductorId::new(1).unwrap(),
        end: End::A,
        terminal,
    };
    session
        .execute_batch(
            session.revision(),
            vec![
                set(Some(TerminalId::new(1).unwrap())),
                set(None),
                set(None),
                device(3),
            ],
        )
        .unwrap();
    assert_eq!(session.undo_len(), 2);
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &original);
    session.redo(session.revision()).unwrap();
    assert_eq!(
        session
            .project()
            .conductor(ConductorId::new(1).unwrap())
            .unwrap()
            .endpoint(End::A),
        None
    );
}

#[test]
fn history_capacity_counts_transactions_not_individual_commands() {
    let mut session =
        Session::with_history_limit(Project::default(), NonZeroUsize::new(1).unwrap());
    session.execute(session.revision(), device(99)).unwrap();
    session
        .execute_batch(session.revision(), assembly())
        .unwrap();
    assert_eq!(session.undo_len(), 1);
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project().devices().count(), 1);
    assert!(
        session
            .project()
            .device(DeviceId::new(99).unwrap())
            .is_some()
    );
    assert!(!session.undo(session.revision()).unwrap());
    session.redo(session.revision()).unwrap();
    assert_eq!(session.project().devices().count(), 3);
}

#[test]
fn equal_sequence_from_another_session_cannot_edit_undo_or_redo() {
    let first = Session::new(Project::default());
    let mut second = Session::new(Project::default());
    assert_eq!(first.revision().get(), second.revision().get());
    assert_ne!(first.revision().epoch(), second.revision().epoch());
    let foreign = first.revision();
    assert!(matches!(
        second.execute(foreign, device(1)),
        Err(EditError::StaleRevision { .. })
    ));
    assert!(matches!(
        second.execute_batch(foreign, assembly()),
        Err(EditError::StaleRevision { .. })
    ));
    assert!(matches!(
        second.undo(foreign),
        Err(EditError::StaleRevision { .. })
    ));
    assert!(matches!(
        second.redo(foreign),
        Err(EditError::StaleRevision { .. })
    ));
    assert_eq!(second.project(), &Project::default());
    assert_eq!(second.undo_len(), 0);
}
