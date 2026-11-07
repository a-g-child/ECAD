//! Explicitly review/adopt one occurrence while another remains pinned, then undo.

use ecad_adapters::{JsonFile, RandomIds, decode_project};
use ecad_app::{Command, Document, adoption_plan, instance_commands};
use ecad_domain::*;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let body = decode_project(include_str!("../tests/fixtures/sensor-module-v3.json"))?;
    let first = DefinitionRef::new(RandomIds::definition()?, RandomIds::definition_revision()?);
    let revision = Arc::new(SubsystemRevision::new(
        first,
        SubsystemId::new(10)?,
        body.clone(),
    )?);
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("adoption.json"));
    let mut document = Document::new(Project::new(RandomIds::project()?), store.clone());
    document.execute(
        document.session().revision(),
        Command::InsertDefinition(Arc::clone(&revision)),
    )?;
    let mut roots = Vec::new();
    for name in ["Left", "Right"] {
        let ids = RandomIds::instance_ids(&revision, InstanceLimits::default())?;
        roots.push(ids.subsystems[&revision.root()]);
        let commands = instance_commands(
            document.session().project(),
            first,
            ids,
            None,
            name,
            InstanceLimits::default(),
        )?;
        document.execute_batch(document.session().revision(), commands)?;
    }
    let terminal = document
        .session()
        .project()
        .instance(roots[0])
        .ok_or("missing instance")?
        .ids()
        .terminals[&TerminalId::new(201)?];
    let external = RandomIds::conductor()?;
    document.execute(
        document.session().revision(),
        Command::InsertConductor(Conductor::new(external, Some(terminal), None)),
    )?;
    let mut changed = body;
    changed.set_termination(ConductorId::new(2)?, End::B, None)?;
    let second = DefinitionRef::new(first.definition(), RandomIds::definition_revision()?);
    let revision = Arc::new(SubsystemRevision::new(
        second,
        SubsystemId::new(10)?,
        changed,
    )?);
    let ids = RandomIds::instance_ids(&revision, InstanceLimits::default())?;
    document.execute(
        document.session().revision(),
        Command::InsertDefinition(revision),
    )?;
    document.save()?;
    let before = document.session().project().clone();
    let plan = adoption_plan(&before, roots[0], second, ids, InstanceLimits::default())?;
    println!(
        "Review: {} modified internal conductor(s)",
        plan.changes.conductors.modified.len()
    );
    document.execute(document.session().revision(), plan.command)?;
    document.save()?;
    let reopened = Document::open(store)?;
    let project = reopened.session().project();
    if project != document.session().project()
        || project
            .instance(roots[0])
            .ok_or("missing first pin")?
            .source()
            != second
        || project
            .instance(roots[1])
            .ok_or("missing second pin")?
            .source()
            != first
        || project
            .conductor(external)
            .ok_or("missing external wire")?
            .endpoint(End::A)
            != Some(terminal)
    {
        return Err("adoption changed another pin, external wiring, or saved content".into());
    }
    document.undo(document.session().revision())?;
    if document.session().project() != &before {
        return Err("undo failed to restore original revision and content".into());
    }
    println!(
        "One occurrence adopted; the other pin and external connection stayed intact. Offline reopen and exact undo verified."
    );
    Ok(())
}
