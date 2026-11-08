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
    /// Remove an unused bundled body and return its exact state for undo.
    /// # Errors
    /// Rejects missing revisions or any reference from creation defaults, document defaults
    /// (including inherited sheets), or explicit sheet overrides, without mutation.
    pub fn remove_template_revision(
        &mut self,
        pin: TemplateRevisionRef,
    ) -> Result<Arc<TemplateRevision>, TemplateLibraryError> {
        let used = |choice: crate::SheetTemplate| matches!(choice,crate::SheetTemplate::Saved(p) if p.revision_ref()==pin);
        if [
            self.template_defaults.schematic,
            self.template_defaults.cover,
            self.template_defaults.drawing_list,
        ]
        .into_iter()
        .any(used)
            || self
                .drawing_documents
                .iter()
                .any(|d| used(d.default_template))
            || self.sheets.iter().filter_map(|s| s.template).any(used)
        {
            return Err(TemplateLibraryError::InUse(pin));
        }
        self.template_revisions
            .remove(&pin)
            .ok_or(TemplateLibraryError::MissingRevision(pin))
    }
}
