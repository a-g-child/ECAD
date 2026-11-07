//! Immutable publication, strict catalog persistence, and cooperating-writer conflicts.

#[path = "library/support.rs"]
mod support;
#[path = "library/workflow.rs"]
mod workflow;
use ecad_adapters::{LibraryFile, LibraryFileError, decode_library, encode_library};
use ecad_app::{LibraryPublisher, Recommendation, VersionedLibraryStore};
use ecad_domain::*;
use serde_json::{Value, json};
use std::{fs, sync::Arc};
use support::*;

#[test]
fn publication_and_recommendations_preserve_immutable_history_and_reject_bad_pointers() {
    let mut library = catalog();
    assert!(!library.publish(revision(1)).unwrap());
    library.publish(revision(2)).unwrap();
    assert_eq!(library.recommended(source(1).definition()), Some(source(1)));
    let before = library.clone();
    let conflict = Arc::new(
        SubsystemRevision::new(
            source(1),
            SubsystemId::new(10).unwrap(),
            revision(2).content().clone(),
        )
        .unwrap(),
    );
    assert_eq!(
        library.publish(conflict),
        Err(LibraryError::RevisionConflict(source(1)))
    );
    assert!(
        library
            .set_recommended(source(1).definition(), Some(source(99)))
            .is_err()
    );
    assert!(
        library
            .set_recommended(DefinitionId::new(99).unwrap(), Some(source(1)))
            .is_err()
    );
    assert_eq!(library, before);
    assert!(
        library
            .set_recommended(source(1).definition(), Some(source(2)))
            .unwrap()
    );
    assert!(
        library
            .set_recommended(source(1).definition(), Some(source(1)))
            .unwrap()
    );
    assert!(
        library
            .set_recommended(source(1).definition(), None)
            .unwrap()
    );
    assert_eq!(library.revisions().len(), 2);
}

#[test]
fn canonical_roundtrip_rejects_duplicate_entries_unknown_fields_and_unresolved_recommendations() {
    let mut library = catalog();
    assert_eq!(
        encode_library(&library).unwrap(),
        include_str!("fixtures/sensor-library-v1.json")
    );
    library.publish(revision(2)).unwrap();
    let text = encode_library(&library).unwrap();
    assert!(text.ends_with('\n'));
    assert_eq!(decode_library(&text).unwrap(), library);
    let mut data: Value = serde_json::from_str(&text).unwrap();
    data["revisions"].as_array_mut().unwrap().reverse();
    assert_eq!(
        encode_library(&decode_library(&data.to_string()).unwrap()).unwrap(),
        text
    );
    let mut repeated = data.clone();
    let item = repeated["revisions"][0].clone();
    repeated["revisions"].as_array_mut().unwrap().push(item);
    assert!(matches!(
        decode_library(&repeated.to_string()),
        Err(LibraryFileError::Catalog(LibraryError::DuplicateRevision(
            _
        )))
    ));
    let mut repeated = data.clone();
    let item = repeated["recommended"][0].clone();
    repeated["recommended"].as_array_mut().unwrap().push(item);
    assert!(matches!(
        decode_library(&repeated.to_string()),
        Err(LibraryFileError::Catalog(
            LibraryError::DuplicateRecommendation(_)
        ))
    ));
    let mut unresolved = data.clone();
    unresolved["recommended"][0]["revision"] = json!(source(99).revision().to_string());
    assert!(matches!(
        decode_library(&unresolved.to_string()),
        Err(LibraryFileError::Catalog(LibraryError::MissingRevision(_)))
    ));
    for field in ["kind", "schema_version", "revisions"] {
        let mut malformed = data.clone();
        malformed[field] = json!("invalid");
        assert!(decode_library(&malformed.to_string()).is_err());
    }
    for pointer in [
        "/unexpected",
        "/revisions/0/unexpected",
        "/revisions/0/body/unexpected",
        "/recommended/0/unexpected",
    ] {
        let mut malformed = data.clone();
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        let object = if parent.is_empty() {
            &mut malformed
        } else {
            malformed.pointer_mut(parent).unwrap()
        };
        object[key] = json!(true);
        assert!(decode_library(&malformed.to_string()).is_err());
    }
    data["schema_version"] = json!(2);
    assert!(decode_library(&data.to_string()).is_err());
}

#[test]
fn publisher_persists_body_and_recommendation_together_and_noops_preserve_exact_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let file = LibraryFile::new(directory.path().join("library.json"));
    let mut publisher = LibraryPublisher::new(file.clone());
    assert!(
        publisher
            .publish(revision(1), Recommendation::RecommendPublished)
            .unwrap()
    );
    let snapshot = file.load_snapshot().unwrap();
    assert_eq!(&snapshot.catalog, publisher.catalog());
    let bytes = fs::read(file.path()).unwrap();
    assert!(
        !publisher
            .publish(revision(1), Recommendation::KeepCurrent)
            .unwrap()
    );
    assert_eq!(fs::read(file.path()).unwrap(), bytes);
    assert_eq!(file.load_snapshot().unwrap().version, snapshot.version);
    assert!(
        publisher
            .publish(revision(2), Recommendation::KeepCurrent)
            .unwrap()
    );
    assert_eq!(
        publisher.catalog().recommended(source(1).definition()),
        Some(source(1))
    );
    publisher
        .set_recommended(source(1).definition(), Some(source(2)))
        .unwrap();
    publisher
        .set_recommended(source(1).definition(), Some(source(1)))
        .unwrap();
    publisher
        .set_recommended(source(1).definition(), None)
        .unwrap();
    assert_eq!(
        LibraryPublisher::open(file.clone()).unwrap().catalog(),
        publisher.catalog()
    );
    assert_eq!(publisher.catalog().revisions().len(), 2);
    assert!(!file.lock_path().unwrap().exists());
}

#[test]
fn stale_clients_and_unobserved_existing_destinations_cannot_overwrite_history_even_on_noop() {
    let directory = tempfile::tempdir().unwrap();
    let file = LibraryFile::new(directory.path().join("library.json"));
    let mut first = LibraryPublisher::new(file.clone());
    first
        .publish(revision(1), Recommendation::RecommendPublished)
        .unwrap();
    let mut second = LibraryPublisher::open(file.clone()).unwrap();
    let before = second.catalog().clone();
    first
        .publish(revision(2), Recommendation::RecommendPublished)
        .unwrap();
    assert!(matches!(
        second.publish(revision(1), Recommendation::KeepCurrent),
        Err(LibraryFileError::ExternalChange(_))
    ));
    assert_eq!(second.catalog(), &before);
    let mut unobserved = LibraryPublisher::new(file.clone());
    assert!(matches!(
        unobserved.publish(revision(3), Recommendation::KeepCurrent),
        Err(LibraryFileError::ExternalChange(_))
    ));
    assert_eq!(unobserved.catalog().revisions().len(), 0);
    second.reload().unwrap();
    second
        .publish(revision(3), Recommendation::KeepCurrent)
        .unwrap();
    assert_eq!(second.catalog().revisions().len(), 3);
    assert!(!file.lock_path().unwrap().exists());
}

#[test]
fn adapter_rejects_history_deletion_rewrites_and_tokens_from_another_location() {
    let directory = tempfile::tempdir().unwrap();
    let file = LibraryFile::new(directory.path().join("a.json"));
    let second = LibraryFile::new(directory.path().join("b.json"));
    let catalog = catalog();
    let version = file.save_if_unchanged(&catalog, None).unwrap();
    second.save_if_unchanged(&catalog, None).unwrap();
    let bytes = fs::read(file.path()).unwrap();
    assert!(matches!(
        file.save_if_unchanged(&SubsystemLibrary::default(), Some(&version)),
        Err(LibraryFileError::HistoryRewrite(_))
    ));
    let mut replacement = SubsystemLibrary::default();
    replacement
        .publish(Arc::new(
            SubsystemRevision::new(
                source(1),
                SubsystemId::new(10).unwrap(),
                revision(2).content().clone(),
            )
            .unwrap(),
        ))
        .unwrap();
    assert!(matches!(
        file.save_if_unchanged(&replacement, Some(&version)),
        Err(LibraryFileError::HistoryRewrite(_))
    ));
    assert!(matches!(
        second.save_if_unchanged(&catalog, Some(&version)),
        Err(LibraryFileError::ExternalChange(_))
    ));
    assert_eq!(fs::read(file.path()).unwrap(), bytes);
}

#[test]
fn held_or_stale_lock_is_reported_without_being_stolen_and_readers_still_work() {
    let directory = tempfile::tempdir().unwrap();
    let file = LibraryFile::new(directory.path().join("library.json"));
    let mut publisher = LibraryPublisher::new(file.clone());
    publisher
        .publish(revision(1), Recommendation::KeepCurrent)
        .unwrap();
    let lock = file.lock_path().unwrap();
    fs::write(&lock, "reserved by another writer").unwrap();
    let before = publisher.catalog().clone();
    assert!(matches!(
        publisher.publish(revision(2), Recommendation::RecommendPublished),
        Err(LibraryFileError::Busy(_))
    ));
    assert_eq!(publisher.catalog(), &before);
    assert_eq!(file.load_snapshot().unwrap().catalog, before);
    assert_eq!(
        fs::read_to_string(&lock).unwrap(),
        "reserved by another writer"
    );
    fs::remove_file(&lock).unwrap();
    publisher
        .publish(revision(2), Recommendation::RecommendPublished)
        .unwrap();
}

#[test]
fn cooperating_concurrent_publishers_preserve_winner_and_can_retry_after_explicit_reload() {
    let directory = tempfile::tempdir().unwrap();
    let file = LibraryFile::new(directory.path().join("library.json"));
    LibraryPublisher::new(file.clone())
        .publish(revision(1), Recommendation::RecommendPublished)
        .unwrap();
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let writers: Vec<_> = [2, 3]
        .into_iter()
        .map(|number| {
            let file = file.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                let mut publisher = LibraryPublisher::open(file).unwrap();
                barrier.wait();
                let result = publisher.publish(revision(number), Recommendation::KeepCurrent);
                (publisher, number, result)
            })
        })
        .collect();
    let mut successes = 0;
    let mut loser = None;
    for writer in writers {
        let (publisher, number, result) = writer.join().unwrap();
        match result {
            Ok(true) => successes += 1,
            Err(LibraryFileError::Busy(_) | LibraryFileError::ExternalChange(_)) => {
                assert_eq!(publisher.catalog().revisions().len(), 1);
                loser = Some((publisher, number));
            }
            other => panic!("unexpected publication result: {other:?}"),
        }
    }
    assert_eq!(successes, 1);
    let (mut publisher, number) = loser.unwrap();
    publisher.reload().unwrap();
    publisher
        .publish(revision(number), Recommendation::KeepCurrent)
        .unwrap();
    assert_eq!(file.load_snapshot().unwrap().catalog.revisions().len(), 3);
}

#[test]
fn failed_io_publication_and_invalid_reload_leave_observed_state_intact() {
    let directory = tempfile::tempdir().unwrap();
    let file = LibraryFile::new(directory.path().join("missing/library.json"));
    let mut publisher = LibraryPublisher::new(file.clone());
    assert!(
        publisher
            .publish(revision(1), Recommendation::RecommendPublished)
            .is_err()
    );
    assert_eq!(publisher.catalog().revisions().len(), 0);
    fs::create_dir(directory.path().join("missing")).unwrap();
    publisher
        .publish(revision(1), Recommendation::RecommendPublished)
        .unwrap();
    let before = publisher.catalog().clone();
    let bytes = fs::read(file.path()).unwrap();
    fs::write(file.path(), b"invalid").unwrap();
    assert!(publisher.reload().is_err());
    assert!(
        publisher
            .publish(revision(2), Recommendation::RecommendPublished)
            .is_err()
    );
    assert_eq!(publisher.catalog(), &before);
    fs::write(file.path(), bytes).unwrap();
    publisher
        .publish(revision(2), Recommendation::RecommendPublished)
        .unwrap();
    fs::remove_file(file.path()).unwrap();
    assert!(matches!(
        publisher.publish(revision(3), Recommendation::KeepCurrent),
        Err(LibraryFileError::ExternalChange(_))
    ));
}

#[test]
fn parent_path_aliases_share_version_and_lock_location() {
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir(directory.path().join("nested")).unwrap();
    let first = LibraryFile::new(directory.path().join("library.json"));
    let alias = LibraryFile::new(directory.path().join("nested/../library.json"));
    let version = first.save_if_unchanged(&catalog(), None).unwrap();
    assert_eq!(alias.load_snapshot().unwrap().version, version);
    assert_eq!(alias.lock_path().unwrap(), first.lock_path().unwrap());
}

#[cfg(unix)]
#[test]
fn destination_symlinks_are_rejected_without_replacing_link_or_target() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("real.json");
    let target = LibraryFile::new(&path);
    target.save_if_unchanged(&catalog(), None).unwrap();
    let bytes = fs::read(&path).unwrap();
    let link = directory.path().join("alias.json");
    std::os::unix::fs::symlink(&path, &link).unwrap();
    let alias = LibraryFile::new(&link);
    assert!(alias.load_snapshot().is_err());
    assert!(alias.save_if_unchanged(&catalog(), None).is_err());
    assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    assert_eq!(fs::read(path).unwrap(), bytes);
}

#[test]
fn malformed_electrical_bodies_and_nested_versions_cannot_be_published_from_disk() {
    let data: Value = serde_json::from_str(&encode_library(&catalog()).unwrap()).unwrap();
    let mut future = data.clone();
    future["revisions"][0]["body"]["schema_version"] = json!(99);
    assert!(decode_library(&future.to_string()).is_err());
    let mut dangling = data.clone();
    dangling["revisions"][0]["body"]["conductors"][0]["a"] =
        json!("00000000000000000000000000000099");
    assert!(decode_library(&dangling.to_string()).is_err());
    let mut malformed = data;
    malformed["revisions"][0]["definition"] = json!("not-an-id");
    assert!(decode_library(&malformed.to_string()).is_err());
}
