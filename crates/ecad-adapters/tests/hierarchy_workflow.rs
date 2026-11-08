//! Controller/conveyor electrical meaning, persistence, history, and malformed hierarchy.

use ecad_adapters::{JsonFile, ProjectFileError, decode_project, encode_project};
use ecad_app::{Command, Document};
use ecad_domain::{
    CableId, ConductorId, End, InterfacePort, InterfacePortId, ModelError, Project, SubsystemId,
    TerminalId,
};
use serde_json::Value;

const FIXTURE: &str = include_str!("fixtures/controller-conveyor-v12.json");

fn commands(project: &Project) -> Vec<Command> {
    let mut commands: Vec<_> = project
        .subsystems()
        .cloned()
        .map(Command::InsertSubsystem)
        .collect();
    commands.extend(project.devices().cloned().map(Command::InsertDevice));
    commands.extend(project.ports().cloned().map(Command::InsertPort));
    commands.extend(project.conductors().cloned().map(Command::InsertConductor));
    commands.extend(project.cables().cloned().map(Command::InsertCable));
    commands
}

#[test]
fn overview_and_nested_wiring_share_the_same_junction_terminal_and_conductors() {
    let project = decode_project(FIXTURE).unwrap();
    assert_eq!(project.subsystems().count(), 2);
    assert_eq!(project.devices().count(), 3);
    assert_eq!(project.conductors().count(), 2);
    assert_eq!(project.cables().count(), 1);
    let junction = project
        .port(InterfacePortId::new(1).unwrap())
        .unwrap()
        .terminal();
    assert_eq!(
        project
            .conductor(ConductorId::new(1).unwrap())
            .unwrap()
            .endpoint(End::B),
        Some(junction)
    );
    assert_eq!(
        project
            .conductor(ConductorId::new(2).unwrap())
            .unwrap()
            .endpoint(End::A),
        Some(junction)
    );
    assert_eq!(
        project.conductor_cable(ConductorId::new(1).unwrap()),
        Some(CableId::new(1).unwrap())
    );
    assert_eq!(project.conductor_cable(ConductorId::new(2).unwrap()), None);
    assert_eq!(encode_project(&project).unwrap(), FIXTURE);
}

#[test]
fn one_construction_transaction_undoes_redoes_and_reopens_with_all_identities_intact() {
    let expected = decode_project(FIXTURE).unwrap();
    let empty = Project::new(expected.id());
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("conveyor.json"));
    let mut document = Document::new(empty.clone(), store.clone());
    document
        .execute_batch(document.session().revision(), commands(&expected))
        .unwrap();
    assert_eq!(document.session().undo_len(), 1);
    assert_eq!(document.session().project(), &expected);
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &empty);
    document.redo(document.session().revision()).unwrap();
    document.save().unwrap();
    let reopened = Document::open(store).unwrap();
    assert_eq!(reopened.session().project(), &expected);
    assert_eq!(
        std::fs::read_to_string(directory.path().join("conveyor.json")).unwrap(),
        FIXTURE
    );
}

#[test]
fn dependent_removal_failure_restores_ports_cable_membership_and_history_exactly() {
    let directory = tempfile::tempdir().unwrap();
    let expected = decode_project(FIXTURE).unwrap();
    let mut document = Document::new(
        expected.clone(),
        JsonFile::new(directory.path().join("project.json")),
    );
    document.save().unwrap();
    let revision = document.session().revision();
    assert!(
        document
            .execute_batch(
                revision,
                vec![
                    Command::RemoveCable(CableId::new(1).unwrap()),
                    Command::RemoveConductor(ConductorId::new(1).unwrap()),
                    Command::RemovePort(InterfacePortId::new(1).unwrap()),
                    Command::RemoveSubsystem(SubsystemId::new(10).unwrap()),
                ]
            )
            .is_err()
    );
    assert_eq!(document.session().project(), &expected);
    assert_eq!(document.session().revision(), revision);
    assert_eq!(document.session().undo_len(), 0);
    assert!(!document.session().is_dirty());
}

#[test]
fn detailed_core_change_updates_the_existing_overview_assembly_and_undo_restores_it() {
    let directory = tempfile::tempdir().unwrap();
    let expected = decode_project(FIXTURE).unwrap();
    let mut document = Document::new(
        expected.clone(),
        JsonFile::new(directory.path().join("project.json")),
    );
    let terminal = TerminalId::new(301).unwrap();
    document
        .execute_batch(
            document.session().revision(),
            vec![
                Command::InsertPort(
                    InterfacePort::new(
                        InterfacePortId::new(3).unwrap(),
                        SubsystemId::new(10).unwrap(),
                        "Direct sensor",
                        terminal,
                    )
                    .unwrap(),
                ),
                Command::SetTermination {
                    conductor: ConductorId::new(1).unwrap(),
                    end: End::B,
                    terminal: Some(terminal),
                },
            ],
        )
        .unwrap();
    let model = document.session().project();
    assert_eq!(
        model
            .cable(CableId::new(1).unwrap())
            .unwrap()
            .conductors()
            .collect::<Vec<_>>(),
        vec![ConductorId::new(1).unwrap()]
    );
    assert_eq!(
        model
            .conductor(ConductorId::new(1).unwrap())
            .unwrap()
            .endpoint(End::B),
        Some(terminal)
    );
    assert_eq!(
        decode_project(&encode_project(model).unwrap()).unwrap(),
        *model
    );
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &expected);
}

#[test]
fn loader_resolves_child_first_arrays_and_preserves_deterministic_output() {
    let mut data: Value = serde_json::from_str(FIXTURE).unwrap();
    for name in ["subsystems", "devices", "ports", "conductors", "cables"] {
        data[name].as_array_mut().unwrap().reverse();
    }
    assert_eq!(
        encode_project(&decode_project(&data.to_string()).unwrap()).unwrap(),
        FIXTURE
    );
}

#[test]
fn cyclic_missing_and_unexposed_hierarchy_is_rejected_as_a_complete_load() {
    let original: Value = serde_json::from_str(FIXTURE).unwrap();
    let mut cyclic = original.clone();
    cyclic["subsystems"][0]["parent"] = cyclic["subsystems"][1]["id"].clone();
    assert!(matches!(
        decode_project(&cyclic.to_string()),
        Err(ProjectFileError::Model(ModelError::HierarchyCycle(_)))
    ));
    let mut missing = original.clone();
    missing["subsystems"][0]["parent"] = Value::String(SubsystemId::new(99).unwrap().to_string());
    assert!(matches!(
        decode_project(&missing.to_string()),
        Err(ProjectFileError::Model(ModelError::MissingSubsystem(_)))
    ));
    let mut unexposed = original.clone();
    unexposed["ports"].as_array_mut().unwrap().remove(1);
    assert!(matches!(
        decode_project(&unexposed.to_string()),
        Err(ProjectFileError::Model(ModelError::InterfaceRequired(_, _)))
    ));
    let mut wrong_core = original;
    wrong_core["cables"][0]["conductors"][0] =
        Value::String(ConductorId::new(2).unwrap().to_string());
    assert!(matches!(
        decode_project(&wrong_core.to_string()),
        Err(ProjectFileError::Model(ModelError::CableEndpointMismatch(
            _,
            _,
            _
        )))
    ));
}

#[test]
fn v2_upgrade_preserves_lineage_and_all_existing_electrical_objects() {
    let old = decode_project(include_str!("fixtures/minimal-v2.json")).unwrap();
    assert_eq!(
        encode_project(&old).unwrap(),
        include_str!("fixtures/minimal-v12.json")
    );
    assert_eq!(old.subsystems().count(), 0);
    assert_eq!(old.cables().count(), 0);
    let invalid_v2 = include_str!("fixtures/minimal-v2.json")
        .replace("\"devices\":", "\"subsystems\": [], \"devices\":");
    assert!(matches!(
        decode_project(&invalid_v2),
        Err(ProjectFileError::Json(_))
    ));
}

#[test]
fn containment_commands_undo_exactly_and_no_ops_preserve_the_saved_checkpoint() {
    let directory = tempfile::tempdir().unwrap();
    let original = decode_project(FIXTURE).unwrap();
    let mut document = Document::new(
        original.clone(),
        JsonFile::new(directory.path().join("project.json")),
    );
    document.save().unwrap();
    let revision = document.session().revision();
    assert!(
        !document
            .execute(
                revision,
                Command::SetSubsystemParent {
                    subsystem: SubsystemId::new(10).unwrap(),
                    parent: None
                }
            )
            .unwrap()
    );
    assert!(
        !document
            .execute(
                revision,
                Command::SetDeviceSubsystem {
                    device: ecad_domain::DeviceId::new(1).unwrap(),
                    subsystem: None
                }
            )
            .unwrap()
    );
    assert_eq!(document.session().revision(), revision);
    assert!(!document.session().is_dirty());
    document
        .execute(
            revision,
            Command::SetSubsystemParent {
                subsystem: SubsystemId::new(20).unwrap(),
                parent: None,
            },
        )
        .unwrap();
    document.session().project().validate().unwrap();
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &original);
    document
        .execute(
            document.session().revision(),
            Command::SetDeviceSubsystem {
                device: ecad_domain::DeviceId::new(1).unwrap(),
                subsystem: Some(SubsystemId::new(10).unwrap()),
            },
        )
        .unwrap();
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &original);
    assert!(!document.session().is_dirty());
}
