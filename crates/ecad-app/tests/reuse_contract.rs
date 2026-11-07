//! Immutable snapshots, independent occurrences, and explicit local-edit transactions.

use ecad_app::{Command, EditError, Session, instance_commands};
use ecad_domain::*;
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

fn source(revision: u128) -> DefinitionRef {
    DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(revision).unwrap(),
    )
}
fn sid(value: u128) -> SubsystemId {
    SubsystemId::new(value).unwrap()
}
fn tid(value: u128) -> TerminalId {
    TerminalId::new(value).unwrap()
}

fn revision() -> Arc<SubsystemRevision> {
    let mut body = Project::new(ProjectId::new(1).unwrap());
    body.insert_subsystem(Subsystem::new(sid(10), "Assembly", None).unwrap())
        .unwrap();
    body.insert_subsystem(Subsystem::new(sid(20), "Sensor", Some(sid(10))).unwrap())
        .unwrap();
    for (id, owner, terminal) in [(1, 10, 101), (2, 20, 201)] {
        body.insert_device(
            Device::new(
                DeviceId::new(id).unwrap(),
                format!("-X{id}"),
                [Terminal::new(tid(terminal), "1").unwrap()],
            )
            .unwrap()
            .in_subsystem(sid(owner)),
        )
        .unwrap();
    }
    body.insert_port(
        InterfacePort::new(InterfacePortId::new(1).unwrap(), sid(10), "Input", tid(101)).unwrap(),
    )
    .unwrap();
    body.insert_port(
        InterfacePort::new(
            InterfacePortId::new(2).unwrap(),
            sid(20),
            "Signal",
            tid(201),
        )
        .unwrap(),
    )
    .unwrap();
    let core = ConductorId::new(1).unwrap();
    body.insert_conductor(Conductor::new(core, Some(tid(101)), Some(tid(201))))
        .unwrap();
    body.insert_cable(
        Cable::new(
            CableId::new(1).unwrap(),
            "-C1",
            CableEndpoint::Device(DeviceId::new(1).unwrap()),
            CableEndpoint::Subsystem(sid(20)),
            [core],
        )
        .unwrap(),
    )
    .unwrap();
    Arc::new(SubsystemRevision::new(source(1), sid(10), body).unwrap())
}

fn ids(offset: u128) -> InstanceIds {
    InstanceIds {
        subsystems: [10, 20].map(|id| (sid(id), sid(id + offset))).into(),
        devices: [1, 2]
            .map(|id| {
                (
                    DeviceId::new(id).unwrap(),
                    DeviceId::new(id + offset).unwrap(),
                )
            })
            .into(),
        terminals: [101, 201].map(|id| (tid(id), tid(id + offset))).into(),
        ports: [1, 2]
            .map(|id| {
                (
                    InterfacePortId::new(id).unwrap(),
                    InterfacePortId::new(id + offset).unwrap(),
                )
            })
            .into(),
        conductors: BTreeMap::from([(
            ConductorId::new(1).unwrap(),
            ConductorId::new(offset + 1).unwrap(),
        )]),
        cables: BTreeMap::from([(CableId::new(1).unwrap(), CableId::new(offset + 1).unwrap())]),
    }
}
fn session() -> Session {
    let mut project = Project::new(ProjectId::new(77).unwrap());
    project.insert_definition(revision()).unwrap();
    Session::new(project)
}
fn place(session: &mut Session, offset: u128) {
    let commands = instance_commands(
        session.project(),
        source(1),
        ids(offset),
        None,
        &format!("Unit {offset}"),
        InstanceLimits::default(),
    )
    .unwrap();
    session.execute_batch(session.revision(), commands).unwrap();
}

#[test]
fn repeated_instances_have_distinct_terminals_wires_and_pinned_correspondence() {
    let mut session = session();
    place(&mut session, 1_000);
    place(&mut session, 2_000);
    assert_eq!(session.project().definition_count(), 1);
    assert_eq!(session.project().instances().len(), 2);
    assert_eq!(session.project().devices().len(), 4);
    assert_eq!(
        session
            .project()
            .conductor(ConductorId::new(1_001).unwrap())
            .unwrap()
            .endpoint(End::A),
        Some(tid(1_101))
    );
    assert_eq!(
        session
            .project()
            .conductor(ConductorId::new(2_001).unwrap())
            .unwrap()
            .endpoint(End::A),
        Some(tid(2_101))
    );
    assert_eq!(
        session.project().instance(sid(1_010)).unwrap().source(),
        source(1)
    );
    let complete = session.project().clone();
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project().instances().len(), 1);
    session.redo(session.revision()).unwrap();
    assert_eq!(session.project(), &complete);
    session.project().validate().unwrap();
}

#[test]
fn pinned_internals_require_explicit_detach_and_undo_rebinds_exactly() {
    let mut session = session();
    place(&mut session, 1_000);
    place(&mut session, 2_000);
    let before = session.project().clone();
    let change = Command::SetTermination {
        conductor: ConductorId::new(1_001).unwrap(),
        end: End::B,
        terminal: None,
    };
    assert!(matches!(
        session.execute(session.revision(), change.clone()),
        Err(EditError::Model(ModelError::Reuse(
            ReuseError::ReadOnlyInstance(_)
        )))
    ));
    assert_eq!(session.project(), &before);
    session
        .execute_batch(
            session.revision(),
            vec![Command::DetachInstance(sid(1_010)), change],
        )
        .unwrap();
    assert!(session.project().instance(sid(1_010)).is_none());
    assert!(session.project().instance(sid(2_010)).is_some());
    assert_eq!(
        session.project().definition(source(1)).unwrap().content(),
        revision().content()
    );
    assert_eq!(
        session
            .project()
            .conductor(ConductorId::new(2_001).unwrap())
            .unwrap()
            .endpoint(End::B),
        Some(tid(2_201))
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}

#[test]
fn failed_local_edit_transaction_restores_binding_ownership_and_redo() {
    let mut session = session();
    place(&mut session, 1_000);
    let before = session.project().clone();
    let revision = session.revision();
    assert!(
        session
            .execute_batch(
                revision,
                vec![
                    Command::DetachInstance(sid(1_010)),
                    Command::SetTermination {
                        conductor: ConductorId::new(1_001).unwrap(),
                        end: End::B,
                        terminal: None
                    },
                    Command::RemoveDevice(DeviceId::new(999).unwrap())
                ]
            )
            .is_err()
    );
    assert_eq!(session.project(), &before);
    assert_eq!(session.revision(), revision);
    assert_eq!(session.undo_len(), 1);
    session.project().validate().unwrap();
}

#[test]
fn immutable_key_conflicts_are_rejected_and_newer_imports_do_not_update_instances() {
    let mut session = session();
    place(&mut session, 1_000);
    let mut changed = revision().content().clone();
    changed
        .set_termination(ConductorId::new(1).unwrap(), End::B, None)
        .unwrap();
    let conflict = Arc::new(SubsystemRevision::new(source(1), sid(10), changed.clone()).unwrap());
    assert!(matches!(
        session.execute(session.revision(), Command::InsertDefinition(conflict)),
        Err(EditError::Model(ModelError::Reuse(
            ReuseError::RevisionConflict(_)
        )))
    ));
    let newer = Arc::new(SubsystemRevision::new(source(2), sid(10), changed).unwrap());
    session
        .execute(session.revision(), Command::InsertDefinition(newer))
        .unwrap();
    assert_eq!(
        session.project().instance(sid(1_010)).unwrap().source(),
        source(1)
    );
    assert_eq!(
        session
            .project()
            .conductor(ConductorId::new(1_001).unwrap())
            .unwrap()
            .endpoint(End::B),
        Some(tid(1_201))
    );
    assert!(matches!(
        session.execute(session.revision(), Command::RemoveDefinition(source(1))),
        Err(EditError::Model(ModelError::Reuse(
            ReuseError::RevisionInUse(_)
        )))
    ));
    assert!(
        !session
            .execute(session.revision(), Command::InsertDefinition(revision()))
            .unwrap()
    );
}

#[test]
fn complete_mapping_and_object_budget_are_checked_before_applying_edits() {
    let session = session();
    let mut missing = ids(1_000);
    missing.terminals.remove(&tid(101));
    assert!(matches!(
        instance_commands(
            session.project(),
            source(1),
            missing,
            None,
            "Unit",
            InstanceLimits::default()
        ),
        Err(ModelError::Reuse(ReuseError::InvalidMapping))
    ));
    let mut duplicated = ids(1_000);
    duplicated.terminals.insert(tid(101), tid(1_201));
    assert!(matches!(
        instance_commands(
            session.project(),
            source(1),
            duplicated,
            None,
            "Unit",
            InstanceLimits::default()
        ),
        Err(ModelError::Reuse(ReuseError::InvalidMapping))
    ));
    let limit = InstanceLimits {
        max_objects: NonZeroUsize::new(1).unwrap(),
    };
    assert!(matches!(
        instance_commands(
            session.project(),
            source(1),
            ids(1_000),
            None,
            "Unit",
            limit
        ),
        Err(ModelError::Reuse(ReuseError::ExpansionLimit))
    ));
    assert_eq!(session.project().subsystems().len(), 0);
}

#[test]
fn late_identity_collision_rolls_back_the_placed_prefix_without_erasing_redo() {
    let mut session = session();
    session
        .execute(
            session.revision(),
            Command::InsertDevice(
                Device::new(DeviceId::new(1_001).unwrap(), "Existing", []).unwrap(),
            ),
        )
        .unwrap();
    session
        .execute(
            session.revision(),
            Command::InsertSubsystem(Subsystem::new(sid(99), "Marker", None).unwrap()),
        )
        .unwrap();
    session.undo(session.revision()).unwrap();
    let before = session.project().clone();
    let revision = session.revision();
    let commands = instance_commands(
        session.project(),
        source(1),
        ids(1_000),
        None,
        "Unit",
        InstanceLimits::default(),
    )
    .unwrap();
    assert!(session.execute_batch(revision, commands).is_err());
    assert_eq!(session.project(), &before);
    assert_eq!(session.revision(), revision);
    assert_eq!(session.redo_len(), 1);
}

#[test]
fn outside_wiring_uses_root_interfaces_and_cannot_reach_hidden_internal_terminals() {
    let mut session = session();
    place(&mut session, 1_000);
    session
        .execute(
            session.revision(),
            Command::InsertConductor(Conductor::new(
                ConductorId::new(99).unwrap(),
                Some(tid(1_101)),
                None,
            )),
        )
        .unwrap();
    let before = session.project().clone();
    assert!(matches!(
        session.execute(
            session.revision(),
            Command::InsertConductor(Conductor::new(
                ConductorId::new(98).unwrap(),
                Some(tid(1_201)),
                None
            ))
        ),
        Err(EditError::Model(ModelError::Reuse(
            ReuseError::UnexposedInstanceTerminal(_)
        )))
    ));
    assert!(
        session
            .execute(
                session.revision(),
                Command::InsertDevice(
                    Device::new(DeviceId::new(99).unwrap(), "Hidden addition", [])
                        .unwrap()
                        .in_subsystem(sid(1_010))
                )
            )
            .is_err()
    );
    assert_eq!(session.project(), &before);
}

#[test]
fn definitions_reject_foreign_roots_and_nested_reusable_dependencies() {
    let original = revision();
    assert!(matches!(
        SubsystemRevision::new(source(2), sid(20), original.content().clone()),
        Err(ModelError::Reuse(ReuseError::InvalidDefinitionRoot))
    ));
    let mut extra = original.content().clone();
    extra
        .insert_subsystem(Subsystem::new(sid(99), "Other root", None).unwrap())
        .unwrap();
    assert!(matches!(
        SubsystemRevision::new(source(2), sid(10), extra),
        Err(ModelError::Reuse(ReuseError::InvalidDefinitionRoot))
    ));
    let mut nested = original.content().clone();
    nested.insert_definition(original).unwrap();
    assert!(matches!(
        SubsystemRevision::new(source(2), sid(10), nested),
        Err(ModelError::Reuse(ReuseError::NestedDefinitionsUnsupported))
    ));
}

#[test]
fn every_owned_structure_edit_requires_detach_but_root_reparenting_is_a_parameter() {
    let mut session = session();
    session
        .execute(
            session.revision(),
            Command::InsertSubsystem(Subsystem::new(sid(99), "Container", None).unwrap()),
        )
        .unwrap();
    place(&mut session, 1_000);
    let before = session.project().clone();
    for command in [
        Command::RemoveDevice(DeviceId::new(1_001).unwrap()),
        Command::RemoveSubsystem(sid(1_020)),
        Command::RemovePort(InterfacePortId::new(1_001).unwrap()),
        Command::RemoveCable(CableId::new(1_001).unwrap()),
        Command::RemoveConductor(ConductorId::new(1_001).unwrap()),
        Command::SetDeviceSubsystem {
            device: DeviceId::new(1_001).unwrap(),
            subsystem: None,
        },
        Command::SetSubsystemParent {
            subsystem: sid(1_020),
            parent: None,
        },
    ] {
        assert!(session.execute(session.revision(), command).is_err());
        assert_eq!(session.project(), &before);
    }
    session
        .execute(
            session.revision(),
            Command::SetSubsystemParent {
                subsystem: sid(1_010),
                parent: Some(sid(99)),
            },
        )
        .unwrap();
    assert_eq!(
        session.project().instance(sid(1_010)).unwrap().source(),
        source(1)
    );
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}
