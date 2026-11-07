//! Electrical projection and deterministic output contracts independent of drawing representations.
use ecad_adapters::{
    ConnectionCsvKind, JsonFile, decode_project, encode_connection_csv,
    encode_current_connection_csv, encode_project,
};
use ecad_app::*;
use ecad_domain::*;
#[path = "reports/actions.rs"]
mod actions;
#[path = "reports/persistence.rs"]
mod persistence;
#[path = "reports/support.rs"]
mod support;
use support::*;

#[test]
fn report_retains_actual_hierarchy_all_terminal_links_and_cable_overview_vs_core_identity() {
    let mut session = Session::new(project());
    let before = session.project().clone();
    let revision = session.revision();
    let report = ConnectionReport::from_session(&session);
    assert!(report.is_current(&session));
    assert_eq!(report.project_id(), before.id());
    assert_eq!(report.devices().len(), 3);
    assert_eq!(report.terminals().len(), 3);
    assert_eq!(report.conductors().len(), 2);
    assert_eq!(report.cables().len(), 1);
    let device = report.device(DeviceId::new(2).unwrap()).unwrap();
    assert_eq!(device.designation, "-Junction");
    assert_eq!(device.terminals, vec![TerminalId::new(201).unwrap()]);
    let terminal = report.terminal(TerminalId::new(201).unwrap()).unwrap();
    assert_eq!(
        terminal.connections,
        vec![
            ConductorEnd {
                conductor: ConductorId::new(1).unwrap(),
                end: End::B
            },
            ConductorEnd {
                conductor: ConductorId::new(2).unwrap(),
                end: End::A
            }
        ]
    );
    let path = report
        .subsystem_path(SubsystemId::new(20).unwrap())
        .unwrap();
    assert_eq!(
        path.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        vec!["Conveyor", "Sensor assembly"]
    );
    assert!(matches!(
        report.subsystem_path(SubsystemId::new(999).unwrap()),
        Err(ModelError::MissingSubsystem(_))
    ));
    let cable = report.cable(CableId::new(1).unwrap()).unwrap();
    assert_eq!(
        cable.endpoints,
        [
            CableEndpoint::Device(DeviceId::new(1).unwrap()),
            CableEndpoint::Subsystem(SubsystemId::new(10).unwrap())
        ]
    );
    let wire = report.conductor(cable.conductors[0]).unwrap();
    assert_eq!(wire.endpoints[1], Some(TerminalId::new(201).unwrap()));
    assert_eq!(wire.cable, Some(cable.id));
    assert_eq!(session.project(), &before);
    assert_eq!(session.revision(), revision);
    assert_eq!(session.undo_len(), 0);
    // Editing a caller-owned record clone cannot alter the immutable report.
    let mut copy = terminal.clone();
    copy.connections.clear();
    assert_eq!(report.terminal(copy.id).unwrap().connections.len(), 2);
    assert!(!session.undo(revision).unwrap());
}

#[test]
fn every_free_end_unconnected_terminal_self_loop_and_empty_cable_survives_projection_and_export() {
    let mut project = project();
    let device = DeviceId::new(4).unwrap();
    let terminal = TerminalId::new(401).unwrap();
    let unused = TerminalId::new(402).unwrap();
    project
        .insert_device(
            Device::new(
                device,
                "-Draft",
                [
                    Terminal::new(terminal, "A").unwrap(),
                    Terminal::new(unused, "B").unwrap(),
                ],
            )
            .unwrap(),
        )
        .unwrap();
    for (wire, a, b) in [
        (3, None, None),
        (4, Some(terminal), None),
        (5, None, Some(terminal)),
        (6, Some(terminal), Some(terminal)),
    ] {
        project
            .insert_conductor(Conductor::new(ConductorId::new(wire).unwrap(), a, b))
            .unwrap();
    }
    project
        .insert_cable(
            Cable::new(
                CableId::new(2).unwrap(),
                "-Empty",
                CableEndpoint::Device(device),
                CableEndpoint::Device(device),
                [],
            )
            .unwrap(),
        )
        .unwrap();
    project
        .insert_cable(
            Cable::new(
                CableId::new(3).unwrap(),
                "-Free core",
                CableEndpoint::Device(device),
                CableEndpoint::Device(device),
                [ConductorId::new(3).unwrap()],
            )
            .unwrap(),
        )
        .unwrap();
    let session = Session::new(project);
    let report = ConnectionReport::from_session(&session);
    assert_eq!(report.terminal(unused).unwrap().connections.len(), 0);
    let attached = &report.terminal(terminal).unwrap().connections;
    assert_eq!(attached.len(), 4);
    assert_eq!(
        attached[2..],
        [
            ConductorEnd {
                conductor: ConductorId::new(6).unwrap(),
                end: End::A
            },
            ConductorEnd {
                conductor: ConductorId::new(6).unwrap(),
                end: End::B
            }
        ]
    );
    for (wire, state) in [
        (3, ConnectionState::BothFree),
        (4, ConnectionState::BFree),
        (5, ConnectionState::AFree),
        (6, ConnectionState::BothAttached),
    ] {
        assert_eq!(
            report
                .conductor(ConductorId::new(wire).unwrap())
                .unwrap()
                .state(),
            state
        );
    }
    let wire_rows = parse_csv(&encode_connection_csv(&report, ConnectionCsvKind::WireList));
    assert_eq!(wire_rows.len(), 7);
    let state = column(&wire_rows, "connection_state");
    assert_eq!(wire_rows[3][state], "both_free");
    assert_eq!(wire_rows[4][state], "b_free");
    assert_eq!(wire_rows[5][state], "a_free");
    let rows = parse_csv(&encode_connection_csv(
        &report,
        ConnectionCsvKind::TerminalTable,
    ));
    let state = column(&rows, "connection_state");
    let term = column(&rows, "terminal_id");
    let wire = column(&rows, "conductor_id");
    let end = column(&rows, "local_end");
    assert_eq!(
        rows.iter()
            .skip(1)
            .filter(|r| r[state] == "unconnected")
            .count(),
        1
    );
    assert_eq!(
        rows.iter()
            .skip(1)
            .filter(|r| r[state] == "destination_free")
            .count(),
        2
    );
    let loops: Vec<_> = rows
        .iter()
        .skip(1)
        .filter(|r| r[term] == id(401) && r[wire] == id(6))
        .map(|r| r[end].as_str())
        .collect();
    assert_eq!(loops, vec!["A", "B"]);
    let rows = parse_csv(&encode_connection_csv(
        &report,
        ConnectionCsvKind::CableSchedule,
    ));
    let state = column(&rows, "core_state");
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[2][state], "empty_cable");
    assert_eq!(rows[3][state], "both_free");
}

#[test]
fn renames_assignments_and_undo_rebuild_values_by_stable_identity_without_mutating_old_snapshots() {
    let mut session = Session::new(project());
    let original = ConnectionReport::from_session(&session);
    let before_csv = report_csv(&session);
    set(
        &mut session,
        ObjectRef::Device(DeviceId::new(2).unwrap()),
        PropertyId::Designation,
        "-New Junction",
    );
    set(
        &mut session,
        ObjectRef::Terminal(TerminalId::new(201).unwrap()),
        PropertyId::TerminalNumber,
        "7",
    );
    set(
        &mut session,
        ObjectRef::Subsystem(SubsystemId::new(10).unwrap()),
        PropertyId::Name,
        "Renamed conveyor",
    );
    session
        .execute(
            session.revision(),
            Command::SetTermination {
                conductor: ConductorId::new(2).unwrap(),
                end: End::B,
                terminal: None,
            },
        )
        .unwrap();
    assert!(!original.is_current(&session));
    let current = ConnectionReport::from_session(&session);
    assert_eq!(
        current
            .device(DeviceId::new(2).unwrap())
            .unwrap()
            .designation,
        "-New Junction"
    );
    assert_eq!(
        current
            .terminal(TerminalId::new(201).unwrap())
            .unwrap()
            .number,
        "7"
    );
    assert_eq!(
        current
            .subsystem_path(SubsystemId::new(20).unwrap())
            .unwrap()[0]
            .name,
        "Renamed conveyor"
    );
    assert_eq!(
        current
            .conductor(ConductorId::new(2).unwrap())
            .unwrap()
            .state(),
        ConnectionState::BFree
    );
    assert!(
        current
            .terminal(TerminalId::new(301).unwrap())
            .unwrap()
            .connections
            .is_empty()
    );
    assert_eq!(
        original
            .device(DeviceId::new(2).unwrap())
            .unwrap()
            .designation,
        "-Junction"
    );
    assert_eq!(
        original
            .terminal(TerminalId::new(201).unwrap())
            .unwrap()
            .number,
        "1"
    );
    for kind in kind_list() {
        assert!(matches!(
            encode_current_connection_csv(&session, &original, kind),
            Err(EditError::StaleRevision { .. })
        ));
    }
    for _ in 0..4 {
        session.undo(session.revision()).unwrap();
    }
    assert!(!original.is_current(&session));
    assert_eq!(report_csv(&session), before_csv);
}

#[test]
fn drawing_representations_and_edits_never_duplicate_or_reassign_electrical_report_rows() {
    let base = Session::new(project());
    let mut drawn =
        Session::new(decode_project(include_str!("fixtures/routed-conveyor-v6.json")).unwrap());
    assert_eq!(report_csv(&base), report_csv(&drawn));
    let before = ConnectionReport::from_session(&drawn);
    drawn
        .execute(
            drawn.revision(),
            Command::Route(RouteCommand::MoveAnchor {
                anchor: AnchorId::new(2).unwrap(),
                position: DrawingPoint { x: 10000, y: 5000 },
            }),
        )
        .unwrap();
    assert!(!before.is_current(&drawn));
    assert_eq!(report_csv(&base), report_csv(&drawn));
}

#[test]
fn empty_projects_produce_header_only_reports_and_different_session_epochs_are_never_current() {
    let session = Session::new(Project::new(ProjectId::new(1).unwrap()));
    let report = ConnectionReport::from_session(&session);
    for kind in kind_list() {
        let text = encode_current_connection_csv(&session, &report, kind).unwrap();
        assert_eq!(parse_csv(&text).len(), 1);
        assert!(text.ends_with('\n'));
    }
    assert!(report.device(DeviceId::new(999).unwrap()).is_none());
    let other = Session::new(session.project().clone());
    assert!(!report.is_current(&other));
    assert!(encode_current_connection_csv(&other, &report, ConnectionCsvKind::WireList).is_err());
}
