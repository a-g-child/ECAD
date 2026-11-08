//! Independent template drafts, exact history and publication checkpoints. No electrical Project is created.
use crate::{Recommendation, Revision, TemplatePublisher, VersionedTemplateLibraryStore};
use ecad_domain::*;
use std::{collections::BTreeMap, error::Error, fmt, sync::Arc};
/// One validated draft variant. Published identities are kept outside editable history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateDraftVariant {
    name: String,
    layout: TemplateLayout,
    retained: bool,
}
impl TemplateDraftVariant {
    /// Literal multiline library name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Retained exact paper, content region and table structure.
    pub fn layout(&self) -> &TemplateLayout {
        &self.layout
    }
    /// Whether an omitted legacy block still requires repair before new assignment.
    pub fn retained_legacy(&self) -> bool {
        self.retained
    }
}
#[derive(Clone)]
struct Change {
    id: TemplateVariantId,
    before: TemplateDraftVariant,
    after: TemplateDraftVariant,
    before_state: u64,
    after_state: u64,
}
/// A template editor session with its own epoch, history and saved checkpoint.
/// Editable bodies never enter assignment catalogues; publication builds a fresh complete revision.
pub struct TemplateDocument {
    revision: Revision,
    definition: TemplateDefinitionId,
    variants: BTreeMap<TemplateVariantId, TemplateDraftVariant>,
    published: Option<TemplateRevisionRef>,
    undo: Vec<Change>,
    redo: Vec<Change>,
    state: u64,
    next_state: u64,
    saved_state: Option<u64>,
}
impl TemplateDocument {
    /// Edit a complete saved family without changing it. Opening is clean and performs no IO.
    pub fn open(body: &TemplateRevision) -> Self {
        let variants = body
            .variants()
            .map(|v| {
                (
                    v.reference().variant,
                    TemplateDraftVariant {
                        name: v.name().into(),
                        layout: v.layout().clone(),
                        retained: !v.allows_new_assignments(),
                    },
                )
            })
            .collect();
        Self {
            revision: Revision::fresh(),
            definition: body.reference().definition,
            variants,
            published: Some(body.reference()),
            undo: vec![],
            redo: vec![],
            state: 0,
            next_state: 1,
            saved_state: Some(0),
        }
    }
    /// Start a new unsaved definition from a validated layout, without a project or hidden sheet.
    /// # Errors
    /// Rejects blank names. The supplied definition/variant identities must be newly allocated.
    pub fn new(
        definition: TemplateDefinitionId,
        variant: TemplateVariantId,
        name: String,
        layout: TemplateLayout,
    ) -> Result<Self, TemplateEditError> {
        if name.trim().is_empty() {
            return Err(TemplateEditError::BlankName);
        }
        Ok(Self {
            revision: Revision::fresh(),
            definition,
            variants: [(
                variant,
                TemplateDraftVariant {
                    name,
                    layout,
                    retained: false,
                },
            )]
            .into(),
            published: None,
            undo: vec![],
            redo: vec![],
            state: 0,
            next_state: 1,
            saved_state: None,
        })
    }
    /// Captured editor epoch/revision, independent of every project session.
    pub fn revision(&self) -> Revision {
        self.revision
    }
    /// Stable reusable definition lineage.
    pub fn definition(&self) -> TemplateDefinitionId {
        self.definition
    }
    /// Last successful published pin, separate from unsaved edits and undo.
    pub fn published(&self) -> Option<TemplateRevisionRef> {
        self.published
    }
    /// Whether editable state differs from its successful publication checkpoint.
    pub fn is_dirty(&self) -> bool {
        self.saved_state != Some(self.state)
    }
    /// Retained undo transaction count (bounded at 256).
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }
    /// Retained redo transaction count.
    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }
    /// Draft variants in stable identity order; these are not saved catalogue entries.
    pub fn variants(
        &self,
    ) -> impl ExactSizeIterator<Item = (TemplateVariantId, &TemplateDraftVariant)> {
        self.variants.iter().map(|(id, v)| (*id, v))
    }
    /// Inspect one draft variant.
    pub fn variant(&self, id: TemplateVariantId) -> Option<&TemplateDraftVariant> {
        self.variants.get(&id)
    }
    /// Commit one complete validated variant edit. Equal state preserves redo/checkpoints.
    /// # Errors
    /// Rejects stale tokens, missing variants, blank names or exhausted revision/state counters.
    pub fn replace_variant(
        &mut self,
        expected: Revision,
        id: TemplateVariantId,
        name: String,
        layout: TemplateLayout,
    ) -> Result<bool, TemplateEditError> {
        self.check(expected)?;
        if name.trim().is_empty() {
            return Err(TemplateEditError::BlankName);
        }
        let before = self
            .variants
            .get(&id)
            .ok_or(TemplateEditError::MissingVariant(id))?
            .clone();
        let retained = before.retained && layout.tables().is_empty();
        let after = TemplateDraftVariant {
            name,
            layout,
            retained,
        };
        if before == after {
            return Ok(false);
        }
        let revision = self.revision.next().ok_or(TemplateEditError::Exhausted)?;
        let next = self
            .next_state
            .checked_add(1)
            .ok_or(TemplateEditError::Exhausted)?;
        let change = Change {
            id,
            before,
            after: after.clone(),
            before_state: self.state,
            after_state: self.next_state,
        };
        self.variants.insert(id, after);
        self.state = self.next_state;
        self.next_state = next;
        self.revision = revision;
        self.undo.push(change);
        self.redo.clear();
        if self.undo.len() > 256 {
            self.undo.remove(0);
        }
        Ok(true)
    }
    /// Undo exact variant state without retracting shared publication.
    /// # Errors
    /// Rejects stale/exhausted revisions before changing history.
    pub fn undo(&mut self, expected: Revision) -> Result<bool, TemplateEditError> {
        self.check(expected)?;
        if self.undo.is_empty() {
            return Ok(false);
        }
        let revision = self.revision.next().ok_or(TemplateEditError::Exhausted)?;
        let change = self.undo.pop().expect("checked history");
        self.variants.insert(change.id, change.before.clone());
        self.state = change.before_state;
        self.redo.push(change);
        self.revision = revision;
        Ok(true)
    }
    /// Redo exact variant state, including the saved checkpoint when applicable.
    /// # Errors
    /// Rejects stale/exhausted revisions before changing history.
    pub fn redo(&mut self, expected: Revision) -> Result<bool, TemplateEditError> {
        self.check(expected)?;
        if self.redo.is_empty() {
            return Ok(false);
        }
        let revision = self.revision.next().ok_or(TemplateEditError::Exhausted)?;
        let change = self.redo.pop().expect("checked history");
        self.variants.insert(change.id, change.after.clone());
        self.state = change.after_state;
        self.undo.push(change);
        self.revision = revision;
        Ok(true)
    }
    /// Publish a complete immutable revision, then mark exactly this state clean.
    /// Clean saves verify the existing pin through the publisher. Changed drafts require a fresh
    /// revision identity; failures retain draft/history/checkpoint and previous published pin.
    /// # Errors
    /// Returns validation/pin errors or the store's original conflict/IO failure.
    pub fn save<S: VersionedTemplateLibraryStore>(
        &mut self,
        expected: Revision,
        new_id: TemplateRevisionId,
        publisher: &mut TemplatePublisher<S>,
    ) -> Result<TemplateRevisionRef, TemplateDocumentError<S::Error>> {
        self.check(expected).map_err(TemplateDocumentError::Edit)?;
        let pin = if !self.is_dirty() {
            self.published.expect("clean state has publication")
        } else {
            if self.published.is_some_and(|p| p.revision == new_id) {
                return Err(TemplateDocumentError::Edit(
                    TemplateEditError::PublishedIdentity,
                ));
            }
            TemplateRevisionRef {
                definition: self.definition,
                revision: new_id,
            }
        };
        let mut variants = Vec::new();
        for (id, v) in &self.variants {
            let reference = TemplateVariantRef {
                definition: pin.definition,
                revision: pin.revision,
                variant: *id,
            };
            let build = if v.retained {
                SavedTemplateVariant::preserve_legacy
            } else {
                SavedTemplateVariant::new
            };
            variants.push(
                build(reference, v.name.clone(), v.layout.clone())
                    .map_err(|e| TemplateDocumentError::Edit(TemplateEditError::Layout(e)))?,
            );
        }
        let body = TemplateRevision::new(pin, variants)
            .map_err(|e| TemplateDocumentError::Edit(TemplateEditError::Catalog(e)))?;
        publisher
            .publish(Arc::new(body), Recommendation::KeepCurrent)
            .map_err(TemplateDocumentError::Store)?;
        self.published = Some(pin);
        self.saved_state = Some(self.state);
        Ok(pin)
    }
    pub(crate) fn check(&self, expected: Revision) -> Result<(), TemplateEditError> {
        if expected != self.revision {
            Err(TemplateEditError::Stale)
        } else {
            Ok(())
        }
    }
}
/// Template editing rejected without changing the draft/history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateEditError {
    /// Editor epoch/revision is stale or belongs to another editor.
    Stale,
    /// A draft variant is unavailable.
    MissingVariant(TemplateVariantId),
    /// Names must contain visible text.
    BlankName,
    /// History/revision counter cannot advance.
    Exhausted,
    /// Changed content cannot reuse its published revision identity.
    PublishedIdentity,
    /// Invalid geometry/content.
    Layout(TemplateError),
    /// Invalid complete revision.
    Catalog(TemplateLibraryError),
}
impl fmt::Display for TemplateEditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stale => f.write_str("The template editor changed; refresh before applying"),
            Self::MissingVariant(id) => write!(f, "Template variant {id} is unavailable"),
            Self::BlankName => f.write_str("Template name must not be blank"),
            Self::Exhausted => f.write_str("Template history counter exhausted"),
            Self::PublishedIdentity => {
                f.write_str("Changed templates require a new saved revision")
            }
            Self::Layout(e) => e.fmt(f),
            Self::Catalog(e) => e.fmt(f),
        }
    }
}
impl Error for TemplateEditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Layout(e) => Some(e),
            Self::Catalog(e) => Some(e),
            _ => None,
        }
    }
}
/// A failed template publication, retaining the original typed failure.
#[derive(Debug)]
pub enum TemplateDocumentError<E> {
    /// Draft/revision validation failed.
    Edit(TemplateEditError),
    /// Shared library persistence failed.
    Store(E),
}
impl<E: fmt::Display> fmt::Display for TemplateDocumentError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(e) => e.fmt(f),
            Self::Store(e) => e.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for TemplateDocumentError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(e) => Some(e),
            Self::Store(e) => Some(e),
        }
    }
}
