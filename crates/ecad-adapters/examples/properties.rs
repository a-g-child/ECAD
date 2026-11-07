//! Inspect heterogeneous objects and edit applicable subsets in one saved transaction.

use ecad_adapters::{JsonFile, decode_project};
use ecad_app::{Document, SelectionValue, inspect_properties, property_edit_plan};
use ecad_domain::*;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project = decode_project(include_str!(
        "../tests/fixtures/controller-conveyor-v4.json"
    ))?;
    let selection = [
        ObjectRef::Device(DeviceId::new(1)?),
        ObjectRef::Cable(CableId::new(1)?),
        ObjectRef::Subsystem(SubsystemId::new(10)?),
        ObjectRef::Port(InterfacePortId::new(1)?),
        ObjectRef::Terminal(TerminalId::new(101)?),
        ObjectRef::Conductor(ConductorId::new(1)?),
    ];
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("properties.json"));
    let mut document = Document::new(project, store.clone());
    document.save()?;
    let before = document.session().project().clone();
    let inspector = inspect_properties(&before, selection)?;
    let row = &inspector.properties[&PropertyId::Designation];
    println!(
        "Designation applies to {} of {} objects; value: {:?}",
        row.applicable.len(),
        inspector.objects.len(),
        row.value
    );
    let plan = property_edit_plan(
        &before,
        selection,
        BTreeMap::from([
            (
                PropertyId::Designation,
                PropertyValue::Text("-Updated".into()),
            ),
            (
                PropertyId::Name,
                PropertyValue::Text("Updated scope".into()),
            ),
            (PropertyId::TerminalNumber, PropertyValue::Text("A".into())),
        ]),
    )?;
    println!(
        "Review: {} changes; {} inapplicable object/field pairs",
        plan.changes.len(),
        plan.inapplicable.len()
    );
    document.execute_batch(document.session().revision(), plan.commands)?;
    let inspector = inspect_properties(document.session().project(), selection)?;
    if !matches!(
        inspector.properties[&PropertyId::Designation].value,
        SelectionValue::Uniform(_)
    ) {
        return Err("applicable designations did not become uniform".into());
    }
    document.save()?;
    if Document::open(store)?.session().project() != document.session().project() {
        return Err("property save/reopen changed content".into());
    }
    document.undo(document.session().revision())?;
    if document.session().project() != &before {
        return Err("bulk undo did not restore exact fields/topology".into());
    }
    println!(
        "One transaction changed several applicable subsets; save/reopen and exact undo verified."
    );
    Ok(())
}
