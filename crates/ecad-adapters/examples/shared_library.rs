//! Publish shared content, import explicitly, and keep consuming projects independent.

use ecad_adapters::{JsonFile, LibraryFile, RandomIds, decode_project};
use ecad_app::{
    Document, LibraryPublisher, Recommendation, adoption_plan, instance_commands, library_import,
};
use ecad_domain::*;
use std::{path::Path, sync::Arc};

fn consumer(
    path: &Path,
    library: &SubsystemLibrary,
    source: DefinitionRef,
) -> Result<(Document<JsonFile>, SubsystemId), Box<dyn std::error::Error>> {
    let mut document = Document::new(Project::new(RandomIds::project()?), JsonFile::new(path));
    document.execute(
        document.session().revision(),
        library_import(library, source)?,
    )?;
    let revision = library
        .revision(source)
        .ok_or("missing published revision")?;
    let ids = RandomIds::instance_ids(revision, InstanceLimits::default())?;
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
    Ok((document, root))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let library_file = LibraryFile::new(directory.path().join("subsystems.json"));
    let mut publisher = LibraryPublisher::new(library_file.clone());
    let source = DefinitionRef::new(RandomIds::definition()?, RandomIds::definition_revision()?);
    let body = decode_project(include_str!("../tests/fixtures/sensor-module-v3.json"))?;
    let root = SubsystemId::new(10)?;
    publisher.publish(
        Arc::new(SubsystemRevision::new(source, root, body.clone())?),
        Recommendation::RecommendPublished,
    )?;
    let (mut a, root_a) = consumer(
        &directory.path().join("a.json"),
        publisher.catalog(),
        source,
    )?;
    let (b, root_b) = consumer(
        &directory.path().join("b.json"),
        publisher.catalog(),
        source,
    )?;
    let mut changed = body;
    changed.set_termination(ConductorId::new(2)?, End::B, None)?;
    let next = DefinitionRef::new(source.definition(), RandomIds::definition_revision()?);
    let revision = Arc::new(SubsystemRevision::new(next, root, changed)?);
    let ids = RandomIds::instance_ids(&revision, InstanceLimits::default())?;
    publisher.publish(revision, Recommendation::RecommendPublished)?;
    if a.session().is_dirty() || b.session().is_dirty() {
        return Err("publication unexpectedly edited a consuming project".into());
    }
    a.execute(
        a.session().revision(),
        library_import(publisher.catalog(), next)?,
    )?;
    let preview = adoption_plan(
        a.session().project(),
        root_a,
        next,
        ids,
        InstanceLimits::default(),
    )?;
    a.execute(a.session().revision(), preview.command)?;
    if a.session()
        .project()
        .instance(root_a)
        .ok_or("missing A pin")?
        .source()
        != next
        || b.session()
            .project()
            .instance(root_b)
            .ok_or("missing B pin")?
            .source()
            != source
    {
        return Err("consuming projects did not retain independent pins".into());
    }
    a.undo(a.session().revision())?;
    a.undo(a.session().revision())?;
    let reopened = LibraryPublisher::open(library_file)?;
    if reopened.catalog().recommended(source.definition()) != Some(next)
        || reopened.catalog().revisions().len() != 2
    {
        return Err("project undo erased shared publication".into());
    }
    println!(
        "Two revisions published; projects stayed pinned until explicit import/adoption. Project undo left shared history and its recommendation intact."
    );
    Ok(())
}
