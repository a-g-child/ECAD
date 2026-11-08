//! Durable template publication, separate from electrical project history and draft editors.
use crate::Recommendation;
use ecad_domain::{
    TemplateDefinitionId, TemplateLibrary, TemplateLibraryError, TemplateRevision,
    TemplateRevisionRef,
};
use std::sync::Arc;

/// Validated complete catalogue and its adapter-owned observed destination/version token.
pub struct TemplateLibrarySnapshot<V> {
    /// Published immutable revisions and explicit recommendations.
    pub catalog: TemplateLibrary,
    /// Exact observed location/content version.
    pub version: V,
}
/// IO boundary for coherent template publication, independent of project document saves.
pub trait VersionedTemplateLibraryStore {
    /// Opaque destination/version token.
    type Version: Clone + Eq;
    /// Storage/format/catalogue error, preserving its source.
    type Error: std::error::Error + Send + Sync + From<TemplateLibraryError> + 'static;
    /// Load a complete validated catalogue and observe its version.
    /// # Errors
    /// Reports missing/unreadable/invalid stores without exposing partial state.
    fn load_snapshot(&self) -> Result<TemplateLibrarySnapshot<Self::Version>, Self::Error>;
    /// Save only against the observed version; None permits creation only. Implementations
    /// retain all published bodies and document writer coordination/durability guarantees.
    /// # Errors
    /// Reports external changes, history rewrites, invalid catalogues and IO failures.
    fn save_if_unchanged(
        &self,
        catalog: &TemplateLibrary,
        expected: Option<&Self::Version>,
    ) -> Result<Self::Version, Self::Error>;
}
/// Publishes complete template revisions outside project undo. In-memory state and its
/// observed version change only after successful storage, including recommendation updates.
pub struct TemplatePublisher<S: VersionedTemplateLibraryStore> {
    catalog: TemplateLibrary,
    store: S,
    version: Option<S::Version>,
}
impl<S: VersionedTemplateLibraryStore> TemplatePublisher<S> {
    /// Prepare an empty catalogue for a missing destination without IO.
    pub fn new(store: S) -> Self {
        Self {
            catalog: TemplateLibrary::default(),
            store,
            version: None,
        }
    }
    /// Observe an existing validated catalogue.
    /// # Errors
    /// Returns storage/validation errors without partial publisher state.
    pub fn open(store: S) -> Result<Self, S::Error> {
        let snapshot = store.load_snapshot()?;
        Ok(Self {
            catalog: snapshot.catalog,
            store,
            version: Some(snapshot.version),
        })
    }
    /// Inspect successfully published content; callers cannot mutate it in place.
    pub fn catalog(&self) -> &TemplateLibrary {
        &self.catalog
    }
    /// Publish one complete revision, optionally recommending it in the same save.
    /// Even identical re-publication checks the observed storage version.
    /// # Errors
    /// Failed/conflicting saves or immutable-key conflicts retain the previous state/version.
    pub fn publish(
        &mut self,
        revision: Arc<TemplateRevision>,
        recommendation: Recommendation,
    ) -> Result<bool, S::Error> {
        let pin = revision.reference();
        let mut candidate = self.catalog.clone();
        let mut changed = candidate.publish(revision)?;
        if recommendation == Recommendation::RecommendPublished {
            changed |= candidate.set_recommended(pin.definition, Some(pin))?;
        }
        self.persist(candidate)?;
        Ok(changed)
    }
    /// Explicitly change/clear a recommendation; earlier revisions remain available.
    /// # Errors
    /// Invalid targets and failed saves preserve the existing catalogue/version.
    pub fn set_recommended(
        &mut self,
        definition: TemplateDefinitionId,
        pin: Option<TemplateRevisionRef>,
    ) -> Result<bool, S::Error> {
        let mut candidate = self.catalog.clone();
        let changed = candidate.set_recommended(definition, pin)?;
        self.persist(candidate)?;
        Ok(changed)
    }
    /// Explicitly reload the complete store. Failed reloads retain previous observations.
    /// # Errors
    /// Reports missing/unreadable/invalid replacements without changing this publisher.
    pub fn reload(&mut self) -> Result<(), S::Error> {
        let snapshot = self.store.load_snapshot()?;
        self.catalog = snapshot.catalog;
        self.version = Some(snapshot.version);
        Ok(())
    }
    fn persist(&mut self, candidate: TemplateLibrary) -> Result<(), S::Error> {
        let version = self
            .store
            .save_if_unchanged(&candidate, self.version.as_ref())?;
        self.catalog = candidate;
        self.version = Some(version);
        Ok(())
    }
}

/// Prepare an explicit template import through ordinary project undo. Publication,
/// project bundling and sheet assignment are distinct operations.
/// # Errors
/// Rejects a revision missing from this catalogue. Key conflicts are checked on execution.
pub fn template_library_import(
    catalog: &TemplateLibrary,
    pin: TemplateRevisionRef,
) -> Result<crate::Command, TemplateLibraryError> {
    let revision = catalog
        .revision(pin)
        .ok_or(TemplateLibraryError::MissingRevision(pin))?;
    Ok(crate::Command::InsertTemplateRevision(Arc::new(
        revision.clone(),
    )))
}
