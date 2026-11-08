//! Bundled immutable definitions, persisted pins, and malformed mapping rejection.

use ecad_adapters::{JsonFile, RandomIds, decode_project, encode_project};
use ecad_app::{Command, Document, instance_commands};
use ecad_domain::*;
use serde_json::Value;
use std::sync::Arc;

fn source(revision: u128) -> DefinitionRef {
    DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(revision).unwrap(),
    )
}
fn revision(number: u128) -> Arc<SubsystemRevision> {
    let mut body = decode_project(include_str!("fixtures/sensor-module-v3.json")).unwrap();
    if number > 1 {
        body.set_termination(ConductorId::new(2).unwrap(), End::B, None)
            .unwrap();
    }
    Arc::new(SubsystemRevision::new(source(number), SubsystemId::new(10).unwrap(), body).unwrap())
}
fn place(document: &mut Document<JsonFile>, name: &str) -> InstanceBinding {
    let ids = RandomIds::instance_ids(
        document.session().project().definition(source(1)).unwrap(),
        InstanceLimits::default(),
    )
    .unwrap();
    let root = ids.subsystems[&SubsystemId::new(10).unwrap()];
    let commands = instance_commands(
        document.session().project(),
        source(1),
        ids,
        None,
        name,
        InstanceLimits::default(),
    )
    .unwrap();
    document
        .execute_batch(document.session().revision(), commands)
        .unwrap();
    document.session().project().instance(root).unwrap().clone()
}
fn populate(path: &std::path::Path) -> (Document<JsonFile>, InstanceBinding, InstanceBinding) {
    let mut document = Document::new(
        Project::new(RandomIds::project().unwrap()),
        JsonFile::new(path),
    );
    document
        .execute(
            document.session().revision(),
            Command::InsertDefinition(revision(1)),
        )
        .unwrap();
    let first = place(&mut document, "Left");
    let second = place(&mut document, "Right");
    (document, first, second)
}

#[test]
fn save_reopen_bundles_one_source_and_two_complete_independent_mappings() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("repeated.json");
    let (mut document, first, second) = populate(&path);
    assert_ne!(first.ids().terminals, second.ids().terminals);
    document.save().unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let data: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(data["definitions"].as_array().unwrap().len(), 1);
    assert_eq!(data["instances"].as_array().unwrap().len(), 2);
    let mut reopened = Document::open(JsonFile::new(&path)).unwrap();
    assert_eq!(reopened.session().project(), document.session().project());
    let wire = *first.ids().conductors.values().next().unwrap();
    assert!(
        reopened
            .execute(
                reopened.session().revision(),
                Command::SetTermination {
                    conductor: wire,
                    end: End::B,
                    terminal: None
                }
            )
            .is_err()
    );
    reopened.save().unwrap();
    assert_eq!(std::fs::read_to_string(path).unwrap(), text);
}

#[test]
fn source_array_order_and_mapping_array_order_do_not_change_canonical_output() {
    let directory = tempfile::tempdir().unwrap();
    let (document, _, _) = populate(&directory.path().join("project.json"));
    let canonical = encode_project(document.session().project()).unwrap();
    let mut data: Value = serde_json::from_str(&canonical).unwrap();
    for field in [
        "subsystems",
        "devices",
        "ports",
        "conductors",
        "cables",
        "definitions",
        "instances",
    ] {
        data[field].as_array_mut().unwrap().reverse();
    }
    for instance in data["instances"].as_array_mut().unwrap() {
        for field in [
            "subsystems",
            "devices",
            "terminals",
            "ports",
            "conductors",
            "cables",
        ] {
            instance["ids"][field].as_array_mut().unwrap().reverse();
        }
    }
    assert_eq!(
        encode_project(&decode_project(&data.to_string()).unwrap()).unwrap(),
        canonical
    );
}

#[test]
fn duplicate_missing_and_aliased_identity_mappings_never_load_partially() {
    let directory = tempfile::tempdir().unwrap();
    let (document, _, _) = populate(&directory.path().join("project.json"));
    let original: Value =
        serde_json::from_str(&encode_project(document.session().project()).unwrap()).unwrap();
    let mut duplicate = original.clone();
    let pair = duplicate["instances"][0]["ids"]["terminals"][0].clone();
    duplicate["instances"][0]["ids"]["terminals"]
        .as_array_mut()
        .unwrap()
        .push(pair);
    assert!(decode_project(&duplicate.to_string()).is_err());
    let mut missing = original.clone();
    missing["instances"][0]["ids"]["terminals"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    assert!(decode_project(&missing.to_string()).is_err());
    let mut alias = original;
    alias["instances"][0]["ids"]["terminals"][0]["placed"] =
        alias["instances"][0]["ids"]["terminals"][1]["placed"].clone();
    assert!(decode_project(&alias.to_string()).is_err());
}

#[test]
fn altered_materialized_content_unknown_sources_and_duplicate_catalogs_are_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let (document, _, _) = populate(&directory.path().join("project.json"));
    let original: Value =
        serde_json::from_str(&encode_project(document.session().project()).unwrap()).unwrap();
    let mut altered = original.clone();
    altered["conductors"][0]["b"] = Value::Null;
    assert!(decode_project(&altered.to_string()).is_err());
    let mut unknown = original.clone();
    unknown["instances"][0]["revision"] =
        Value::String(DefinitionRevisionId::new(99).unwrap().to_string());
    assert!(decode_project(&unknown.to_string()).is_err());
    let mut duplicate = original.clone();
    duplicate["definitions"]
        .as_array_mut()
        .unwrap()
        .push(original["definitions"][0].clone());
    assert!(decode_project(&duplicate.to_string()).is_err());
    let mut invalid_body = original;
    invalid_body["definitions"][0]["body"]["schema_version"] = Value::from(99);
    assert!(decode_project(&invalid_body.to_string()).is_err());
}

#[test]
fn make_local_survives_reopen_and_does_not_unlock_the_other_instance() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let (mut document, first, second) = populate(&path);
    let first_wire = *first.ids().conductors.values().next().unwrap();
    let second_wire = *second.ids().conductors.values().next().unwrap();
    document
        .execute_batch(
            document.session().revision(),
            vec![
                Command::DetachInstance(first.root()),
                Command::SetTermination {
                    conductor: first_wire,
                    end: End::B,
                    terminal: None,
                },
            ],
        )
        .unwrap();
    document.save().unwrap();
    let mut reopened = Document::open(JsonFile::new(path)).unwrap();
    assert!(
        reopened
            .session()
            .project()
            .instance(first.root())
            .is_none()
    );
    assert_eq!(
        reopened
            .session()
            .project()
            .local_origin(first.root())
            .unwrap()
            .source(),
        first.source()
    );
    assert!(
        reopened
            .session()
            .project()
            .instance(second.root())
            .is_some()
    );
    reopened
        .execute(
            reopened.session().revision(),
            Command::SetTermination {
                conductor: first_wire,
                end: End::A,
                terminal: None,
            },
        )
        .unwrap();
    assert!(
        reopened
            .execute(
                reopened.session().revision(),
                Command::SetTermination {
                    conductor: second_wire,
                    end: End::B,
                    terminal: None
                }
            )
            .is_err()
    );
}

#[test]
fn same_source_in_two_projects_remains_pinned_when_one_imports_a_new_revision() {
    let directory = tempfile::tempdir().unwrap();
    let (mut first, _, right) = populate(&directory.path().join("first.json"));
    let (second, _, _) = populate(&directory.path().join("second.json"));
    let before_second = second.session().project().clone();
    first
        .execute(
            first.session().revision(),
            Command::InsertDefinition(revision(2)),
        )
        .unwrap();
    assert_eq!(first.session().project().definition_count(), 2);
    assert_eq!(
        first
            .session()
            .project()
            .instance(right.root())
            .unwrap()
            .source(),
        source(1)
    );
    assert_eq!(second.session().project(), &before_second);
    assert_eq!(second.session().project().definition_count(), 1);
}

#[test]
fn v3_upgrades_without_inventing_definitions_or_changing_object_identity() {
    let legacy = decode_project(include_str!("fixtures/minimal-v3.json")).unwrap();
    assert_eq!(legacy.definition_count(), 0);
    assert_eq!(legacy.instances().len(), 0);
    assert_eq!(
        encode_project(&legacy).unwrap(),
        include_str!("fixtures/minimal-v12.json")
    );
}

#[test]
fn local_origin_metadata_rejects_duplicate_active_or_unavailable_source_claims() {
    let directory = tempfile::tempdir().unwrap();
    let (mut document, first, second) = populate(&directory.path().join("project.json"));
    document
        .execute(
            document.session().revision(),
            Command::DetachInstance(first.root()),
        )
        .unwrap();
    let original: Value =
        serde_json::from_str(&encode_project(document.session().project()).unwrap()).unwrap();
    assert_eq!(original["local_origins"].as_array().unwrap().len(), 1);
    let mut duplicate = original.clone();
    duplicate["local_origins"]
        .as_array_mut()
        .unwrap()
        .push(original["local_origins"][0].clone());
    assert!(decode_project(&duplicate.to_string()).is_err());
    let mut active = original.clone();
    active["local_origins"][0]["root"] = Value::String(second.root().to_string());
    assert!(decode_project(&active.to_string()).is_err());
    let mut unknown = original;
    unknown["local_origins"][0]["revision"] =
        Value::String(DefinitionRevisionId::new(99).unwrap().to_string());
    assert!(decode_project(&unknown.to_string()).is_err());
}

#[test]
fn production_mapping_generation_checks_the_budget_before_allocating() {
    let limit = InstanceLimits {
        max_objects: std::num::NonZeroUsize::new(1).unwrap(),
    };
    assert!(matches!(
        RandomIds::instance_ids(&revision(1), limit),
        Err(ecad_adapters::InstanceIdError::ExpansionLimit)
    ));
}
