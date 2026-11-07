//! Portable sheet-field foundation: exact history, canonical persistence and explicit boundaries.
use ecad_adapters::{JsonFile, decode_project, encode_project};
use ecad_app::{Command, Document};
use ecad_domain::*;
use std::collections::BTreeMap;
fn values() -> ProjectMetadata {
    ProjectMetadata::new(BTreeMap::from([
        (MetadataField::CompanyName, "Example & Co".into()),
        (
            MetadataField::CompanyAddress,
            " Unit 2\n\nExample Road\n".into(),
        ),
        (MetadataField::ProjectTitle, "Harness <prototype>".into()),
        (MetadataField::Issue, "B".into()),
    ]))
}
#[test]
fn metadata_is_literal_portable_and_one_exact_saved_history_edit() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let original = decode_project(include_str!("fixtures/routed-conveyor-v6.json")).unwrap();
    let mut document = Document::new(original.clone(), JsonFile::new(&path));
    document.save().unwrap();
    let before: serde_json::Value =
        serde_json::from_str(&encode_project(&original).unwrap()).unwrap();
    document
        .execute(
            document.session().revision(),
            Command::ProjectMetadata(values()),
        )
        .unwrap();
    document.save().unwrap();
    let loaded = Document::open(JsonFile::new(&path)).unwrap();
    assert_eq!(loaded.session().project(), document.session().project());
    assert_eq!(
        loaded
            .session()
            .project()
            .metadata()
            .value(MetadataField::CompanyAddress),
        Some(" Unit 2\n\nExample Road\n")
    );
    let mut after: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(after["schema_version"], 7);
    after["metadata"] = before["metadata"].clone();
    assert_eq!(after, before);
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &original);
    let revision = document.session().revision();
    document
        .execute(
            revision,
            Command::ProjectMetadata(ProjectMetadata::new(BTreeMap::from([(
                MetadataField::CompanyName,
                String::new(),
            )]))),
        )
        .unwrap();
    assert_eq!(document.session().revision(), revision);
    document.redo(revision).unwrap();
    assert_eq!(document.session().project().metadata(), &values());
    assert!(!document.session().is_dirty());
    assert_eq!(
        MetadataField::from_key("project.title"),
        Some(MetadataField::ProjectTitle)
    );
    assert_eq!(MetadataField::from_key("sheet.title"), None);
    assert_eq!(
        document
            .session()
            .project()
            .metadata()
            .value(MetadataField::Client),
        None
    );
}
#[test]
fn legacy_files_upgrade_without_inventing_values_and_v7_rejects_unknown_duplicate_or_malformed_fields()
 {
    for text in [
        include_str!("fixtures/minimal-v2.json"),
        include_str!("fixtures/controller-conveyor-v3.json"),
        include_str!("fixtures/minimal-v4.json"),
        include_str!("fixtures/minimal-v5.json"),
        include_str!("fixtures/routed-conveyor-v6.json"),
    ] {
        let project = decode_project(text).unwrap();
        assert_eq!(project.metadata(), &ProjectMetadata::default());
        assert_eq!(
            decode_project(&encode_project(&project).unwrap()).unwrap(),
            project
        );
    }
    let mut project = Project::new(ProjectId::new(1).unwrap());
    project.replace_metadata(values());
    let text = encode_project(&project).unwrap();
    assert_eq!(
        encode_project(&decode_project(&text).unwrap()).unwrap(),
        text
    );
    for invalid in [
        text.replace("\"project.title\":", "\"project.unknown\":"),
        text.replace(
            "\"metadata\": {",
            "\"metadata\": {\"company.name\": \"duplicate\",",
        ),
        text.replace("\"project.issue\": \"B\"", "\"project.issue\": 2"),
        text.replace(
            "\"metadata\": {",
            "\"metadata\": {\"sheet.title\": \"wrong scope\",",
        ),
    ] {
        assert!(decode_project(&invalid).is_err());
    }
    let old = include_str!("fixtures/routed-conveyor-v6.json");
    assert!(
        decode_project(&old.replace("\"drawing\": {", "\"metadata\": {}, \"drawing\": {")).is_err()
    );
}
#[test]
fn failed_batch_restores_metadata_and_electrical_subsystem_publication_cannot_drop_it() {
    let mut body = Project::new(ProjectId::new(1).unwrap());
    let root = SubsystemId::new(1).unwrap();
    body.insert_subsystem(Subsystem::new(root, "Assembly", None).unwrap())
        .unwrap();
    let mut session = ecad_app::Session::new(body.clone());
    assert!(
        session
            .execute_batch(
                session.revision(),
                vec![
                    Command::ProjectMetadata(values()),
                    Command::RemoveDevice(DeviceId::new(999).unwrap())
                ]
            )
            .is_err()
    );
    assert_eq!(session.project(), &body);
    body.replace_metadata(values());
    let source = DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(1).unwrap(),
    );
    assert!(matches!(
        SubsystemRevision::new(source, root, body),
        Err(ModelError::Reuse(ReuseError::ProjectMetadataUnsupported))
    ));
}
