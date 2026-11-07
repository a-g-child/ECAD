//! Persistent library publication; independent of reversible project editing.

use crate::Command;
use ecad_domain::{DefinitionId, DefinitionRef, LibraryError, SubsystemLibrary, SubsystemRevision};
use std::sync::Arc;

/// Validated shared content and the exact storage version it came from.
pub struct LibrarySnapshot<V> {
    /// Immutable revisions and advisory recommendations.
    pub catalog: SubsystemLibrary,
    /// Adapter-owned token identifying the observed destination and bytes.
    pub version: V,
}

/// IO boundary for coherent publication and observed-version replacement.
pub trait VersionedLibraryStore {
    /// Opaque observed destination/version token.
    type Version: Clone + Eq;
    /// Concrete storage or catalog failure, preserving its source.
    type Error: std::error::Error + Send + Sync + From<LibraryError> + 'static;
    /// Load one validated snapshot; never return partially resolved recommendations.
    /// # Errors
    /// Reports missing/unreadable files, invalid catalogs, or unsupported schemas.
    fn load_snapshot(&self) -> Result<LibrarySnapshot<Self::Version>, Self::Error>;
    /// Persist only against the observed version, retaining all published bodies.
    /// `None` permits creation only. Publication and recommendation changes must be
    /// coherent; implementations must state writer-coordination and durability limits.
    /// # Errors
    /// Reports conflicts, attempts to remove/rewrite history, invalid data, or IO failures.
    fn save_if_unchanged(
        &self,
        catalog: &SubsystemLibrary,
        expected: Option<&Self::Version>,
    ) -> Result<Self::Version, Self::Error>;
}

/// Whether publication also changes the recommendation for this lineage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recommendation {
    /// Leave the existing pointer untouched, including when none exists.
    KeepCurrent,
    /// Recommend exactly the revision being published.
    RecommendPublished,
}

/// A shared-library publisher whose state changes only after successful persistence.
/// Publication is external to project undo. To revert a recommendation, explicitly
/// point it at an earlier revision; immutable published bodies remain available.
pub struct LibraryPublisher<S: VersionedLibraryStore> {
    catalog: SubsystemLibrary,
    store: S,
    version: Option<S::Version>,
}
impl<S: VersionedLibraryStore> LibraryPublisher<S> {
    /// Prepare an empty catalog for a missing destination; performs no IO.
    pub fn new(store: S) -> Self {
        Self {
            catalog: SubsystemLibrary::default(),
            store,
            version: None,
        }
    }
    /// Open an existing validated library, observing its exact storage version.
    /// # Errors
    /// Reports the store failure without creating partial publisher state.
    pub fn open(store: S) -> Result<Self, S::Error> {
        let snapshot = store.load_snapshot()?;
        Ok(Self {
            catalog: snapshot.catalog,
            store,
            version: Some(snapshot.version),
        })
    }
    /// Inspect published content without exposing mutable catalog state.
    pub fn catalog(&self) -> &SubsystemLibrary {
        &self.catalog
    }
    /// Publish one immutable body and optionally recommend it in one persisted change.
    /// Returns whether the catalog changed. Even no-ops verify the observed store.
    /// # Errors
    /// Conflicts, immutable-key violations, and IO failures leave local state unchanged.
    pub fn publish(
        &mut self,
        revision: Arc<SubsystemRevision>,
        recommendation: Recommendation,
    ) -> Result<bool, S::Error> {
        let source = revision.source();
        let mut candidate = self.catalog.clone();
        let mut changed = candidate.publish(revision)?;
        if recommendation == Recommendation::RecommendPublished {
            changed |= candidate.set_recommended(source.definition(), Some(source))?;
        }
        self.persist(candidate)?;
        Ok(changed)
    }
    /// Change or clear one recommendation without modifying published bodies.
    /// # Errors
    /// Invalid pointers or failed persistence leave the previous catalog/version intact.
    pub fn set_recommended(
        &mut self,
        lineage: DefinitionId,
        source: Option<DefinitionRef>,
    ) -> Result<bool, S::Error> {
        let mut candidate = self.catalog.clone();
        let changed = candidate.set_recommended(lineage, source)?;
        self.persist(candidate)?;
        Ok(changed)
    }
    /// Refresh from disk explicitly, replacing local state only after validation.
    /// # Errors
    /// Missing/invalid replacements leave the observed catalog and version unchanged.
    pub fn reload(&mut self) -> Result<(), S::Error> {
        let snapshot = self.store.load_snapshot()?;
        self.catalog = snapshot.catalog;
        self.version = Some(snapshot.version);
        Ok(())
    }
    fn persist(&mut self, candidate: SubsystemLibrary) -> Result<(), S::Error> {
        let version = self
            .store
            .save_if_unchanged(&candidate, self.version.as_ref())?;
        self.catalog = candidate;
        self.version = Some(version);
        Ok(())
    }
}

/// Bundle an explicitly chosen library revision through ordinary project history.
/// This imports content only; placement and adoption remain separate commands.
/// # Errors
/// Rejects a revision unavailable in this catalog. Project key conflicts are checked
/// when the returned command executes against the session revision.
pub fn library_import(
    catalog: &SubsystemLibrary,
    source: DefinitionRef,
) -> Result<Command, LibraryError> {
    let revision = catalog
        .revision(source)
        .ok_or(LibraryError::MissingRevision(source))?;
    Ok(Command::InsertDefinition(Arc::new(revision.clone())))
}
