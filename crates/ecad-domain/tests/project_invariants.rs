//! Structural validity, deliberate incompleteness, and rejection without mutation.

use ecad_domain::{
    Conductor, ConductorId, Device, DeviceId, End, ModelError, Project, Terminal, TerminalId,
};

fn terminal(value: u128) -> TerminalId {
    TerminalId::new(value).unwrap()
}
fn device(value: u128, terminals: &[u128]) -> Device {
    Device::new(
        DeviceId::new(value).unwrap(),
        format!("-X{value}"),
        terminals
            .iter()
            .map(|id| Terminal::new(terminal(*id), id.to_string()).unwrap()),
    )
    .unwrap()
}

#[test]
fn duplicate_terminal_rejects_entire_device_without_leaking_anchors() {
    let mut project = Project::new(ecad_domain::ProjectId::new(1).unwrap());
    project.insert_device(device(1, &[100])).unwrap();
    let original = project.clone();
    assert_eq!(
        project.insert_device(device(2, &[99, 100])),
        Err(ModelError::DuplicateTerminal(terminal(100)))
    );
    assert_eq!(project, original);
    assert_eq!(project.terminal_owner(terminal(99)), None);
}

#[test]
fn partially_assigned_conductors_are_valid_but_dangling_ends_are_not() {
    let mut project = Project::new(ecad_domain::ProjectId::new(1).unwrap());
    project.insert_device(device(1, &[100])).unwrap();
    let first = ConductorId::new(1).unwrap();
    project
        .insert_conductor(Conductor::new(first, Some(terminal(100)), None))
        .unwrap();
    project
        .insert_conductor(Conductor::new(ConductorId::new(2).unwrap(), None, None))
        .unwrap();
    project.validate().unwrap();
    let original = project.clone();
    assert_eq!(
        project.insert_conductor(Conductor::new(
            ConductorId::new(3).unwrap(),
            Some(terminal(100)),
            Some(terminal(999))
        )),
        Err(ModelError::MissingTerminal(terminal(999)))
    );
    assert_eq!(project, original);
    assert_eq!(
        project.set_termination(first, End::B, Some(terminal(999))),
        Err(ModelError::MissingTerminal(terminal(999)))
    );
    assert_eq!(project, original);
}

#[test]
fn connected_device_cannot_be_removed_and_disconnect_is_explicit() {
    let mut project = Project::new(ecad_domain::ProjectId::new(1).unwrap());
    project.insert_device(device(1, &[100])).unwrap();
    let id = ConductorId::new(1).unwrap();
    project
        .insert_conductor(Conductor::new(id, Some(terminal(100)), None))
        .unwrap();
    let original = project.clone();
    assert_eq!(
        project.remove_device(DeviceId::new(1).unwrap()),
        Err(ModelError::DeviceConnected(DeviceId::new(1).unwrap()))
    );
    assert_eq!(project, original);
    project.set_termination(id, End::A, None).unwrap();
    project.remove_device(DeviceId::new(1).unwrap()).unwrap();
    assert_eq!(project.terminal_owner(terminal(100)), None);
    assert!(project.conductor(id).is_some());
    project.validate().unwrap();
}

#[test]
fn duplicate_identity_never_replaces_an_existing_object() {
    let mut project = Project::new(ecad_domain::ProjectId::new(1).unwrap());
    project.insert_device(device(1, &[100])).unwrap();
    let original = project.clone();
    assert_eq!(
        project.insert_device(device(1, &[200])),
        Err(ModelError::DuplicateDevice(DeviceId::new(1).unwrap()))
    );
    assert_eq!(project, original);
    let id = ConductorId::new(1).unwrap();
    project
        .insert_conductor(Conductor::new(id, None, None))
        .unwrap();
    let original = project.clone();
    assert_eq!(
        project.insert_conductor(Conductor::new(id, Some(terminal(100)), None)),
        Err(ModelError::DuplicateConductor(id))
    );
    assert_eq!(project, original);
}

#[test]
fn device_numbers_are_unique_locally_not_across_devices() {
    let terminals = [
        Terminal::new(terminal(100), "1").unwrap(),
        Terminal::new(terminal(101), "1").unwrap(),
    ];
    assert_eq!(
        Device::new(DeviceId::new(1).unwrap(), "-X1", terminals),
        Err(ModelError::DuplicateTerminalNumber("1".into()))
    );
    let mut project = Project::new(ecad_domain::ProjectId::new(1).unwrap());
    for value in [1, 2] {
        project
            .insert_device(
                Device::new(
                    DeviceId::new(value).unwrap(),
                    format!("-X{value}"),
                    [Terminal::new(terminal(value), "1").unwrap()],
                )
                .unwrap(),
            )
            .unwrap();
    }
    project.validate().unwrap();
}

#[test]
fn blank_required_labels_are_rejected_without_silent_normalization() {
    assert_eq!(
        Terminal::new(terminal(1), " \t"),
        Err(ModelError::EmptyLabel)
    );
    assert_eq!(
        Device::new(DeviceId::new(1).unwrap(), "", []),
        Err(ModelError::EmptyLabel)
    );
    let label = "Power + return";
    let device = Device::new(DeviceId::new(1).unwrap(), label, []).unwrap();
    assert_eq!(device.designation(), label);
}
