//! Real-file template publication, strict catalogues, closed revisions and failure recovery.
use ecad_adapters::*;
use ecad_app::*;
use ecad_domain::*;
use std::{fs, sync::Arc};
fn pin(revision: u128) -> TemplateRevisionRef {
    TemplateRevisionRef {
        definition: TemplateDefinitionId::new(1).unwrap(),
        revision: TemplateRevisionId::new(revision).unwrap(),
    }
}
fn variant(revision: u128, id: u128, name: &str) -> SavedTemplateVariant {
    SavedTemplateVariant::new(
        TemplateVariantRef {
            definition: pin(revision).definition,
            revision: pin(revision).revision,
            variant: TemplateVariantId::new(id).unwrap(),
        },
        name.into(),
        SheetTemplate::StandardV1
            .layout(&SheetPaper::default())
            .unwrap(),
    )
    .unwrap()
}
fn revision(id: u128) -> Arc<TemplateRevision> {
    Arc::new(
        TemplateRevision::new(
            pin(id),
            vec![
                variant(id, 2, "A4 landscape"),
                variant(id, 1, "Second saved variant"),
            ],
        )
        .unwrap(),
    )
}
#[test]
fn complete_revisions_reject_foreign_duplicate_empty_and_in_place_variant_additions() {
    assert!(matches!(
        TemplateRevision::new(pin(1), vec![]),
        Err(TemplateLibraryError::EmptyRevision(_))
    ));
    assert!(matches!(
        TemplateRevision::new(pin(1), vec![variant(2, 1, "Foreign")]),
        Err(TemplateLibraryError::ForeignVariant(_))
    ));
    let v = variant(1, 1, "First");
    assert!(matches!(
        TemplateRevision::new(pin(1), vec![v.clone(), v.clone()]),
        Err(TemplateLibraryError::DuplicateVariant(_))
    ));
    let mut catalog = TemplateLibrary::default();
    let body = Arc::new(TemplateRevision::new(pin(1), vec![v.clone()]).unwrap());
    assert!(catalog.publish(body.clone()).unwrap());
    assert!(!catalog.publish(body).unwrap());
    let snapshot = catalog.clone();
    let extended =
        Arc::new(TemplateRevision::new(pin(1), vec![v, variant(1, 2, "Added later")]).unwrap());
    assert_eq!(
        catalog.publish(extended),
        Err(TemplateLibraryError::RevisionConflict(pin(1)))
    );
    assert_eq!(catalog, snapshot);
    assert!(matches!(
        catalog.set_recommended(pin(1).definition, Some(pin(2))),
        Err(TemplateLibraryError::MissingRevision(_))
    ));
    let other = TemplateDefinitionId::new(9).unwrap();
    assert!(matches!(
        catalog.set_recommended(other, Some(pin(1))),
        Err(TemplateLibraryError::WrongLineage(_, _))
    ));
    assert_eq!(catalog, snapshot);
}
#[test]
fn published_revisions_reopen_and_recommendations_never_retarget_exact_variants() {
    let dir = tempfile::tempdir().unwrap();
    let store = TemplateLibraryFile::new(dir.path().join("Drawing templates.json"));
    let mut publisher = TemplatePublisher::new(store.clone());
    assert!(!store.path().exists());
    let earlier = revision(1);
    let saved_pin = earlier.variants().next().unwrap().reference();
    let saved_body = earlier.variant(saved_pin.variant).unwrap().clone();
    assert!(
        publisher
            .publish(earlier.clone(), Recommendation::RecommendPublished)
            .unwrap()
    );
    assert!(
        !publisher
            .publish(earlier, Recommendation::KeepCurrent)
            .unwrap()
    );
    publisher
        .publish(revision(2), Recommendation::KeepCurrent)
        .unwrap();
    assert_eq!(
        publisher.catalog().recommended(pin(1).definition),
        Some(pin(1))
    );
    publisher
        .set_recommended(pin(1).definition, Some(pin(2)))
        .unwrap();
    assert_eq!(publisher.catalog().variant(saved_pin), Some(&saved_body));
    let reopened = TemplatePublisher::open(store.clone()).unwrap();
    assert_eq!(reopened.catalog(), publisher.catalog());
    publisher
        .set_recommended(pin(1).definition, Some(pin(1)))
        .unwrap();
    publisher.set_recommended(pin(1).definition, None).unwrap();
    assert_eq!(publisher.catalog().revisions().len(), 2);
    assert_eq!(publisher.catalog().recommended(pin(1).definition), None);
    assert!(!store.lock_path().unwrap().exists());
}
#[test]
fn stale_publications_and_noops_preserve_local_state_until_explicit_reload() {
    let dir = tempfile::tempdir().unwrap();
    let store = TemplateLibraryFile::new(dir.path().join("templates.json"));
    let mut first = TemplatePublisher::new(store.clone());
    first
        .publish(revision(1), Recommendation::RecommendPublished)
        .unwrap();
    let mut stale = TemplatePublisher::open(store.clone()).unwrap();
    let observed = stale.catalog().clone();
    first
        .publish(revision(2), Recommendation::RecommendPublished)
        .unwrap();
    let bytes = fs::read(store.path()).unwrap();
    assert!(matches!(
        stale.publish(revision(3), Recommendation::RecommendPublished),
        Err(TemplateLibraryFileError::ExternalChange(_))
    ));
    assert!(matches!(
        stale.publish(revision(1), Recommendation::KeepCurrent),
        Err(TemplateLibraryFileError::ExternalChange(_))
    ));
    assert!(matches!(
        stale.set_recommended(pin(1).definition, None),
        Err(TemplateLibraryFileError::ExternalChange(_))
    ));
    assert_eq!(stale.catalog(), &observed);
    assert_eq!(fs::read(store.path()).unwrap(), bytes);
    stale.reload().unwrap();
    stale
        .publish(revision(3), Recommendation::KeepCurrent)
        .unwrap();
    assert_eq!(stale.catalog().recommended(pin(1).definition), Some(pin(2)));
    assert_eq!(stale.catalog().revisions().len(), 3);
    // An immutable conflict rejects before storage and retains local published state.
    let changed = Arc::new(TemplateRevision::new(pin(1), vec![variant(1, 1, "Changed")]).unwrap());
    let observed = stale.catalog().clone();
    assert!(matches!(
        stale.publish(changed, Recommendation::RecommendPublished),
        Err(TemplateLibraryFileError::Catalog(
            TemplateLibraryError::RevisionConflict(_)
        ))
    ));
    assert_eq!(stale.catalog(), &observed);
}
#[test]
fn file_boundary_rejects_history_removal_rewrites_and_extending_a_saved_revision() {
    let dir = tempfile::tempdir().unwrap();
    let store = TemplateLibraryFile::new(dir.path().join("templates.json"));
    let mut publisher = TemplatePublisher::new(store.clone());
    publisher
        .publish(revision(1), Recommendation::RecommendPublished)
        .unwrap();
    let observed = store.load_snapshot().unwrap();
    let bytes = fs::read(store.path()).unwrap();
    let mut changed = TemplateLibrary::default();
    changed
        .publish(Arc::new(
            TemplateRevision::new(pin(1), vec![variant(1, 1, "Changed")]).unwrap(),
        ))
        .unwrap();
    let mut extended = TemplateLibrary::default();
    let mut variants: Vec<_> = revision(1).variants().cloned().collect();
    variants.push(variant(1, 3, "Added"));
    extended
        .publish(Arc::new(TemplateRevision::new(pin(1), variants).unwrap()))
        .unwrap();
    for candidate in [TemplateLibrary::default(), changed, extended] {
        assert!(matches!(
            store.save_if_unchanged(&candidate, Some(&observed.version)),
            Err(TemplateLibraryFileError::HistoryRewrite(_))
        ));
        assert_eq!(fs::read(store.path()).unwrap(), bytes);
        assert!(!store.lock_path().unwrap().exists());
    }
}
#[test]
fn retained_locks_creation_conflicts_and_location_tokens_cannot_authorize_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    let store = TemplateLibraryFile::new(dir.path().join("templates.json"));
    let lock = store.lock_path().unwrap();
    fs::write(&lock, "retained writer").unwrap();
    let mut publisher = TemplatePublisher::new(store.clone());
    assert!(matches!(
        publisher.publish(revision(1), Recommendation::RecommendPublished),
        Err(TemplateLibraryFileError::Busy(_))
    ));
    assert_eq!(fs::read_to_string(&lock).unwrap(), "retained writer");
    assert!(!store.path().exists());
    assert_eq!(publisher.catalog().revisions().len(), 0);
    // Test owns this simulated lock and explicitly releases it; the adapter never steals it.
    fs::remove_file(&lock).unwrap();
    publisher
        .publish(revision(1), Recommendation::RecommendPublished)
        .unwrap();
    let mut unobserved = TemplatePublisher::new(store.clone());
    assert!(matches!(
        unobserved.publish(revision(2), Recommendation::KeepCurrent),
        Err(TemplateLibraryFileError::ExternalChange(_))
    ));
    let observed = store.load_snapshot().unwrap();
    let copy = TemplateLibraryFile::new(dir.path().join("copy.json"));
    fs::copy(store.path(), copy.path()).unwrap();
    assert!(matches!(
        copy.save_if_unchanged(&observed.catalog, Some(&observed.version)),
        Err(TemplateLibraryFileError::ExternalChange(_))
    ));
    let alias = TemplateLibraryFile::new(dir.path().join(".").join("templates.json"));
    assert_eq!(
        alias
            .save_if_unchanged(&observed.catalog, Some(&observed.version))
            .unwrap(),
        observed.version
    );
}
#[test]
fn failed_reload_missing_parent_and_deleted_destination_preserve_observations() {
    let dir = tempfile::tempdir().unwrap();
    let store = TemplateLibraryFile::new(dir.path().join("templates.json"));
    assert!(TemplatePublisher::open(store.clone()).is_err());
    assert!(!store.path().exists());
    let mut publisher = TemplatePublisher::new(store.clone());
    publisher
        .publish(revision(1), Recommendation::KeepCurrent)
        .unwrap();
    let observed = publisher.catalog().clone();
    fs::write(store.path(), "broken").unwrap();
    assert!(publisher.reload().is_err());
    assert_eq!(publisher.catalog(), &observed);
    assert!(matches!(
        publisher.publish(revision(2), Recommendation::KeepCurrent),
        Err(TemplateLibraryFileError::ExternalChange(_))
    ));
    fs::remove_file(store.path()).unwrap();
    assert!(publisher.reload().is_err());
    assert!(matches!(
        publisher.publish(revision(2), Recommendation::KeepCurrent),
        Err(TemplateLibraryFileError::ExternalChange(_))
    ));
    assert_eq!(publisher.catalog(), &observed);
    let store = TemplateLibraryFile::new(dir.path().join("missing").join("templates.json"));
    let mut failed = TemplatePublisher::new(store);
    assert!(
        failed
            .publish(revision(1), Recommendation::RecommendPublished)
            .is_err()
    );
    assert_eq!(failed.catalog(), &TemplateLibrary::default());
}
#[test]
fn canonical_catalogue_roundtrips_and_strict_nested_data_is_never_dropped() {
    let fixture = include_str!("fixtures/template-library-v1.json");
    let catalog = decode_template_library(fixture).unwrap();
    assert_eq!(encode_template_library(&catalog).unwrap(), fixture);
    let mut full = TemplateLibrary::default();
    full.publish(revision(2)).unwrap();
    full.publish(revision(1)).unwrap();
    let text = encode_template_library(&full).unwrap();
    assert_eq!(decode_template_library(&text).unwrap(), full);
    let base: serde_json::Value = serde_json::from_str(&text).unwrap();
    for path in ["root", "revision", "variant"] {
        let mut json = base.clone();
        let target = match path {
            "root" => &mut json,
            "revision" => &mut json["revisions"][0],
            _ => &mut json["revisions"][0]["variants"][0],
        };
        target["unknown"] = true.into();
        assert!(
            decode_template_library(&json.to_string()).is_err(),
            "{path}"
        );
    }
    let mut json = base.clone();
    json["kind"] = "ecad-subsystem-library".into();
    assert!(decode_template_library(&json.to_string()).is_err());
    let mut json = base.clone();
    json["schema_version"] = 99.into();
    assert!(decode_template_library(&json.to_string()).is_err());
    let mut json = base.clone();
    json["revisions"][0]["variants"][0]["schema_version"] = 99.into();
    assert!(decode_template_library(&json.to_string()).is_err());
    let mut json = base.clone();
    json["revisions"][0]["variants"][0]["definition"] =
        TemplateDefinitionId::new(9).unwrap().to_string().into();
    assert!(matches!(
        decode_template_library(&json.to_string()),
        Err(TemplateLibraryFileError::Catalog(
            TemplateLibraryError::ForeignVariant(_)
        ))
    ));
    let mut json = base;
    let extra = json["revisions"][0].clone();
    json["revisions"].as_array_mut().unwrap().push(extra);
    assert!(matches!(
        decode_template_library(&json.to_string()),
        Err(TemplateLibraryFileError::Catalog(
            TemplateLibraryError::DuplicateRevision(_)
        ))
    ));
}

#[test]
fn duplicate_and_dangling_recommendations_are_rejected_before_catalogue_escape() {
    let mut json: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/template-library-v1.json")).unwrap();
    let entry = json["recommended"][0].clone();
    json["recommended"].as_array_mut().unwrap().push(entry);
    assert!(matches!(
        decode_template_library(&json.to_string()),
        Err(TemplateLibraryFileError::Catalog(
            TemplateLibraryError::DuplicateRecommendation(_)
        ))
    ));
    json["recommended"].as_array_mut().unwrap().pop();
    json["recommended"][0]["revision"] = TemplateRevisionId::new(99).unwrap().to_string().into();
    assert!(matches!(
        decode_template_library(&json.to_string()),
        Err(TemplateLibraryFileError::Catalog(
            TemplateLibraryError::MissingRevision(_)
        ))
    ));
    let mut json: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/template-library-v1.json")).unwrap();
    let variant = json["revisions"][0]["variants"][0].clone();
    json["revisions"][0]["variants"]
        .as_array_mut()
        .unwrap()
        .push(variant);
    assert!(matches!(
        decode_template_library(&json.to_string()),
        Err(TemplateLibraryFileError::Catalog(
            TemplateLibraryError::DuplicateVariant(_)
        ))
    ));
}
#[test]
fn two_cooperating_creators_cannot_overwrite_each_other() {
    let dir = tempfile::tempdir().unwrap();
    let store = TemplateLibraryFile::new(dir.path().join("templates.json"));
    let barrier = Arc::new(std::sync::Barrier::new(3));
    let handles: Vec<_> = (1..=2)
        .map(|id| {
            let store = store.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut publisher = TemplatePublisher::new(store);
                barrier.wait();
                publisher.publish(revision(id), Recommendation::RecommendPublished)
            })
        })
        .collect();
    barrier.wait();
    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Ok(true)))
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(
                result,
                Err(TemplateLibraryFileError::Busy(_) | TemplateLibraryFileError::ExternalChange(_))
            ))
            .count(),
        1
    );
    assert_eq!(store.load_snapshot().unwrap().catalog.revisions().len(), 1);
    assert!(!store.lock_path().unwrap().exists());
}
#[cfg(unix)]
#[test]
fn destination_symlinks_are_rejected_and_parent_aliases_share_the_lock() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let store = TemplateLibraryFile::new(dir.path().join("templates.json"));
    let mut publisher = TemplatePublisher::new(store.clone());
    publisher
        .publish(revision(1), Recommendation::KeepCurrent)
        .unwrap();
    let linked = TemplateLibraryFile::new(dir.path().join("linked.json"));
    symlink(store.path(), linked.path()).unwrap();
    assert!(matches!(
        linked.load_snapshot(),
        Err(TemplateLibraryFileError::Io(_))
    ));
    assert!(matches!(
        linked.save_if_unchanged(publisher.catalog(), None),
        Err(TemplateLibraryFileError::Io(_))
    ));
    let alias = tempfile::tempdir().unwrap();
    symlink(dir.path(), alias.path().join("parent")).unwrap();
    let aliased = TemplateLibraryFile::new(alias.path().join("parent").join("templates.json"));
    assert_eq!(aliased.lock_path().unwrap(), store.lock_path().unwrap());
    fs::write(store.lock_path().unwrap(), "retained").unwrap();
    let mut aliased = TemplatePublisher::open(aliased).unwrap();
    assert!(matches!(
        aliased.publish(revision(2), Recommendation::KeepCurrent),
        Err(TemplateLibraryFileError::Busy(_))
    ));
    fs::remove_file(store.lock_path().unwrap()).unwrap();
}
