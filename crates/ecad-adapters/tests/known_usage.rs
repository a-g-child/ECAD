//! Registered-file coverage, labelled historical references, and read-only discovery.

use ecad_adapters::{KnownProjects, RandomIds, decode_project, encode_project};
use ecad_app::{Command, ProjectUsage, Session, UsageStatus, adoption_plan, instance_commands};
use ecad_domain::*;
use std::{fs, path::Path, sync::Arc};

fn source(number: u128) -> DefinitionRef {
    DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(number).unwrap(),
    )
}
fn project() -> (Project, InstanceBinding, InstanceBinding) {
    let body = decode_project(include_str!("fixtures/sensor-module-v3.json")).unwrap();
    let root = SubsystemId::new(10).unwrap();
    let mut project = Project::new(ProjectId::new(77).unwrap());
    project
        .insert_definition(Arc::new(
            SubsystemRevision::new(source(1), root, body.clone()).unwrap(),
        ))
        .unwrap();
    let mut changed = body;
    changed
        .set_termination(ConductorId::new(2).unwrap(), End::B, None)
        .unwrap();
    project
        .insert_definition(Arc::new(
            SubsystemRevision::new(source(2), root, changed).unwrap(),
        ))
        .unwrap();
    let mut session = Session::new(project);
    let mut pins = Vec::new();
    for name in ["Left", "Right"] {
        let revision = session.project().definition(source(1)).unwrap();
        let ids = RandomIds::instance_ids(revision, InstanceLimits::default()).unwrap();
        let placed = ids.subsystems[&root];
        let commands = instance_commands(
            session.project(),
            source(1),
            ids,
            None,
            name,
            InstanceLimits::default(),
        )
        .unwrap();
        session.execute_batch(session.revision(), commands).unwrap();
        pins.push(session.project().instance(placed).unwrap().clone());
    }
    session
        .execute(session.revision(), Command::DetachInstance(pins[1].root()))
        .unwrap();
    (session.project().clone(), pins.remove(0), pins.remove(0))
}
fn write(path: &Path, project: &Project) -> Vec<u8> {
    let text = encode_project(project).unwrap();
    fs::write(path, &text).unwrap();
    text.into_bytes()
}

#[test]
fn summary_distinguishes_exact_pins_local_ancestry_and_bundled_only_revisions() {
    let (project, pin, local) = project();
    let before = project.clone();
    let usage = ProjectUsage::from_project(&project);
    assert_eq!(usage.project_id(), project.id());
    assert_eq!(usage.revisions().len(), 2);
    assert_eq!(
        usage.revision(source(1)).unwrap().pinned,
        [pin.root()].into()
    );
    assert_eq!(
        usage.revision(source(1)).unwrap().local_origins,
        [local.root()].into()
    );
    assert!(usage.revision(source(2)).unwrap().pinned.is_empty());
    assert!(usage.revision(source(2)).unwrap().local_origins.is_empty());
    assert!(usage.revision(source(3)).is_none());
    assert_eq!(project, before);
}

#[test]
fn scan_coverage_includes_unscanned_invalid_and_missing_files_without_aborting_other_locations() {
    let directory = tempfile::tempdir().unwrap();
    let valid = directory.path().join("valid.json");
    let invalid = directory.path().join("invalid.json");
    let missing = directory.path().join("missing.json");
    let unregistered = directory.path().join("unregistered.json");
    let (project, _, _) = project();
    let bytes = write(&valid, &project);
    fs::write(&invalid, "invalid").unwrap();
    fs::write(&unregistered, "untouched").unwrap();
    let mut known = KnownProjects::default();
    let good = known.register(&valid).unwrap();
    let bad = known.register(&invalid).unwrap();
    let absent = known.register(&missing).unwrap();
    assert!(
        known
            .index()
            .entries()
            .all(|(_, record)| record.status() == UsageStatus::Unscanned)
    );
    let report = known.rescan();
    assert_eq!(report.attempted, 3);
    assert_eq!(report.failures.len(), 2);
    assert_eq!(
        known.index().entry(&good).unwrap().status(),
        UsageStatus::Observed
    );
    assert_eq!(
        known.index().entry(&bad).unwrap().status(),
        UsageStatus::Unavailable
    );
    assert_eq!(
        known.index().entry(&absent).unwrap().status(),
        UsageStatus::Unavailable
    );
    assert!(known.index().entry(&bad).unwrap().diagnostic().is_some());
    assert_eq!(known.index().known_uses(source(1).definition()).count(), 2); // Two bundled revisions in one location.
    assert_eq!(
        known.check_freshness(&absent).unwrap(),
        UsageStatus::Missing
    );
    assert_eq!(
        known.check_freshness(&good).unwrap(),
        UsageStatus::MatchesObserved
    );
    assert_eq!(fs::read(valid).unwrap(), bytes);
    assert_eq!(fs::read_to_string(invalid).unwrap(), "invalid");
    assert_eq!(fs::read_to_string(unregistered).unwrap(), "untouched");
    assert!(!missing.exists());
}

#[test]
fn adoption_changes_are_detected_and_last_known_pins_change_only_on_refresh() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let (project, pin, local) = project();
    let bytes = write(&path, &project);
    let mut known = KnownProjects::default();
    let key = known.register(&path).unwrap();
    known.refresh(&key).unwrap();
    let ids = RandomIds::instance_ids(
        project.definition(source(2)).unwrap(),
        InstanceLimits::default(),
    )
    .unwrap();
    let plan = adoption_plan(
        &project,
        pin.root(),
        source(2),
        ids,
        InstanceLimits::default(),
    )
    .unwrap();
    let mut session = Session::new(project);
    session.execute(session.revision(), plan.command).unwrap();
    let changed = write(&path, session.project());
    assert_ne!(bytes, changed);
    assert_eq!(known.check_freshness(&key).unwrap(), UsageStatus::Changed);
    let record = known.index().entry(&key).unwrap();
    assert_eq!(
        record
            .last_known()
            .unwrap()
            .revision(source(1))
            .unwrap()
            .pinned,
        [pin.root()].into()
    );
    assert!(
        known
            .index()
            .known_uses(source(1).definition())
            .all(|hit| hit.record.status() == UsageStatus::Changed)
    );
    known.refresh(&key).unwrap();
    let usage = known.index().entry(&key).unwrap().last_known().unwrap();
    assert!(usage.revision(source(1)).unwrap().pinned.is_empty());
    assert_eq!(
        usage.revision(source(1)).unwrap().local_origins,
        [local.root()].into()
    );
    assert_eq!(
        usage.revision(source(2)).unwrap().pinned,
        [pin.root()].into()
    );
    assert_eq!(fs::read(path).unwrap(), changed);
}

#[test]
fn invalid_or_deleted_sources_retain_historical_hits_until_a_valid_refresh_replaces_them() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let (project, _, _) = project();
    let bytes = write(&path, &project);
    let mut known = KnownProjects::default();
    let key = known.register(&path).unwrap();
    known.refresh(&key).unwrap();
    fs::write(&path, "invalid").unwrap();
    assert!(known.refresh(&key).is_err());
    assert_eq!(known.index().known_uses(source(1).definition()).count(), 2);
    assert!(
        known
            .index()
            .known_uses(source(1).definition())
            .all(|hit| hit.record.status() == UsageStatus::Unavailable)
    );
    fs::write(&path, &bytes).unwrap();
    assert_eq!(
        known.check_freshness(&key).unwrap(),
        UsageStatus::MatchesObserved
    );
    assert!(known.index().entry(&key).unwrap().diagnostic().is_none());
    fs::remove_file(&path).unwrap();
    assert_eq!(known.check_freshness(&key).unwrap(), UsageStatus::Missing);
    assert_eq!(known.index().known_uses(source(1).definition()).count(), 2);
    let replacement = Project::new(ProjectId::new(88).unwrap());
    write(&path, &replacement);
    known.refresh(&key).unwrap();
    assert_eq!(
        known
            .index()
            .entry(&key)
            .unwrap()
            .last_known()
            .unwrap()
            .project_id(),
        replacement.id()
    );
    assert_eq!(known.index().known_uses(source(1).definition()).count(), 0);
}

#[test]
fn copied_worktrees_share_lineage_but_keep_distinct_locations_and_parent_aliases_deduplicate() {
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir(directory.path().join("nested")).unwrap();
    let a = directory.path().join("a.json");
    let b = directory.path().join("b.json");
    let (project, _, _) = project();
    let bytes = write(&a, &project);
    write(&b, &project);
    let mut known = KnownProjects::default();
    let key = known.register(&a).unwrap();
    known.refresh(&key).unwrap();
    assert_eq!(
        known
            .register(directory.path().join("nested/../a.json"))
            .unwrap(),
        key
    );
    assert_eq!(
        known.index().entry(&key).unwrap().status(),
        UsageStatus::Observed
    );
    known.register(&b).unwrap();
    assert!(known.rescan().failures.is_empty());
    assert_eq!(known.index().entries().len(), 2);
    assert!(
        known
            .index()
            .entries()
            .all(|(_, record)| record.last_known().unwrap().project_id() == project.id())
    );
    assert_eq!(known.index().known_uses(source(1).definition()).count(), 4);
    assert!(known.unregister(&key));
    assert!(!known.unregister(&key));
    assert_eq!(known.index().entries().len(), 1);
    assert_eq!(fs::read(a).unwrap(), bytes);
}

#[test]
fn moved_files_require_explicit_registration_and_old_locations_remain_labelled_missing() {
    let directory = tempfile::tempdir().unwrap();
    let old = directory.path().join("old.json");
    let new = directory.path().join("new.json");
    let (project, _, _) = project();
    write(&old, &project);
    let mut known = KnownProjects::default();
    let old_key = known.register(&old).unwrap();
    known.refresh(&old_key).unwrap();
    fs::rename(&old, &new).unwrap();
    assert_eq!(
        known.check_freshness(&old_key).unwrap(),
        UsageStatus::Missing
    );
    let new_key = known.register(&new).unwrap();
    known.refresh(&new_key).unwrap();
    assert_eq!(known.index().entries().len(), 2);
    assert!(known.unregister(&old_key));
    assert_eq!(known.index().known_uses(source(1).definition()).count(), 2);
    assert!(known.register(directory.path()).is_err());
    assert!(
        known
            .register(directory.path().join("absent-parent/file.json"))
            .is_err()
    );
}

#[cfg(unix)]
#[test]
fn existing_file_symlink_aliases_resolve_to_one_registered_location() {
    let directory = tempfile::tempdir().unwrap();
    let real = directory.path().join("real.json");
    let alias = directory.path().join("alias.json");
    write(&real, &project().0);
    std::os::unix::fs::symlink(&real, &alias).unwrap();
    let mut known = KnownProjects::default();
    let key = known.register(&real).unwrap();
    assert_eq!(known.register(alias).unwrap(), key);
    assert_eq!(known.index().entries().len(), 1);
    let dangling = directory.path().join("dangling.json");
    std::os::unix::fs::symlink(directory.path().join("absent.json"), &dangling).unwrap();
    assert!(known.register(dangling).is_err());
}

#[test]
fn same_length_valid_edits_are_detected_by_content_rather_than_file_size() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let bytes = write(&path, &project().0);
    let mut known = KnownProjects::default();
    let key = known.register(&path).unwrap();
    known.refresh(&key).unwrap();
    let changed = String::from_utf8(bytes.clone())
        .unwrap()
        .replacen("\"Left\"", "\"Next\"", 1);
    assert_eq!(bytes.len(), changed.len());
    assert_ne!(bytes, changed.as_bytes());
    decode_project(&changed).unwrap(); // A root-name occurrence parameter remains valid.
    fs::write(&path, changed).unwrap();
    assert_eq!(known.check_freshness(&key).unwrap(), UsageStatus::Changed);
    fs::write(&path, bytes).unwrap();
    assert_eq!(
        known.check_freshness(&key).unwrap(),
        UsageStatus::MatchesObserved
    );
}
