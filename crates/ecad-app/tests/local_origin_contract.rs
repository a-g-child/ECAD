//! Historical ancestry survives local structural changes and explicit deletion/undo.

use ecad_app::{Command, EditError, Session, instance_commands};
use ecad_domain::*;
use std::{collections::BTreeMap, sync::Arc};

#[test]
fn local_ancestry_pins_its_baseline_and_requires_explicit_forgetting_before_root_deletion() {
    let source = DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(1).unwrap(),
    );
    let local_root = SubsystemId::new(10).unwrap();
    let placed_root = SubsystemId::new(1_010).unwrap();
    let mut body = Project::new(ProjectId::new(1).unwrap());
    body.insert_subsystem(Subsystem::new(local_root, "Module", None).unwrap())
        .unwrap();
    body.insert_device(
        Device::new(
            DeviceId::new(1).unwrap(),
            "-X1",
            [Terminal::new(TerminalId::new(101).unwrap(), "1").unwrap()],
        )
        .unwrap()
        .in_subsystem(local_root),
    )
    .unwrap();
    body.insert_port(
        InterfacePort::new(
            InterfacePortId::new(1).unwrap(),
            local_root,
            "Input",
            TerminalId::new(101).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let revision = Arc::new(SubsystemRevision::new(source, local_root, body).unwrap());
    let mut project = Project::new(ProjectId::new(77).unwrap());
    project.insert_definition(revision).unwrap();
    let ids = InstanceIds {
        subsystems: BTreeMap::from([(local_root, placed_root)]),
        devices: BTreeMap::from([(DeviceId::new(1).unwrap(), DeviceId::new(1_001).unwrap())]),
        terminals: BTreeMap::from([(
            TerminalId::new(101).unwrap(),
            TerminalId::new(1_101).unwrap(),
        )]),
        ports: BTreeMap::from([(
            InterfacePortId::new(1).unwrap(),
            InterfacePortId::new(1_001).unwrap(),
        )]),
        ..InstanceIds::default()
    };
    let mut session = Session::new(project);
    let commands = instance_commands(
        session.project(),
        source,
        ids.clone(),
        None,
        "Local",
        InstanceLimits::default(),
    )
    .unwrap();
    session.execute_batch(session.revision(), commands).unwrap();
    session
        .execute_batch(
            session.revision(),
            vec![
                Command::DetachInstance(placed_root),
                Command::RemovePort(InterfacePortId::new(1_001).unwrap()),
                Command::RemoveDevice(DeviceId::new(1_001).unwrap()),
            ],
        )
        .unwrap();
    let origin = session.project().local_origin(placed_root).unwrap();
    assert_eq!(origin.source(), source);
    assert_eq!(origin.ids(), &ids); // Deleted objects remain represented in the historical baseline.
    session.project().validate().unwrap();
    assert!(matches!(
        session.execute(session.revision(), Command::RemoveDefinition(source)),
        Err(EditError::Model(ModelError::Reuse(
            ReuseError::RevisionInUse(_)
        )))
    ));
    assert!(matches!(
        session.execute(session.revision(), Command::RemoveSubsystem(placed_root)),
        Err(EditError::Model(ModelError::Reuse(
            ReuseError::LocalOriginExists(_)
        )))
    ));
    let before = session.project().clone();
    session
        .execute_batch(
            session.revision(),
            vec![
                Command::ForgetLocalOrigin(placed_root),
                Command::RemoveSubsystem(placed_root),
                Command::RemoveDefinition(source),
            ],
        )
        .unwrap();
    assert_eq!(session.project().subsystems().len(), 0);
    assert_eq!(session.project().definition_count(), 0);
    session.undo(session.revision()).unwrap();
    assert_eq!(session.project(), &before);
}
