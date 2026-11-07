//! Shared publication stays independent from consuming document histories.

use super::*;
use ecad_adapters::{JsonFile, RandomIds};
use ecad_app::{Document, adoption_plan, instance_commands, library_import};

fn consumer(
    path: &std::path::Path,
    library: &SubsystemLibrary,
) -> (Document<JsonFile>, SubsystemId) {
    let mut document = Document::new(
        Project::new(RandomIds::project().unwrap()),
        JsonFile::new(path),
    );
    document
        .execute(
            document.session().revision(),
            library_import(library, source(1)).unwrap(),
        )
        .unwrap();
    let revision = library.revision(source(1)).unwrap();
    let ids = RandomIds::instance_ids(revision, InstanceLimits::default()).unwrap();
    let root = ids.subsystems[&revision.root()];
    let commands = instance_commands(
        document.session().project(),
        source(1),
        ids,
        None,
        "Sensor",
        InstanceLimits::default(),
    )
    .unwrap();
    document
        .execute_batch(document.session().revision(), commands)
        .unwrap();
    document.save().unwrap();
    (document, root)
}

#[test]
fn two_projects_remain_pinned_until_explicit_import_and_adoption_and_reopen_without_library() {
    let directory = tempfile::tempdir().unwrap();
    let library_file = LibraryFile::new(directory.path().join("library.json"));
    let mut publisher = LibraryPublisher::new(library_file.clone());
    publisher
        .publish(revision(1), Recommendation::RecommendPublished)
        .unwrap();
    let path_a = directory.path().join("a.json");
    let path_b = directory.path().join("b.json");
    let (mut a, root_a) = consumer(&path_a, publisher.catalog());
    let (b, root_b) = consumer(&path_b, publisher.catalog());
    let bytes_a = fs::read(&path_a).unwrap();
    let bytes_b = fs::read(&path_b).unwrap();
    let before = a.session().project().clone();
    publisher
        .publish(revision(2), Recommendation::RecommendPublished)
        .unwrap();
    assert_eq!(fs::read(&path_a).unwrap(), bytes_a);
    assert_eq!(fs::read(&path_b).unwrap(), bytes_b);
    assert!(!a.session().is_dirty());
    assert!(!b.session().is_dirty());
    assert_eq!(
        b.session().project().instance(root_b).unwrap().source(),
        source(1)
    );
    a.execute(
        a.session().revision(),
        library_import(publisher.catalog(), source(2)).unwrap(),
    )
    .unwrap();
    assert_eq!(
        a.session().project().instance(root_a).unwrap().source(),
        source(1)
    );
    let target = publisher.catalog().revision(source(2)).unwrap();
    let ids = RandomIds::instance_ids(target, InstanceLimits::default()).unwrap();
    let plan = adoption_plan(
        a.session().project(),
        root_a,
        source(2),
        ids,
        InstanceLimits::default(),
    )
    .unwrap();
    a.execute(a.session().revision(), plan.command).unwrap();
    a.save().unwrap();
    a.undo(a.session().revision()).unwrap(); // Undo adoption, not external publication.
    a.undo(a.session().revision()).unwrap(); // Undo the now-unused project import.
    assert_eq!(a.session().project(), &before);
    let published = LibraryPublisher::open(library_file.clone()).unwrap();
    assert_eq!(published.catalog().revisions().len(), 2);
    assert_eq!(
        published.catalog().recommended(source(1).definition()),
        Some(source(2))
    );
    // The saved A still uses revision 2; B uses revision 1. No catalog lookup is
    // needed when reopening either document, even if its library is unavailable.
    fs::remove_file(library_file.path()).unwrap();
    assert_eq!(
        Document::open(JsonFile::new(path_a))
            .unwrap()
            .session()
            .project()
            .instance(root_a)
            .unwrap()
            .source(),
        source(2)
    );
    assert_eq!(
        Document::open(JsonFile::new(path_b))
            .unwrap()
            .session()
            .project()
            .instance(root_b)
            .unwrap()
            .source(),
        source(1)
    );
}

#[test]
fn importing_conflicting_content_fails_without_changing_project_or_publication() {
    let directory = tempfile::tempdir().unwrap();
    let mut publisher =
        LibraryPublisher::new(LibraryFile::new(directory.path().join("library.json")));
    publisher
        .publish(revision(1), Recommendation::RecommendPublished)
        .unwrap();
    let (mut document, _) = consumer(&directory.path().join("a.json"), publisher.catalog());
    let catalog = publisher.catalog().clone();
    assert!(library_import(&catalog, source(99)).is_err());
    let mut foreign = SubsystemLibrary::default();
    let conflicting = SubsystemRevision::new(
        source(1),
        SubsystemId::new(10).unwrap(),
        revision(2).content().clone(),
    )
    .unwrap();
    foreign.publish(Arc::new(conflicting)).unwrap();
    let before = document.session().project().clone();
    let history = document.session().undo_len();
    assert!(
        document
            .execute(
                document.session().revision(),
                library_import(&foreign, source(1)).unwrap()
            )
            .is_err()
    );
    assert_eq!(document.session().project(), &before);
    assert_eq!(document.session().undo_len(), history);
    assert!(!document.session().is_dirty());
    assert_eq!(publisher.catalog(), &catalog);
    // Identical imports are ordinary no-op commands, not duplicate history entries.
    assert!(
        !document
            .execute(
                document.session().revision(),
                library_import(&catalog, source(1)).unwrap()
            )
            .unwrap()
    );
    assert_eq!(document.session().undo_len(), history);
}
