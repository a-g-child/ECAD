//! Connected electrical fixtures and one pinned occurrence for property contracts.

use ecad_adapters::{RandomIds, decode_project};
use ecad_app::{Command, Session, instance_commands};
use ecad_domain::*;
use std::sync::Arc;

pub fn fixture() -> Project {
    decode_project(include_str!("../fixtures/controller-conveyor-v4.json")).unwrap()
}
pub fn selection() -> Vec<ObjectRef> {
    vec![
        ObjectRef::Device(DeviceId::new(1).unwrap()),
        ObjectRef::Cable(CableId::new(1).unwrap()),
        ObjectRef::Subsystem(SubsystemId::new(10).unwrap()),
        ObjectRef::Port(InterfacePortId::new(1).unwrap()),
        ObjectRef::Terminal(TerminalId::new(101).unwrap()),
        ObjectRef::Conductor(ConductorId::new(1).unwrap()),
    ]
}
pub fn pinned(session: &mut Session) -> InstanceBinding {
    let body = decode_project(include_str!("../fixtures/sensor-module-v3.json")).unwrap();
    let source = DefinitionRef::new(
        DefinitionId::new(99).unwrap(),
        DefinitionRevisionId::new(1).unwrap(),
    );
    let revision =
        Arc::new(SubsystemRevision::new(source, SubsystemId::new(10).unwrap(), body).unwrap());
    let ids = RandomIds::instance_ids(&revision, InstanceLimits::default()).unwrap();
    let root = ids.subsystems[&revision.root()];
    session
        .execute(session.revision(), Command::InsertDefinition(revision))
        .unwrap();
    let commands = instance_commands(
        session.project(),
        source,
        ids,
        None,
        "Pinned",
        InstanceLimits::default(),
    )
    .unwrap();
    session.execute_batch(session.revision(), commands).unwrap();
    session.project().instance(root).unwrap().clone()
}
