//! One physical conductor belongs to one overview assembly, with consistent endpoints.

use ecad_domain::{
    Cable, CableEndpoint, CableId, Conductor, ConductorId, Device, DeviceId, End, InterfacePort,
    InterfacePortId, ModelError, Project, ProjectId, Subsystem, SubsystemId, Terminal, TerminalId,
};

fn tid(value: u128) -> TerminalId {
    TerminalId::new(value).unwrap()
}
fn cid(value: u128) -> ConductorId {
    ConductorId::new(value).unwrap()
}
fn fixture() -> Project {
    let mut project = Project::new(ProjectId::new(1).unwrap());
    let block = SubsystemId::new(1).unwrap();
    project
        .insert_subsystem(Subsystem::new(block, "Conveyor", None).unwrap())
        .unwrap();
    for value in 1..=3 {
        let mut device = Device::new(
            DeviceId::new(value).unwrap(),
            format!("-X{value}"),
            [Terminal::new(tid(value), "1").unwrap()],
        )
        .unwrap();
        if value == 2 {
            device = device.in_subsystem(block);
        }
        project.insert_device(device).unwrap();
    }
    project
        .insert_port(
            InterfacePort::new(InterfacePortId::new(1).unwrap(), block, "Sensor", tid(2)).unwrap(),
        )
        .unwrap();
    project
        .insert_conductor(Conductor::new(cid(1), Some(tid(1)), Some(tid(2))))
        .unwrap();
    project
}
fn cable(id: u128, cores: &[u128]) -> Cable {
    Cable::new(
        CableId::new(id).unwrap(),
        format!("-C{id}"),
        CableEndpoint::Device(DeviceId::new(1).unwrap()),
        CableEndpoint::Subsystem(SubsystemId::new(1).unwrap()),
        cores.iter().map(|id| cid(*id)),
    )
    .unwrap()
}

#[test]
fn cable_groups_existing_cores_and_removal_preserves_their_identity_and_wiring() {
    let mut project = fixture();
    let before = project.clone();
    project.insert_cable(cable(1, &[1])).unwrap();
    assert_eq!(project.conductors().count(), 1);
    assert_eq!(
        project.conductor_cable(cid(1)),
        Some(CableId::new(1).unwrap())
    );
    assert_eq!(
        project.remove_conductor(cid(1)),
        Err(ModelError::ConductorInCable(cid(1)))
    );
    project.remove_cable(CableId::new(1).unwrap()).unwrap();
    assert_eq!(project, before);
}

#[test]
fn shared_or_missing_core_rejects_entire_assembly_without_leaking_membership() {
    let mut project = fixture();
    project
        .insert_conductor(Conductor::new(cid(2), None, None))
        .unwrap();
    let before = project.clone();
    assert_eq!(
        project.insert_cable(cable(1, &[1, 999])),
        Err(ModelError::MissingConductor(cid(999)))
    );
    assert_eq!(project, before);
    project.insert_cable(cable(1, &[1])).unwrap();
    let grouped = project.clone();
    assert_eq!(
        project.insert_cable(cable(2, &[2, 1])),
        Err(ModelError::ConductorInCable(cid(1)))
    );
    assert_eq!(project, grouped);
    assert_eq!(project.conductor_cable(cid(2)), None);
}

#[test]
fn detailed_edit_cannot_disagree_with_its_cable_overview_endpoint() {
    let mut project = fixture();
    project.insert_cable(cable(1, &[1])).unwrap();
    let before = project.clone();
    assert!(matches!(
        project.set_termination(cid(1), End::A, Some(tid(3))),
        Err(ModelError::CableEndpointMismatch(_, _, End::A))
    ));
    assert_eq!(project, before);
    project.set_termination(cid(1), End::B, None).unwrap();
    project
        .set_termination(cid(1), End::B, Some(tid(2)))
        .unwrap();
    assert_eq!(project, before);
    assert!(
        project
            .remove_port(InterfacePortId::new(1).unwrap())
            .is_err()
    );
    assert_eq!(project, before);
}

#[test]
fn empty_and_spare_core_drafts_are_valid_but_endpoints_must_exist() {
    let mut project = fixture();
    project
        .insert_conductor(Conductor::new(cid(2), None, None))
        .unwrap();
    project.insert_cable(cable(1, &[2])).unwrap();
    project.insert_cable(cable(2, &[])).unwrap();
    project.validate().unwrap();
    let before = project.clone();
    let bad = Cable::new(
        CableId::new(3).unwrap(),
        "-C3",
        CableEndpoint::Device(DeviceId::new(999).unwrap()),
        CableEndpoint::Device(DeviceId::new(1).unwrap()),
        [],
    )
    .unwrap();
    assert!(matches!(
        project.insert_cable(bad),
        Err(ModelError::MissingDevice(_))
    ));
    assert_eq!(project, before);
}

#[test]
fn duplicate_core_list_is_an_error_rather_than_silent_deduplication() {
    let endpoints = CableEndpoint::Device(DeviceId::new(1).unwrap());
    assert_eq!(
        Cable::new(
            CableId::new(1).unwrap(),
            "-C1",
            endpoints,
            endpoints,
            [cid(1), cid(1)]
        ),
        Err(ModelError::DuplicateCableMember(cid(1)))
    );
}
