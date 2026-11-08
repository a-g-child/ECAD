//! Several real project files retain independent models, revisions, history and save bindings.
use ecad_adapters::{JsonFile, ProjectFileError, decode_project, encode_project};
use ecad_app::*;
use ecad_domain::*;
use std::{collections::BTreeMap, path::Path};
type Workspace = ProjectWorkspace<JsonFile>;
fn fixture() -> Project {
    decode_project(include_str!("fixtures/minimal-v8.json")).unwrap()
}
fn key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
fn revision(w: &Workspace, id: WorkspaceProjectId) -> Revision {
    w.project(id).unwrap().document().session().revision()
}
fn metadata(value: &str) -> Command {
    Command::ProjectMetadata(ProjectMetadata::new(BTreeMap::from([(
        MetadataField::ProjectTitle,
        value.into(),
    )])))
}
fn title(w: &Workspace, id: WorkspaceProjectId) -> Option<&str> {
    w.project(id)
        .unwrap()
        .document()
        .session()
        .project()
        .metadata()
        .value(MetadataField::ProjectTitle)
}
fn pair(directory: &Path) -> (Workspace, WorkspaceProjectId, WorkspaceProjectId) {
    let a = directory.join("a.json");
    let b = directory.join("b.json");
    std::fs::write(&a, encode_project(&fixture()).unwrap()).unwrap();
    std::fs::copy(&a, &b).unwrap();
    let mut w = Workspace::default();
    let aid = w.open(JsonFile::new(&a), key(&a)).unwrap();
    let bid = w.open(JsonFile::new(&b), key(&b)).unwrap();
    (w, aid, bid)
}
#[test]
fn same_lineage_projects_keep_independent_history_dirty_state_and_targeted_saves_when_switching() {
    let directory = tempfile::tempdir().unwrap();
    let (mut w, a, b) = pair(directory.path());
    let arev = revision(&w, a);
    let brev = revision(&w, b);
    let bbytes = std::fs::read(directory.path().join("b.json")).unwrap();
    assert_eq!(w.active(), Some(b));
    assert_ne!(a, b);
    assert_eq!(arev.get(), brev.get());
    assert_ne!(arev.epoch(), brev.epoch());
    assert_eq!(
        w.project(a).unwrap().document().session().project().id(),
        w.project(b).unwrap().document().session().project().id()
    );
    w.execute_batch(
        a,
        arev,
        vec![
            metadata("Project A"),
            Command::SetTermination {
                conductor: ConductorId::new(1001).unwrap(),
                end: End::B,
                terminal: None,
            },
        ],
    )
    .unwrap();
    assert_eq!(
        w.project(a)
            .unwrap()
            .document()
            .session()
            .project()
            .conductor(ConductorId::new(1001).unwrap())
            .unwrap()
            .endpoint(End::B),
        None
    );
    assert_eq!(
        w.project(b)
            .unwrap()
            .document()
            .session()
            .project()
            .conductor(ConductorId::new(1001).unwrap())
            .unwrap()
            .endpoint(End::B),
        Some(TerminalId::new(201).unwrap())
    );
    assert_eq!(title(&w, a), Some("Project A"));
    assert_eq!(title(&w, b), None);
    assert_eq!(w.dirty_projects(), vec![a]);
    assert_eq!(w.active(), Some(b));
    assert!(matches!(
        w.execute(b, arev, metadata("Wrong target")),
        Err(WorkspaceError::Edit(EditError::StaleRevision { .. }))
    ));
    assert_eq!(revision(&w, b), brev);
    assert!(w.save(a, arev).is_err());
    let current = revision(&w, a);
    w.save(a, current).unwrap();
    assert!(w.dirty_projects().is_empty());
    assert_eq!(
        std::fs::read(directory.path().join("b.json")).unwrap(),
        bbytes
    );
    w.undo(a, revision(&w, a)).unwrap();
    assert_eq!(title(&w, a), None);
    assert_eq!(w.dirty_projects(), vec![a]);
    w.activate(a).unwrap();
    w.activate(b).unwrap();
    assert_eq!(revision(&w, b), brev);
    w.redo(a, revision(&w, a)).unwrap();
    assert!(w.dirty_projects().is_empty());
    w.close(b, brev, CloseProjectPolicy::RequireClean).unwrap();
    assert_eq!(w.active(), Some(a));
    w.close(a, revision(&w, a), CloseProjectPolicy::RequireClean)
        .unwrap();
    assert_eq!(w.active(), None);
    assert_eq!(w.projects().len(), 0);
}
#[test]
fn duplicate_location_focuses_without_reloading_and_save_as_rebinds_only_after_success() {
    let directory = tempfile::tempdir().unwrap();
    let (mut w, a, b) = pair(directory.path());
    let ap = directory.path().join("a.json");
    let bp = directory.path().join("b.json");
    let cp = directory.path().join("copy.json");
    w.execute(a, revision(&w, a), metadata("Unsaved A"))
        .unwrap();
    let rev = revision(&w, a);
    let original_a = std::fs::read(&ap).unwrap();
    std::fs::write(&ap, "temporarily invalid external bytes").unwrap();
    assert_eq!(w.open(JsonFile::new(&ap), key(&ap)).unwrap(), a);
    std::fs::write(&ap, &original_a).unwrap();
    assert_eq!(w.projects().len(), 2);
    assert_eq!(revision(&w, a), rev);
    assert_eq!(title(&w, a), Some("Unsaved A"));
    assert!(matches!(
        w.save_as(a, rev, JsonFile::new(&bp), key(&bp)),
        Err(WorkspaceError::LocationInUse(_))
    ));
    std::fs::copy(&bp, &cp).unwrap();
    let original_c = std::fs::read(&cp).unwrap();
    assert!(w.save_as(a, rev, JsonFile::new(&cp), key(&cp)).is_err());
    assert_eq!(w.project(a).unwrap().location(), key(&ap));
    assert_eq!(std::fs::read(&cp).unwrap(), original_c);
    assert_eq!(w.dirty_projects(), vec![a]);
    let c = w.open(JsonFile::new(&cp), key(&cp)).unwrap();
    assert_eq!(title(&w, c), None);
    let dp = directory.path().join("new-destination.json");
    w.save_as(a, rev, JsonFile::new(&dp), key(&dp)).unwrap();
    assert_eq!(w.project(a).unwrap().location(), key(&dp));
    assert!(w.dirty_projects().is_empty());
    assert_eq!(std::fs::read(&ap).unwrap(), original_a);
    assert!(std::fs::read_to_string(&dp).unwrap().contains("Unsaved A"));
    let reopened = w.open(JsonFile::new(&ap), key(&ap)).unwrap();
    assert_ne!(reopened, a);
    assert_eq!(title(&w, reopened), None);
    assert_eq!(revision(&w, b).get(), 0);
}
#[test]
fn failed_open_reload_and_stale_or_dirty_close_preserve_other_projects_and_workspace_handles() {
    let directory = tempfile::tempdir().unwrap();
    let (mut w, a, b) = pair(directory.path());
    w.execute(b, revision(&w, b), metadata("Keep B")).unwrap();
    let brev = revision(&w, b);
    let old = revision(&w, a);
    let missing = directory.path().join("missing.json");
    assert!(w.open(JsonFile::new(&missing), key(&missing)).is_err());
    assert_eq!(w.active(), Some(b));
    assert_eq!(w.projects().len(), 2);
    let ap = directory.path().join("a.json");
    std::fs::write(&ap, "broken").unwrap();
    assert!(matches!(
        w.save(a, old),
        Err(WorkspaceError::Store(ProjectFileError::ExternalChange(_)))
    ));
    assert_eq!(revision(&w, a), old);
    assert_eq!(revision(&w, b), brev);

    assert!(w.reload(a, old, ReloadPolicy::RequireClean).is_err());
    assert_eq!(revision(&w, a), old);
    assert_eq!(revision(&w, b), brev);
    std::fs::write(
        &ap,
        encode_project(&Project::new(ProjectId::new(999).unwrap())).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        w.reload(a, old, ReloadPolicy::RequireClean),
        Err(WorkspaceError::Reload(
            DocumentError::DifferentProject { .. }
        ))
    ));
    let mut disk = fixture();
    disk.replace_metadata(ProjectMetadata::new(BTreeMap::from([(
        MetadataField::ProjectTitle,
        "From disk".into(),
    )])));
    std::fs::write(&ap, encode_project(&disk).unwrap()).unwrap();
    w.reload(a, old, ReloadPolicy::RequireClean).unwrap();
    assert_ne!(revision(&w, a).epoch(), old.epoch());
    assert_eq!(w.project(a).unwrap().id(), a);
    assert_eq!(w.active(), Some(b));
    assert_eq!(title(&w, b), Some("Keep B"));
    assert!(
        w.close(a, old, CloseProjectPolicy::DiscardLocalChanges)
            .is_err()
    );
    w.execute(a, revision(&w, a), metadata("Dirty A")).unwrap();
    assert!(matches!(
        w.close(a, revision(&w, a), CloseProjectPolicy::RequireClean),
        Err(WorkspaceError::UnsavedChanges(_))
    ));
    assert_eq!(w.active(), Some(b));
    w.close(a, revision(&w, a), CloseProjectPolicy::DiscardLocalChanges)
        .unwrap();
    assert_eq!(w.active(), Some(b));
    assert_eq!(w.dirty_projects(), vec![b]);
    assert!(w.activate(a).is_err());
    assert_eq!(revision(&w, b), brev);
    assert!(w.close(b, brev, CloseProjectPolicy::RequireClean).is_err());
    w.close(b, brev, CloseProjectPolicy::DiscardLocalChanges)
        .unwrap();
    assert_eq!(w.projects().len(), 0);
}
fn invoke(
    w: &mut Workspace,
    id: WorkspaceProjectId,
    context: ActionContext,
    input: u64,
    action: &str,
    parameters: ActionParameters,
) -> Result<ActionOutcome, WorkspaceError<ProjectFileError>> {
    w.invoke(
        id,
        &ActionRegistry::builtins(),
        ActionInvocation {
            input_id: InputId::new(input).unwrap(),
            action: ActionId::new(action).unwrap(),
            source: InvocationSource::Toolbar,
            repeat: false,
            context,
            parameters,
        },
    )
}
#[test]
fn actions_keep_captured_project_targets_and_share_replay_protection_even_after_close() {
    let directory = tempfile::tempdir().unwrap();
    let (mut w, a, b) = pair(directory.path());
    let ctx = ActionContext::for_session(w.project(a).unwrap().document().session());
    let values = ProjectMetadata::new(BTreeMap::from([(
        MetadataField::ProjectTitle,
        "A action".into(),
    )]));
    invoke(
        &mut w,
        a,
        ctx,
        1,
        "document.metadata.edit",
        ActionParameters::ProjectMetadata(values),
    )
    .unwrap();
    assert_eq!(w.active(), Some(b));
    assert_eq!(title(&w, a), Some("A action"));
    assert_eq!(title(&w, b), None);
    let wrong = ActionContext::for_session(w.project(a).unwrap().document().session());
    assert!(invoke(&mut w, b, wrong, 2, "document.undo", ActionParameters::None).is_err());
    let ctx = ActionContext::for_session(w.project(b).unwrap().document().session());
    assert!(matches!(
        invoke(
            &mut w,
            b,
            ctx.clone(),
            2,
            "document.metadata.edit",
            ActionParameters::None
        ),
        Err(WorkspaceError::Action(ActionDispatchError::ReplayedInput(
            _
        )))
    ));
    invoke(
        &mut w,
        b,
        ctx,
        3,
        "document.metadata.edit",
        ActionParameters::ProjectMetadata(ProjectMetadata::new(BTreeMap::from([(
            MetadataField::ProjectTitle,
            "B action".into(),
        )]))),
    )
    .unwrap();
    let closed_context = ActionContext::for_session(w.project(a).unwrap().document().session());
    w.close(a, revision(&w, a), CloseProjectPolicy::DiscardLocalChanges)
        .unwrap();
    assert!(matches!(
        invoke(
            &mut w,
            a,
            closed_context,
            4,
            "document.undo",
            ActionParameters::None
        ),
        Err(WorkspaceError::UnknownProject(_))
    ));
    let ctx = ActionContext::for_session(w.project(b).unwrap().document().session());
    assert!(matches!(
        invoke(
            &mut w,
            b,
            ctx.clone(),
            4,
            "document.undo",
            ActionParameters::None
        ),
        Err(WorkspaceError::Action(ActionDispatchError::ReplayedInput(
            _
        )))
    ));
    invoke(&mut w, b, ctx, 5, "document.undo", ActionParameters::None).unwrap();
    assert_eq!(title(&w, b), None);
}
#[test]
fn unsaved_projects_and_reserved_or_empty_locations_are_guarded_without_startup_writes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("new.json");
    let mut w = Workspace::default();
    let id = w
        .new_project(fixture(), JsonFile::new(&path), key(&path))
        .unwrap();
    assert!(!path.exists());
    assert_eq!(w.dirty_projects(), vec![id]);
    assert!(
        w.close(id, revision(&w, id), CloseProjectPolicy::RequireClean)
            .is_err()
    );
    assert!(
        w.new_project(fixture(), JsonFile::new(&path), key(&path))
            .is_err()
    );
    assert!(w.open(JsonFile::new(&path), " \n ").is_err());
    assert_eq!(w.active(), Some(id));
    assert_eq!(w.projects().len(), 1);
    w.save(id, revision(&w, id)).unwrap();
    assert!(path.exists());
    let rev = revision(&w, id);
    assert!(
        w.execute_batch(
            id,
            rev,
            vec![
                metadata("Rollback"),
                Command::RemoveDevice(DeviceId::new(999).unwrap())
            ]
        )
        .is_err()
    );
    assert_eq!(revision(&w, id), rev);
    assert_eq!(title(&w, id), None);
    assert!(w.dirty_projects().is_empty());
}
