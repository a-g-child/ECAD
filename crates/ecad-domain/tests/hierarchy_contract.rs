//! Containment, explicit boundary exposure, and failed-reorganization atomicity.

use ecad_domain::{
    Conductor, ConductorId, Device, DeviceId, End, InterfacePort, InterfacePortId,
    MAX_SUBSYSTEM_DEPTH, ModelError, Project, ProjectId, Subsystem, SubsystemId, Terminal,
    TerminalId,
};

fn sid(value: u128) -> SubsystemId {
    SubsystemId::new(value).unwrap()
}
fn tid(value: u128) -> TerminalId {
    TerminalId::new(value).unwrap()
}
fn port(id: u128, subsystem: u128, terminal: u128) -> InterfacePort {
    InterfacePort::new(
        InterfacePortId::new(id).unwrap(),
        sid(subsystem),
        format!("P{id}"),
        tid(terminal),
    )
    .unwrap()
}
fn device(id: u128, scope: Option<u128>) -> Device {
    let device = Device::new(
        DeviceId::new(id).unwrap(),
        format!("-X{id}"),
        [Terminal::new(tid(id), "1").unwrap()],
    )
    .unwrap();
    if let Some(scope) = scope {
        device.in_subsystem(sid(scope))
    } else {
        device
    }
}
fn fixture() -> Project {
    let mut project = Project::new(ProjectId::new(1).unwrap());
    project
        .insert_subsystem(Subsystem::new(sid(10), "Conveyor", None).unwrap())
        .unwrap();
    project
        .insert_subsystem(Subsystem::new(sid(20), "Sensor assembly", Some(sid(10))).unwrap())
        .unwrap();
    project.insert_device(device(1, None)).unwrap();
    project.insert_device(device(2, Some(20))).unwrap();
    project
}

#[test]
fn nested_crossing_requires_every_exited_boundary_without_duplicating_terminals() {
    let mut project = fixture();
    let wire = Conductor::new(ConductorId::new(1).unwrap(), Some(tid(1)), Some(tid(2)));
    let before = project.clone();
    assert_eq!(
        project.insert_conductor(wire.clone()),
        Err(ModelError::InterfaceRequired(sid(20), tid(2)))
    );
    assert_eq!(project, before);
    project.insert_port(port(1, 20, 2)).unwrap();
    assert_eq!(
        project.insert_conductor(wire.clone()),
        Err(ModelError::InterfaceRequired(sid(10), tid(2)))
    );
    project.insert_port(port(2, 10, 2)).unwrap();
    project.insert_conductor(wire).unwrap();
    assert_eq!(project.ports().count(), 2);
    assert_eq!(project.devices().flat_map(Device::terminals).count(), 2);
    assert_eq!(project.conductors().count(), 1);
    assert_eq!(
        project
            .port(InterfacePortId::new(2).unwrap())
            .unwrap()
            .terminal(),
        tid(2)
    );
    project.validate().unwrap();
}

#[test]
fn exposure_cannot_skip_a_child_boundary_or_reach_a_foreign_terminal() {
    let mut project = fixture();
    let before = project.clone();
    assert_eq!(
        project.insert_port(port(1, 10, 2)),
        Err(ModelError::InterfaceRequired(sid(20), tid(2)))
    );
    assert_eq!(
        project.insert_port(port(2, 10, 1)),
        Err(ModelError::TerminalOutsideSubsystem(sid(10), tid(1)))
    );
    assert_eq!(project, before);
}

#[test]
fn sibling_connections_need_child_exports_but_not_an_unexited_common_parent() {
    let mut project = fixture();
    project
        .insert_subsystem(Subsystem::new(sid(30), "Drive", Some(sid(10))).unwrap())
        .unwrap();
    project.insert_device(device(3, Some(30))).unwrap();
    project.insert_port(port(1, 20, 2)).unwrap();
    project.insert_port(port(2, 30, 3)).unwrap();
    project
        .insert_conductor(Conductor::new(
            ConductorId::new(1).unwrap(),
            Some(tid(2)),
            Some(tid(3)),
        ))
        .unwrap();
    assert!(!project.exposes_terminal(sid(10), tid(2)));
    project.validate().unwrap();
}

#[test]
fn missing_parents_and_cycles_are_rejected_without_reparenting() {
    let mut project = fixture();
    let before = project.clone();
    assert_eq!(
        project.set_subsystem_parent(sid(10), Some(sid(20))),
        Err(ModelError::HierarchyCycle(sid(10)))
    );
    assert_eq!(project, before);
    assert_eq!(
        project.set_subsystem_parent(sid(20), Some(sid(99))),
        Err(ModelError::MissingSubsystem(sid(99)))
    );
    assert_eq!(project, before);
    assert_eq!(
        project.insert_subsystem(Subsystem::new(sid(99), "self", Some(sid(99))).unwrap()),
        Err(ModelError::HierarchyCycle(sid(99)))
    );
    assert_eq!(project, before);
    assert_eq!(
        project.subsystem_path(sid(20)).unwrap(),
        vec![sid(10), sid(20)]
    );
}

#[test]
fn depth_guard_and_deep_descendant_reparent_checks_are_iterative_and_atomic() {
    let mut project = Project::new(ProjectId::new(1).unwrap());
    for depth in 1..=MAX_SUBSYSTEM_DEPTH {
        let parent = if depth == 1 {
            None
        } else {
            Some(sid((depth - 1) as u128))
        };
        project
            .insert_subsystem(Subsystem::new(sid(depth as u128), "Nested", parent).unwrap())
            .unwrap();
    }
    assert_eq!(
        project
            .subsystem_path(sid(MAX_SUBSYSTEM_DEPTH as u128))
            .unwrap()
            .len(),
        MAX_SUBSYSTEM_DEPTH
    );
    let before = project.clone();
    assert_eq!(
        project.insert_subsystem(
            Subsystem::new(sid(999), "Too deep", Some(sid(MAX_SUBSYSTEM_DEPTH as u128))).unwrap()
        ),
        Err(ModelError::HierarchyTooDeep)
    );
    assert_eq!(project, before);
    project
        .insert_subsystem(Subsystem::new(sid(999), "Other root", None).unwrap())
        .unwrap();
    let before_move = project.clone();
    assert_eq!(
        project.set_subsystem_parent(sid(1), Some(sid(999))),
        Err(ModelError::HierarchyTooDeep)
    );
    assert_eq!(project, before_move);
    project.validate().unwrap();
}

#[test]
fn port_removal_and_device_moves_preserve_wired_and_nested_exports_on_failure() {
    let mut project = fixture();
    project.insert_port(port(1, 20, 2)).unwrap();
    project.insert_port(port(2, 10, 2)).unwrap();
    let before = project.clone();
    assert!(matches!(
        project.remove_port(InterfacePortId::new(1).unwrap()),
        Err(ModelError::InterfaceRequired(_, _))
    ));
    assert!(matches!(
        project.set_device_subsystem(DeviceId::new(2).unwrap(), None),
        Err(ModelError::TerminalOutsideSubsystem(_, _))
    ));
    assert_eq!(project, before);
    let wire_id = ConductorId::new(1).unwrap();
    project
        .insert_conductor(Conductor::new(wire_id, Some(tid(1)), Some(tid(2))))
        .unwrap();
    let wired = project.clone();
    assert!(
        project
            .remove_port(InterfacePortId::new(2).unwrap())
            .is_err()
    );
    assert_eq!(project, wired);
    project.set_termination(wire_id, End::A, None).unwrap();
    project
        .remove_port(InterfacePortId::new(2).unwrap())
        .unwrap();
    project
        .remove_port(InterfacePortId::new(1).unwrap())
        .unwrap();
    assert!(!project.exposes_terminal(sid(20), tid(2)));
    project.validate().unwrap();
}

#[test]
fn scopes_ports_and_names_do_not_silently_overwrite_existing_objects() {
    let mut project = fixture();
    project.insert_port(port(1, 20, 2)).unwrap();
    let before = project.clone();
    assert!(matches!(
        project.insert_port(port(1, 20, 2)),
        Err(ModelError::DuplicatePort(_))
    ));
    let duplicate_name =
        InterfacePort::new(InterfacePortId::new(2).unwrap(), sid(20), "P1", tid(2)).unwrap();
    assert!(matches!(
        project.insert_port(duplicate_name),
        Err(ModelError::PortNameInUse(_, _))
    ));
    assert_eq!(
        project.remove_subsystem(sid(10)),
        Err(ModelError::SubsystemInUse(sid(10)))
    );
    assert_eq!(
        project.remove_device(DeviceId::new(2).unwrap()),
        Err(ModelError::DeviceReferenced(DeviceId::new(2).unwrap()))
    );
    assert_eq!(project, before);
}
