//! Bounded mixed-selection inspection/preflight/commit/undo measurements.

use ecad_adapters::{decode_project, encode_project};
use ecad_app::{Session, inspect_properties, property_edit_plan};
use ecad_domain::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, error::Error, time::Instant};

pub(super) fn cases() -> Result<Vec<Value>, Box<dyn Error>> {
    [10, 100, 1_000].into_iter().map(probe).collect()
}
fn probe(count: u128) -> Result<Value, Box<dyn Error>> {
    let mut project = Project::new(ProjectId::new(1)?);
    let mut selection = Vec::new();
    for id in 1..=count {
        let device = DeviceId::new(id)?;
        let wire = ConductorId::new(id)?;
        project.insert_device(Device::new(
            device,
            format!("-X{id}"),
            [Terminal::new(TerminalId::new(id)?, "1")?],
        )?)?;
        project.insert_conductor(Conductor::new(wire, None, None))?;
        selection.extend([ObjectRef::Device(device), ObjectRef::Conductor(wire)]);
    }
    let start = Instant::now();
    let inspection = inspect_properties(&project, selection.iter().copied())?;
    let inspection_ms = super::millis(start.elapsed());
    if inspection.properties[&PropertyId::Designation]
        .applicable
        .len()
        != usize::try_from(count)?
    {
        return Err("inspection lost applicable targets".into());
    }
    let start = Instant::now();
    let plan = property_edit_plan(
        &project,
        selection,
        BTreeMap::from([
            (PropertyId::Designation, PropertyValue::Text("-Bulk".into())),
            (
                PropertyId::ConductorA,
                PropertyValue::OptionalObject(Some(ObjectRef::Terminal(TerminalId::new(1)?))),
            ),
        ]),
    )?;
    let preview_ms = super::millis(start.elapsed());
    let expected = usize::try_from(count * 2)?;
    if plan.changes.len() != expected || plan.inapplicable.len() != expected {
        return Err("property plan changed the wrong subsets".into());
    }
    let before = project.clone();
    let mut session = Session::new(project);
    let start = Instant::now();
    session.execute_batch(session.revision(), plan.commands)?;
    let commit_ms = super::millis(start.elapsed());
    session.project().validate()?;
    if session.undo_len() != 1
        || decode_project(&encode_project(session.project())?)? != *session.project()
    {
        return Err("property transaction history or round trip was incorrect".into());
    }
    let start = Instant::now();
    session.undo(session.revision())?;
    let undo_ms = super::millis(start.elapsed());
    if session.project() != &before {
        return Err("property undo changed identities or topology".into());
    }
    Ok(
        json!({"selected_objects": expected, "actual_changes": expected, "inapplicable_pairs": expected,
        "inspection_ms": inspection_ms, "preview_ms": preview_ms, "commit_ms": commit_ms,
        "undo_ms": undo_ms, "samples": 1, "correctness": "applicable subsets, one transaction, round trip and exact undo verified"}),
    )
}
