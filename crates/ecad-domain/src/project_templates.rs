//! Project-owned immutable template bodies, separate from shared publication and electrical reuse.
use crate::{Project, TemplateLibraryError, TemplateRevision, TemplateRevisionRef};
use std::sync::Arc;
impl Project {
    /// Bundled complete revisions in stable identity order, available without a library path.
    pub fn template_revisions(&self) -> impl ExactSizeIterator<Item = &TemplateRevision> {
        self.template_revisions.values().map(Arc::as_ref)
    }
    /// Resolve an exact bundled revision; never follows a library recommendation.
    pub fn template_revision(&self, pin: TemplateRevisionRef) -> Option<&TemplateRevision> {
        self.template_revisions.get(&pin).map(Arc::as_ref)
    }
    /// Bundle a complete immutable revision; identical imports are a no-op.
    /// # Errors
    /// Rejects different content under an existing pin without changing project state.
    pub fn insert_template_revision(
        &mut self,
        revision: Arc<TemplateRevision>,
    ) -> Result<bool, TemplateLibraryError> {
        let pin = revision.reference();
        if let Some(previous) = self.template_revisions.get(&pin) {
            if previous == &revision {
                return Ok(false);
            }
            return Err(TemplateLibraryError::RevisionConflict(pin));
        }
        self.template_revisions.insert(pin, revision);
        Ok(true)
    }
    /// Remove one bundled body and return its exact state for undo. Current enum-based
    /// sheets do not reference these bodies; assignment-reference guards must land with
    /// saved-template assignment before any such references can be created.
    /// # Errors
    /// Rejects a missing pin without mutation.
    pub fn remove_template_revision(
        &mut self,
        pin: TemplateRevisionRef,
    ) -> Result<Arc<TemplateRevision>, TemplateLibraryError> {
        self.template_revisions
            .remove(&pin)
            .ok_or(TemplateLibraryError::MissingRevision(pin))
    }
}
