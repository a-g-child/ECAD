//! Open same-lineage file copies and edit/save only the captured project, independently of focus.
use ecad_adapters::{JsonFile, encode_project};
use ecad_app::{CloseProjectPolicy, Command, ProjectWorkspace};
use ecad_domain::{MetadataField, Project, ProjectId, ProjectMetadata};
use std::collections::BTreeMap;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let a = directory.path().join("a.json");
    let b = directory.path().join("b.json");
    std::fs::write(&a, encode_project(&Project::new(ProjectId::new(1)?))?)?;
    std::fs::copy(&a, &b)?;
    let mut workspace = ProjectWorkspace::default();
    let aid = workspace.open(JsonFile::new(&a), a.to_string_lossy().into_owned())?;
    let bid = workspace.open(JsonFile::new(&b), b.to_string_lossy().into_owned())?;
    let revision = workspace.project(aid)?.document().session().revision();
    workspace.execute(
        aid,
        revision,
        Command::ProjectMetadata(ProjectMetadata::new(BTreeMap::from([(
            MetadataField::ProjectTitle,
            "Project A".into(),
        )]))),
    )?;
    assert_eq!(workspace.active(), Some(bid));
    assert_eq!(workspace.dirty_projects(), vec![aid]);
    assert!(
        workspace
            .project(bid)?
            .document()
            .session()
            .project()
            .metadata()
            .value(MetadataField::ProjectTitle)
            .is_none()
    );
    let revision = workspace.project(aid)?.document().session().revision();
    workspace.save(aid, revision)?;
    workspace.close(aid, revision, CloseProjectPolicy::RequireClean)?;
    let revision = workspace.project(bid)?.document().session().revision();
    workspace.close(bid, revision, CloseProjectPolicy::RequireClean)?;
    assert_eq!(workspace.projects().len(), 0);
    println!(
        "Two same-lineage projects: independent edits, dirty state, saves and close guards passed."
    );
    Ok(())
}
