//! Real local Git interoperability; no remote, global configuration, or user repository.

use ecad_adapters::{JsonFile, ProjectFileError, decode_project};
use ecad_app::{Command, Document, ProjectStore, ReloadPolicy, Session};
use ecad_domain::{ConductorId, End};
use std::{path::Path, process::Command as Process};

fn git(root: &Path, arguments: &[&str]) -> String {
    let mut command = Process::new("git");
    command.current_dir(root);
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_CONFIG_PARAMETERS",
        "GIT_CONFIG_COUNT",
        "GIT_EXTERNAL_DIFF",
        "GIT_DIFF_OPTS",
        "GIT_TEMPLATE_DIR",
        "GIT_NAMESPACE",
    ] {
        command.env_remove(key);
    }
    command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", root.join("empty-git-config"))
        .env("GIT_ATTR_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_AUTHOR_NAME", "ECAD Test")
        .env("GIT_COMMITTER_NAME", "ECAD Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
        .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
        .args([
            "-c",
            "core.autocrlf=false",
            "-c",
            "core.hooksPath=disabled-hooks",
            "-c",
            "core.attributesFile=disabled-attributes",
            "-c",
            "core.excludesFile=disabled-excludes",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(arguments);
    let output = command
        .output()
        .expect("Git must be installed for local integration tests");
    assert!(
        output.status.success(),
        "git {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn known_usage_detects_git_checkout_without_touching_project_or_repository_state() {
    use ecad_adapters::KnownProjects;
    use ecad_app::UsageStatus;
    use ecad_domain::{
        DefinitionId, DefinitionRef, DefinitionRevisionId, Project, ProjectId, SubsystemId,
        SubsystemRevision,
    };
    use std::sync::Arc;

    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "--quiet"]);
    let path = root.join("project.json");
    let store = JsonFile::new(&path);
    let body = decode_project(include_str!("fixtures/sensor-module-v3.json")).unwrap();
    let first = DefinitionRef::new(
        DefinitionId::new(1).unwrap(),
        DefinitionRevisionId::new(1).unwrap(),
    );
    let second = DefinitionRef::new(first.definition(), DefinitionRevisionId::new(2).unwrap());
    let local_root = SubsystemId::new(10).unwrap();
    let mut project = Project::new(ProjectId::new(77).unwrap());
    project
        .insert_definition(Arc::new(
            SubsystemRevision::new(first, local_root, body.clone()).unwrap(),
        ))
        .unwrap();
    store.save(&project).unwrap();
    git(root, &["add", "project.json"]);
    git(
        root,
        &["commit", "--quiet", "-m", "Initial bundled revision"],
    );
    let mut known = KnownProjects::default();
    let key = known.register(&path).unwrap();
    known.refresh(&key).unwrap();
    git(root, &["checkout", "--quiet", "-b", "different-revision"]);
    project.remove_definition(first).unwrap();
    project
        .insert_definition(Arc::new(
            SubsystemRevision::new(second, local_root, body).unwrap(),
        ))
        .unwrap();
    store.save(&project).unwrap();
    git(root, &["add", "project.json"]);
    git(
        root,
        &["commit", "--quiet", "-m", "Replace unused bundled revision"],
    );
    assert_eq!(known.check_freshness(&key).unwrap(), UsageStatus::Changed);
    assert_eq!(
        known
            .index()
            .known_uses(first.definition())
            .next()
            .unwrap()
            .source,
        first
    );
    known.refresh(&key).unwrap();
    assert_eq!(
        known
            .index()
            .known_uses(first.definition())
            .next()
            .unwrap()
            .source,
        second
    );
    git(root, &["checkout", "--quiet", "main"]);
    assert_eq!(known.check_freshness(&key).unwrap(), UsageStatus::Changed);
    known.refresh(&key).unwrap();
    assert_eq!(
        known
            .index()
            .known_uses(first.definition())
            .next()
            .unwrap()
            .source,
        first
    );
    assert_eq!(git(root, &["status", "--porcelain"]), "");
}

#[test]
fn stable_saves_focused_diffs_and_branch_checkout_preserve_electrical_data() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "--quiet"]);
    let store = JsonFile::new(root.join("project.json"));
    let original = decode_project(include_str!("fixtures/minimal-v4.json")).unwrap();
    let mut session = Session::new(original.clone());
    store.save(session.project()).unwrap();
    git(root, &["add", "project.json"]);
    git(
        root,
        &["commit", "--quiet", "-m", "Initial electrical fixture"],
    );
    store.save(session.project()).unwrap();
    assert_eq!(git(root, &["status", "--porcelain"]), "");

    git(root, &["checkout", "--quiet", "-b", "disconnect-core"]);
    session
        .execute(
            session.revision(),
            Command::SetTermination {
                conductor: ConductorId::new(1001).unwrap(),
                end: End::B,
                terminal: None,
            },
        )
        .unwrap();
    store.save(session.project()).unwrap();
    assert_eq!(git(root, &["diff", "--numstat"]), "1\t1\tproject.json\n");
    let edited = session.project().clone();
    session.undo(session.revision()).unwrap();
    store.save(session.project()).unwrap();
    assert_eq!(git(root, &["status", "--porcelain"]), "");
    session.redo(session.revision()).unwrap();
    store.save(session.project()).unwrap();
    git(root, &["add", "project.json"]);
    git(
        root,
        &["commit", "--quiet", "-m", "Leave one core unterminated"],
    );
    git(root, &["checkout", "--quiet", "main"]);
    assert_eq!(store.load().unwrap(), original);
    git(root, &["checkout", "--quiet", "disconnect-core"]);
    assert_eq!(store.load().unwrap(), edited);
}

#[test]
fn branch_checkout_is_detected_and_reload_invalidates_the_previous_session() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    git(root, &["init", "--quiet"]);
    let store = JsonFile::new(root.join("project.json"));
    let original = decode_project(include_str!("fixtures/minimal-v4.json")).unwrap();
    store.save(&original).unwrap();
    git(root, &["add", "project.json"]);
    git(root, &["commit", "--quiet", "-m", "Initial project"]);
    let mut document = Document::open(store).unwrap();
    git(root, &["checkout", "--quiet", "-b", "variant"]);
    document
        .execute(
            document.session().revision(),
            Command::SetTermination {
                conductor: ConductorId::new(1001).unwrap(),
                end: End::B,
                terminal: None,
            },
        )
        .unwrap();
    document.save().unwrap();
    git(root, &["add", "project.json"]);
    git(root, &["commit", "--quiet", "-m", "Variant connection"]);
    let old = document.session().revision();
    git(root, &["checkout", "--quiet", "main"]);
    assert!(document.has_external_changes().unwrap());
    assert!(matches!(
        document.save(),
        Err(ProjectFileError::ExternalChange(_))
    ));
    document.reload(old, ReloadPolicy::RequireClean).unwrap();
    assert_eq!(document.session().project(), &original);
    assert_ne!(document.session().revision().epoch(), old.epoch());
    assert_eq!(document.session().undo_len(), 0);
    assert!(!document.has_external_changes().unwrap());
    assert_eq!(git(root, &["status", "--porcelain"]), "");
}
