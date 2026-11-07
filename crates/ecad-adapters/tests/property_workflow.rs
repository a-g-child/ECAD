//! Complete existing electrical-field inspection and coherent mixed-selection edits.

#[path = "properties/restrictions.rs"]
mod restrictions;
#[path = "properties/support.rs"]
mod support;
use ecad_adapters::{JsonFile, decode_project, encode_project};
use ecad_app::{
    Command, Document, SelectionValue, Session, inspect_properties, property_edit_plan,
};
use ecad_domain::*;
use std::collections::BTreeMap;
use support::*;

#[test]
fn union_inspection_reports_full_applicability_mixed_uniform_and_restrictions() {
    let project = fixture();
    let mut targets = selection();
    targets.push(targets[0]);
    let inspection = inspect_properties(&project, targets).unwrap();
    assert_eq!(inspection.objects.len(), 6);
    let designation = &inspection.properties[&PropertyId::Designation];
    assert_eq!(designation.applicable.len(), 2);
    assert_eq!(designation.inapplicable.len(), 4);
    assert_eq!(designation.value, SelectionValue::Mixed);
    assert_eq!(designation.editable.len(), 2);
    let terminal = &inspection.properties[&PropertyId::TerminalNumber];
    assert_eq!(
        terminal.value,
        SelectionValue::Uniform(PropertyValue::Text("1".into()))
    );
    let identity = &inspection.properties[&PropertyId::Identity];
    assert_eq!(identity.applicable.len(), 6);
    assert_eq!(identity.read_only.len(), 6);
    assert!(
        identity
            .read_only
            .values()
            .all(|reason| *reason == PropertyRestriction::Identity)
    );
    let members = &inspection.properties[&PropertyId::DeviceTerminals];
    assert!(
        members
            .read_only
            .values()
            .all(|reason| *reason == PropertyRestriction::StructuralOperation)
    );
    assert!(
        inspect_properties(&project, [])
            .unwrap()
            .properties
            .is_empty()
    );
}

#[test]
fn every_registered_field_reads_its_declared_type_on_every_supported_existing_object() {
    let project = fixture();
    let mut keys = std::collections::BTreeSet::new();
    for property in PROPERTY_IDS {
        assert!(keys.insert(property.descriptor().key));
        for target in selection() {
            let read = project.property(target, property);
            if property.applies_to(target) {
                assert!(property.descriptor().accepts_value(&read.unwrap().value));
            } else {
                assert!(matches!(
                    read,
                    Err(ModelError::Property(PropertyError::NotApplicable { .. }))
                ));
            }
        }
    }
}

#[test]
fn compounded_mixed_edits_skip_only_unsupported_targets_and_save_with_one_exact_undo() {
    let directory = tempfile::tempdir().unwrap();
    let store = JsonFile::new(directory.path().join("properties.json"));
    let mut document = Document::new(fixture(), store.clone());
    document.save().unwrap();
    let before = document.session().project().clone();
    let plan = property_edit_plan(
        &before,
        selection(),
        BTreeMap::from([
            (
                PropertyId::Designation,
                PropertyValue::Text("-Renamed".into()),
            ),
            (
                PropertyId::Name,
                PropertyValue::Text("Renamed scope".into()),
            ),
            (PropertyId::TerminalNumber, PropertyValue::Text("A".into())),
            (PropertyId::ConductorB, PropertyValue::OptionalObject(None)),
        ]),
    )
    .unwrap();
    assert_eq!(plan.changes.len(), 6);
    assert_eq!(plan.inapplicable.len(), 18);
    assert_eq!(document.session().project(), &before);
    let history = document.session().undo_len();
    document
        .execute_batch(document.session().revision(), plan.commands)
        .unwrap();
    assert_eq!(document.session().undo_len(), history + 1);
    let after = document.session().project().clone();
    assert_eq!(
        after
            .conductor(ConductorId::new(1).unwrap())
            .unwrap()
            .endpoint(End::A),
        Some(TerminalId::new(101).unwrap())
    );
    after.validate().unwrap();
    document.save().unwrap();
    assert_eq!(Document::open(store).unwrap().session().project(), &after);
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &before);
    assert!(document.session().is_dirty());
    document.redo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &after);
    assert!(!document.session().is_dirty());
}

#[test]
fn free_end_is_a_uniform_typed_absence_and_differs_from_mixed_attachments() {
    let mut project = fixture();
    for id in [10, 11] {
        project
            .insert_conductor(Conductor::new(ConductorId::new(id).unwrap(), None, None))
            .unwrap();
    }
    let free = [10, 11].map(|id| ObjectRef::Conductor(ConductorId::new(id).unwrap()));
    let inspection = inspect_properties(&project, free).unwrap();
    assert_eq!(
        inspection.properties[&PropertyId::ConductorA].value,
        SelectionValue::Uniform(PropertyValue::OptionalObject(None))
    );
    let inspection = inspect_properties(
        &project,
        [free[0], ObjectRef::Conductor(ConductorId::new(1).unwrap())],
    )
    .unwrap();
    assert_eq!(
        inspection.properties[&PropertyId::ConductorA].value,
        SelectionValue::Mixed
    );
}

#[test]
fn noops_and_all_unsupported_proposals_preserve_history_revision_and_redo() {
    let mut session = Session::new(fixture());
    session
        .execute(
            session.revision(),
            Command::InsertConductor(Conductor::new(ConductorId::new(10).unwrap(), None, None)),
        )
        .unwrap();
    session.undo(session.revision()).unwrap();
    let revision = session.revision();
    let redo = session.redo_len();
    let target = ObjectRef::Device(DeviceId::new(1).unwrap());
    let plan = property_edit_plan(
        session.project(),
        [target, target],
        BTreeMap::from([
            (
                PropertyId::Designation,
                PropertyValue::Text("-Controller".into()),
            ),
            (PropertyId::Name, PropertyValue::Text("Unused".into())),
            (PropertyId::Identity, PropertyValue::Object(target)),
        ]),
    )
    .unwrap();
    assert_eq!(plan.unchanged.len(), 2);
    assert_eq!(plan.inapplicable.len(), 1);
    assert!(!session.execute_batch(revision, plan.commands).unwrap());
    assert_eq!(session.revision(), revision);
    assert_eq!(session.redo_len(), redo);
    let project = session.project();
    assert_eq!(
        decode_project(&encode_project(project).unwrap()).unwrap(),
        *project
    );
}

#[test]
fn property_topology_changes_use_existing_boundary_and_cable_validation() {
    let mut project = fixture();
    let before = project.clone();
    let device = ObjectRef::Device(DeviceId::new(3).unwrap());
    assert!(
        project
            .set_property(
                device,
                PropertyId::DeviceSubsystem,
                PropertyValue::OptionalObject(None)
            )
            .is_err()
    );
    assert_eq!(project, before);
    let wire = ObjectRef::Conductor(ConductorId::new(1).unwrap());
    let hidden = ObjectRef::Terminal(TerminalId::new(301).unwrap());
    assert!(
        project
            .set_property(
                wire,
                PropertyId::ConductorB,
                PropertyValue::OptionalObject(Some(hidden))
            )
            .is_err()
    );
    assert_eq!(project, before);
    let child = ObjectRef::Subsystem(SubsystemId::new(10).unwrap());
    assert!(
        project
            .set_property(
                child,
                PropertyId::SubsystemParent,
                PropertyValue::OptionalObject(Some(ObjectRef::Subsystem(
                    SubsystemId::new(20).unwrap()
                )))
            )
            .is_err()
    );
    assert_eq!(project, before);
}
