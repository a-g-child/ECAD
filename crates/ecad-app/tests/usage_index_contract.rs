//! Read-boundary and failure-state contracts independent of filesystem adapters.

use ecad_app::{
    KnownUseIndex, ProjectSnapshot, UsageRefreshError, UsageStatus, VersionedProjectStore,
};
use ecad_domain::{Project, ProjectId};
use std::{cell::Cell, error::Error, io};

#[derive(Default)]
struct Source {
    loads: Cell<usize>,
    checks: Cell<usize>,
    fail_load: Cell<bool>,
    fail_check: Cell<bool>,
}
impl VersionedProjectStore for Source {
    type Version = u64;
    type Error = io::Error;
    fn load_snapshot(&self) -> Result<ProjectSnapshot<u64>, io::Error> {
        self.loads.set(self.loads.get() + 1);
        if self.fail_load.get() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid snapshot",
            ));
        }
        Ok(ProjectSnapshot {
            project: Project::new(ProjectId::new(1).unwrap()),
            version: 1,
        })
    }
    fn current_version(&self) -> Result<Option<u64>, io::Error> {
        self.checks.set(self.checks.get() + 1);
        if self.fail_check.get() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "unreadable source",
            ));
        }
        Ok(Some(1))
    }
    fn save_if_unchanged(&self, _: &Project, _: Option<&u64>) -> Result<u64, io::Error> {
        panic!("usage discovery must never call the write boundary")
    }
}

#[test]
fn unregistered_operations_perform_no_io_and_duplicate_registration_preserves_observations() {
    let mut index = KnownUseIndex::default();
    let source = Source::default();
    assert!(matches!(
        index.refresh(&"project", &source),
        Err(UsageRefreshError::Unregistered)
    ));
    assert!(matches!(
        index.check_freshness(&"project", &source),
        Err(UsageRefreshError::Unregistered)
    ));
    assert_eq!(source.loads.get(), 0);
    assert_eq!(source.checks.get(), 0);
    assert!(index.register("project"));
    index.refresh(&"project", &source).unwrap();
    assert_eq!(source.loads.get(), 1);
    assert!(!index.register("project"));
    assert!(index.entry(&"project").unwrap().last_known().is_some());
    assert_eq!(
        index.check_freshness(&"project", &source).unwrap(),
        UsageStatus::MatchesObserved
    );
    assert_eq!(source.loads.get(), 1); // Content checking does not re-load the model.
    assert!(index.unregister(&"project"));
    assert!(!index.unregister(&"project"));
    assert_eq!(index.entries().len(), 0);
}

#[test]
fn read_failures_preserve_last_valid_observation_and_return_original_typed_source() {
    let mut index = KnownUseIndex::default();
    let source = Source::default();
    index.register("project");
    index.refresh(&"project", &source).unwrap();
    let before = index
        .entry(&"project")
        .unwrap()
        .last_known()
        .unwrap()
        .clone();
    source.fail_load.set(true);
    let error = index.refresh(&"project", &source).unwrap_err();
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<io::Error>()
            .is_some()
    );
    assert_eq!(
        index.entry(&"project").unwrap().status(),
        UsageStatus::Unavailable
    );
    assert_eq!(index.entry(&"project").unwrap().last_known(), Some(&before));
    source.fail_check.set(true);
    assert!(index.check_freshness(&"project", &source).is_err());
    assert_eq!(
        index.entry(&"project").unwrap().diagnostic(),
        Some("unreadable source")
    );
    source.fail_load.set(false);
    source.fail_check.set(false);
    index.refresh(&"project", &source).unwrap();
    assert_eq!(index.entry(&"project").unwrap().last_known(), Some(&before));
    assert!(index.entry(&"project").unwrap().diagnostic().is_none());
}

#[test]
fn content_check_cannot_invent_valid_usage_after_a_failed_first_load() {
    let mut index = KnownUseIndex::default();
    let source = Source::default();
    source.fail_load.set(true);
    index.register("project");
    assert!(index.refresh(&"project", &source).is_err());
    assert_eq!(
        index.check_freshness(&"project", &source).unwrap(),
        UsageStatus::Unscanned
    );
    assert!(index.entry(&"project").unwrap().last_known().is_none());
    assert_eq!(index.entries().len(), 1); // Failed coverage remains visible.
}
