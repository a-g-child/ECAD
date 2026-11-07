//! Place two independently wired instances, reopen them, and undo making one local.

use ecad_adapters::{JsonFile, RandomIds, decode_project};
use ecad_app::{Command, Document, instance_commands};
use ecad_domain::{DefinitionRef, End, InstanceLimits, Project, SubsystemId, SubsystemRevision};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let body = decode_project(include_str!("../tests/fixtures/sensor-module-v3.json"))?;
    let source = DefinitionRef::new(RandomIds::definition()?, RandomIds::definition_revision()?);
    let revision = Arc::new(SubsystemRevision::new(source, SubsystemId::new(10)?, body)?);
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("reusable.json"));
    let mut document = Document::new(Project::new(RandomIds::project()?), store.clone());
    document.execute(
        document.session().revision(),
        Command::InsertDefinition(Arc::clone(&revision)),
    )?;
    let mut roots = Vec::new();
    for name in ["Left sensor module", "Right sensor module"] {
        let ids = RandomIds::instance_ids(&revision, InstanceLimits::default())?;
        roots.push(ids.subsystems[&revision.root()]);
        let commands = instance_commands(
            document.session().project(),
            source,
            ids,
            None,
            name,
            InstanceLimits::default(),
        )?;
        document.execute_batch(document.session().revision(), commands)?;
    }
    document.save()?;
    let mut reopened = Document::open(store)?;
    if reopened.session().project() != document.session().project() {
        return Err("repeated instances changed during save/reopen".into());
    }
    let before = reopened.session().project().clone();
    let first = before
        .instance(roots[0])
        .ok_or("first instance disappeared")?;
    let wire = *first
        .ids()
        .conductors
        .values()
        .next()
        .ok_or("missing mapped conductor")?;
    let change = Command::SetTermination {
        conductor: wire,
        end: End::B,
        terminal: None,
    };
    if reopened
        .execute(reopened.session().revision(), change.clone())
        .is_ok()
    {
        return Err("pinned internals were unexpectedly editable".into());
    }
    reopened.execute_batch(
        reopened.session().revision(),
        vec![Command::DetachInstance(roots[0]), change],
    )?;
    if reopened.session().project().instance(roots[1]).is_none() {
        return Err("local edit affected the other instance".into());
    }
    reopened.undo(reopened.session().revision())?;
    if reopened.session().project() != &before || reopened.session().is_dirty() {
        return Err("undo did not restore the pinned saved state".into());
    }
    println!(
        "One immutable revision, two separately identified sensor modules; guarded save/reopen and make-local/undo verified."
    );
    Ok(())
}
