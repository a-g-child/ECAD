//! Versioned persistence fixtures and session-to-file integration behavior.

use ecad_adapters::{JsonFile, ProjectFileError, decode_project, encode_project};
use ecad_app::{Command, ProjectStore, Session};
use ecad_domain::{ConductorId, DeviceId, End, ModelError, Project, TerminalId};

const FIXTURE: &str = include_str!("fixtures/minimal-v10.json");

#[test]
fn golden_fixture_has_known_electrical_meaning_and_canonical_encoding() {
    let project = decode_project(FIXTURE).unwrap();
    assert_eq!(project.devices().count(), 2);
    assert_eq!(project.conductors().count(), 2);
    let wire = project.conductor(ConductorId::new(1001).unwrap()).unwrap();
    assert_eq!(wire.endpoint(End::A), Some(TerminalId::new(101).unwrap()));
    assert_eq!(wire.endpoint(End::B), Some(TerminalId::new(201).unwrap()));
    assert_eq!(
        project
            .device(DeviceId::new(2).unwrap())
            .unwrap()
            .designation(),
        "-X2"
    );
    assert_eq!(encode_project(&project).unwrap(), FIXTURE);
}

#[test]
fn insertion_order_does_not_change_saved_text() {
    let expected = decode_project(FIXTURE).unwrap();
    let mut reversed = Project::new(ecad_domain::ProjectId::new(1).unwrap());
    let devices: Vec<_> = expected.devices().cloned().collect();
    let conductors: Vec<_> = expected.conductors().cloned().collect();
    for device in devices.into_iter().rev() {
        reversed.insert_device(device).unwrap();
    }
    for conductor in conductors.into_iter().rev() {
        reversed.insert_conductor(conductor).unwrap();
    }
    assert_eq!(encode_project(&reversed).unwrap(), FIXTURE);
}

#[test]
fn schema_and_unknown_data_cannot_be_silently_downgraded() {
    assert!(matches!(
        decode_project(&FIXTURE.replace("\"schema_version\": 10", "\"schema_version\": 11")),
        Err(ProjectFileError::UnsupportedVersion(11))
    ));
    for invalid in [
        FIXTURE.replace(
            "\"schema_version\": 10,",
            "\"schema_version\": 10, \"future_feature\": true,",
        ),
        FIXTURE.replace(
            "\"number\": \"1\"",
            "\"number\": \"1\", \"future_terminal\": 1",
        ),
        FIXTURE[..FIXTURE.len() / 2].to_owned(),
    ] {
        assert!(matches!(
            decode_project(&invalid),
            Err(ProjectFileError::Json(_))
        ));
    }
}

#[test]
fn dangling_references_and_duplicate_ids_reject_the_whole_load() {
    let missing = FIXTURE.replace(
        "\"b\": \"000000000000000000000000000000c9\"",
        "\"b\": \"000000000000000000000000000003e7\"",
    );
    assert!(matches!(
        decode_project(&missing),
        Err(ProjectFileError::Model(ModelError::MissingTerminal(_)))
    ));
    let duplicate = FIXTURE.replace(
        "\"id\": \"00000000000000000000000000000002\"",
        "\"id\": \"00000000000000000000000000000001\"",
    );
    assert!(matches!(
        decode_project(&duplicate),
        Err(ProjectFileError::Model(ModelError::DuplicateDevice(_)))
    ));
}

#[test]
fn command_save_reopen_and_undo_preserve_complete_document() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("project with ünicode.json"));
    let original = decode_project(FIXTURE).unwrap();
    let mut session = Session::new(original.clone());
    session
        .execute(
            session.revision(),
            Command::SetTermination {
                conductor: ConductorId::new(1001).unwrap(),
                end: End::B,
                terminal: None,
            },
        )
        .unwrap();
    store.save(session.project()).unwrap();
    assert_eq!(store.load().unwrap(), *session.project());
    let first_save = std::fs::read(store.path()).unwrap();
    store.save(session.project()).unwrap();
    assert_eq!(std::fs::read(store.path()).unwrap(), first_save);
    session.undo(session.revision()).unwrap();
    store.save(session.project()).unwrap();
    assert_eq!(store.load().unwrap(), original);
    assert_eq!(std::fs::read_to_string(store.path()).unwrap(), FIXTURE);
}

#[test]
fn failed_replacement_leaves_destination_and_no_stray_temporary_files() {
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("not-a-file");
    std::fs::create_dir(&destination).unwrap();
    let marker = destination.join("keep.txt");
    std::fs::write(&marker, "existing content").unwrap();
    let store = JsonFile::new(&destination);
    assert!(matches!(
        store.save(&decode_project(FIXTURE).unwrap()),
        Err(ProjectFileError::Io(_))
    ));
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "existing content");
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn missing_files_are_io_errors_and_loaded_sessions_start_without_history() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("missing.json"));
    assert!(matches!(store.load(), Err(ProjectFileError::Io(_))));
    store.save(&decode_project(FIXTURE).unwrap()).unwrap();
    let session = Session::new(store.load().unwrap());
    assert_eq!(session.undo_len(), 0);
    assert_eq!(session.redo_len(), 0);
}
