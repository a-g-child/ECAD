//! Register chosen files, inspect coverage, and retain labelled historical use after deletion.

use ecad_adapters::{JsonFile, KnownProjects, RandomIds, decode_project};
use ecad_app::{Command, Document, UsageStatus, instance_commands};
use ecad_domain::*;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("project.json");
    let body = decode_project(include_str!("../tests/fixtures/sensor-module-v3.json"))?;
    let source = DefinitionRef::new(RandomIds::definition()?, RandomIds::definition_revision()?);
    let revision = Arc::new(SubsystemRevision::new(source, SubsystemId::new(10)?, body)?);
    let mut document = Document::new(Project::new(RandomIds::project()?), JsonFile::new(&path));
    document.execute(
        document.session().revision(),
        Command::InsertDefinition(Arc::clone(&revision)),
    )?;
    let ids = RandomIds::instance_ids(&revision, InstanceLimits::default())?;
    let root = ids.subsystems[&revision.root()];
    let commands = instance_commands(
        document.session().project(),
        source,
        ids,
        None,
        "Sensor",
        InstanceLimits::default(),
    )?;
    document.execute_batch(document.session().revision(), commands)?;
    document.save()?;
    let mut known = KnownProjects::default();
    let key = known.register(&path)?;
    known.register(directory.path().join("unavailable.json"))?;
    let report = known.rescan();
    println!(
        "Registered coverage: {} locations scanned, {} unavailable",
        report.attempted,
        report.failures.len()
    );
    if report.attempted != 2 || report.failures.len() != 1 {
        return Err("scan hid incomplete coverage".into());
    }
    let hit = known
        .index()
        .known_uses(source.definition())
        .next()
        .ok_or("missing known use")?;
    if hit.usage.pinned.len() != 1 || hit.record.status() != UsageStatus::Observed {
        return Err("initial usage summary did not preserve exact pins".into());
    }
    document.execute(document.session().revision(), Command::DetachInstance(root))?;
    document.save()?;
    if known.check_freshness(&key)? != UsageStatus::Changed {
        return Err("changed bytes were not detected".into());
    }
    known.refresh(&key)?;
    let hit = known
        .index()
        .known_uses(source.definition())
        .next()
        .ok_or("missing local ancestry")?;
    if !hit.usage.pinned.is_empty() || hit.usage.local_origins.len() != 1 {
        return Err("local ancestry was confused with an exact pin".into());
    }
    std::fs::remove_file(&path)?;
    if known.check_freshness(&key)? != UsageStatus::Missing
        || known.index().known_uses(source.definition()).count() != 1
    {
        return Err("missing source erased its historical usage".into());
    }
    println!(
        "Pinned use became local ancestry after refresh; deleting its file retained a labelled last-known reference. Discovery changed no project data."
    );
    Ok(())
}
