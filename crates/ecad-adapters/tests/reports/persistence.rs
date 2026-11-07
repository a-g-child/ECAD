//! Golden outputs, escaping, and deterministic persistence independent of session metadata.
use super::*;
#[test]
fn known_electrical_fixture_matches_independently_authored_wire_terminal_and_cable_goldens() {
    let session = Session::new(project());
    let report = ConnectionReport::from_session(&session);
    for (kind, expected) in [
        (
            ConnectionCsvKind::WireList,
            include_str!("../fixtures/conveyor-wires.csv"),
        ),
        (
            ConnectionCsvKind::TerminalTable,
            include_str!("../fixtures/conveyor-terminals.csv"),
        ),
        (
            ConnectionCsvKind::CableSchedule,
            include_str!("../fixtures/conveyor-cables.csv"),
        ),
    ] {
        let text = encode_connection_csv(&report, kind);
        assert_eq!(text, expected);
        parse_csv(&text);
    }
}
#[test]
fn unicode_quotes_commas_newlines_and_path_separators_roundtrip_as_data_in_every_output() {
    let mut session = Session::new(project());
    let device = "-J,\"quoted\" Ω\r\nline";
    let terminal = "7,\"α\"\nnext";
    let scope = "outer/inner,\"X\"\r\nΩ";
    let cable = "-C,\"1\"\nΩ";
    set(
        &mut session,
        ObjectRef::Device(DeviceId::new(2).unwrap()),
        PropertyId::Designation,
        device,
    );
    set(
        &mut session,
        ObjectRef::Terminal(TerminalId::new(201).unwrap()),
        PropertyId::TerminalNumber,
        terminal,
    );
    set(
        &mut session,
        ObjectRef::Subsystem(SubsystemId::new(10).unwrap()),
        PropertyId::Name,
        scope,
    );
    set(
        &mut session,
        ObjectRef::Cable(CableId::new(1).unwrap()),
        PropertyId::Designation,
        cable,
    );
    let report = ConnectionReport::from_session(&session);
    for kind in kind_list() {
        let text = encode_connection_csv(&report, kind);
        let rows = parse_csv(&text);
        assert!(rows.iter().skip(1).any(|r| r.iter().any(|c| c == device)));
        assert!(rows.iter().skip(1).any(|r| r.iter().any(|c| c == terminal)));
        assert!(rows.iter().skip(1).any(|r| r.iter().any(|c| c == cable)));
        let name_column = match kind {
            ConnectionCsvKind::WireList => "b_scope_names_json",
            ConnectionCsvKind::TerminalTable => "device_scope_names_json",
            ConnectionCsvKind::CableSchedule => "b_overview_scope_names_json",
        };
        let index = column(&rows, name_column);
        let parsed: Vec<Vec<String>> = rows
            .iter()
            .skip(1)
            .filter(|r| !r[index].is_empty())
            .map(|r| serde_json::from_str(&r[index]).unwrap())
            .collect();
        assert!(
            parsed
                .iter()
                .any(|names| names.first().is_some_and(|name| name == scope))
        );
    }
}
#[test]
fn saving_and_reopening_retains_identical_csv_without_persisting_report_revision_or_cache() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("report.json"));
    let mut document = Document::new(project(), store.clone());
    let before = encode_project(document.session().project()).unwrap();
    let original = ConnectionReport::from_session(document.session());
    let outputs = report_csv(document.session());
    document.save().unwrap();
    assert!(original.is_current(document.session()));
    assert_eq!(
        encode_project(document.session().project()).unwrap(),
        before
    );
    let reopened = Document::open(store).unwrap();
    assert!(!original.is_current(reopened.session()));
    assert_eq!(report_csv(reopened.session()), outputs);
    assert!(
        encode_current_connection_csv(
            reopened.session(),
            &original,
            ConnectionCsvKind::TerminalTable
        )
        .is_err()
    );
    assert_eq!(
        encode_connection_csv(&original, ConnectionCsvKind::WireList),
        outputs[0]
    );
    assert!(!document.session().is_dirty());
    assert_eq!(document.session().undo_len(), 0);
}

#[test]
fn immutable_source_bodies_do_not_count_as_placed_devices_and_independent_instances_do() {
    use std::sync::Arc;
    let body = decode_project(include_str!("../fixtures/sensor-module-v3.json")).unwrap();
    let source = DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(1).unwrap(),
    );
    let revision =
        Arc::new(SubsystemRevision::new(source, SubsystemId::new(10).unwrap(), body).unwrap());
    let mut project = Project::new(ProjectId::new(1).unwrap());
    project.insert_definition(Arc::clone(&revision)).unwrap();
    let mut session = Session::new(project);
    assert_eq!(ConnectionReport::from_session(&session).devices().len(), 0);
    for name in ["Left", "Right"] {
        let ids =
            ecad_adapters::RandomIds::instance_ids(&revision, InstanceLimits::default()).unwrap();
        let commands = instance_commands(
            session.project(),
            source,
            ids,
            None,
            name,
            InstanceLimits::default(),
        )
        .unwrap();
        session.execute_batch(session.revision(), commands).unwrap();
    }
    let report = ConnectionReport::from_session(&session);
    assert_eq!(report.devices().len(), 4);
    assert_eq!(report.terminals().len(), 4);
    assert_eq!(report.conductors().len(), 2);
    assert_eq!(report.cables().len(), 2);
    let scope_names: std::collections::BTreeSet<_> = report
        .devices()
        .map(|d| {
            report.subsystem_path(d.subsystem.unwrap()).unwrap()[0]
                .name
                .clone()
        })
        .collect();
    assert_eq!(
        scope_names,
        std::collections::BTreeSet::from(["Left".to_owned(), "Right".to_owned()])
    );
}
