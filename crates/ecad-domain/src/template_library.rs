//! Closed template revisions and append-only publication, independent of project editing.
use crate::{
    SavedTemplateVariant, TemplateDefinitionId, TemplateRevisionId, TemplateVariantId,
    TemplateVariantRef,
};
use std::{collections::BTreeMap, error::Error, fmt, sync::Arc};

/// An exact revision pin; no recommendation or implicit latest-version resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TemplateRevisionRef {
    /// Definition lineage.
    pub definition: TemplateDefinitionId,
    /// Immutable complete revision identity.
    pub revision: TemplateRevisionId,
}
impl TemplateVariantRef {
    /// The complete revision containing this exact paper/layout variant.
    pub fn revision_ref(self) -> TemplateRevisionRef {
        TemplateRevisionRef {
            definition: self.definition,
            revision: self.revision,
        }
    }
}
/// A complete immutable set of paper variants. Adding a variant requires a new revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateRevision {
    reference: TemplateRevisionRef,
    variants: BTreeMap<TemplateVariantId, SavedTemplateVariant>,
}
impl TemplateRevision {
    /// Validate the complete revision before publication. Variant order is canonical by ID.
    /// # Errors
    /// Rejects empty revisions, duplicate variant IDs and foreign definition/revision pins.
    pub fn new(
        reference: TemplateRevisionRef,
        variants: Vec<SavedTemplateVariant>,
    ) -> Result<Self, TemplateLibraryError> {
        if variants.is_empty() {
            return Err(TemplateLibraryError::EmptyRevision(reference));
        }
        let mut body = BTreeMap::new();
        for variant in variants {
            let pin = variant.reference();
            if pin.revision_ref() != reference {
                return Err(TemplateLibraryError::ForeignVariant(pin));
            }
            if body.insert(pin.variant, variant).is_some() {
                return Err(TemplateLibraryError::DuplicateVariant(pin));
            }
        }
        Ok(Self {
            reference,
            variants: body,
        })
    }
    /// Complete revision identity, separate from its variant identities.
    pub fn reference(&self) -> TemplateRevisionRef {
        self.reference
    }
    /// All saved variants in stable identity order.
    pub fn variants(&self) -> impl ExactSizeIterator<Item = &SavedTemplateVariant> {
        self.variants.values()
    }
    /// Resolve a variant in this revision without consulting another revision.
    pub fn variant(&self, id: TemplateVariantId) -> Option<&SavedTemplateVariant> {
        self.variants.get(&id)
    }
}
/// Append-only template revisions and explicit advisory recommendations.
/// No deletion or replacement mutator is provided. Publication never adopts project pins.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TemplateLibrary {
    revisions: BTreeMap<TemplateRevisionRef, Arc<TemplateRevision>>,
    recommended: BTreeMap<TemplateDefinitionId, TemplateRevisionRef>,
}
impl TemplateLibrary {
    /// Published revisions in definition/revision identity order.
    pub fn revisions(&self) -> impl ExactSizeIterator<Item = &TemplateRevision> {
        self.revisions.values().map(Arc::as_ref)
    }
    /// Resolve an exact revision, without following a recommendation.
    pub fn revision(&self, reference: TemplateRevisionRef) -> Option<&TemplateRevision> {
        self.revisions.get(&reference).map(Arc::as_ref)
    }
    /// Resolve an exact saved variant. Missing revisions/variants remain missing.
    pub fn variant(&self, reference: TemplateVariantRef) -> Option<&SavedTemplateVariant> {
        self.revision(reference.revision_ref())?
            .variant(reference.variant)
    }
    /// Advisory revision for new review/selection; existing pins are unaffected.
    pub fn recommended(&self, definition: TemplateDefinitionId) -> Option<TemplateRevisionRef> {
        self.recommended.get(&definition).copied()
    }
    /// Recommendation pointers in stable definition order.
    pub fn recommendations(
        &self,
    ) -> impl ExactSizeIterator<Item = (TemplateDefinitionId, TemplateRevisionRef)> + '_ {
        self.recommended.iter().map(|(id, pin)| (*id, *pin))
    }
    /// Append an entire immutable revision; identical re-publication is a no-op.
    /// # Errors
    /// Rejects different content (including extra variants) under an existing revision pin.
    pub fn publish(
        &mut self,
        revision: Arc<TemplateRevision>,
    ) -> Result<bool, TemplateLibraryError> {
        let pin = revision.reference();
        if let Some(previous) = self.revisions.get(&pin) {
            if previous == &revision {
                return Ok(false);
            }
            return Err(TemplateLibraryError::RevisionConflict(pin));
        }
        self.revisions.insert(pin, revision);
        Ok(true)
    }
    /// Set/clear an explicit recommendation, retaining every immutable body.
    /// # Errors
    /// Missing or foreign-lineage targets reject without changing the current pointer.
    pub fn set_recommended(
        &mut self,
        definition: TemplateDefinitionId,
        reference: Option<TemplateRevisionRef>,
    ) -> Result<bool, TemplateLibraryError> {
        if let Some(pin) = reference {
            if pin.definition != definition {
                return Err(TemplateLibraryError::WrongLineage(definition, pin));
            }
            if self.revision(pin).is_none() {
                return Err(TemplateLibraryError::MissingRevision(pin));
            }
        }
        if self.recommended(definition) == reference {
            return Ok(false);
        }
        match reference {
            Some(pin) => {
                self.recommended.insert(definition, pin);
            }
            None => {
                self.recommended.remove(&definition);
            }
        }
        Ok(true)
    }
}
/// Template revision/catalog validation failed without mutating published state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateLibraryError {
    /// A project default or sheet assignment still references this complete revision.
    InUse(TemplateRevisionRef),
    /// A complete revision must contain at least one variant.
    EmptyRevision(TemplateRevisionRef),
    /// A variant belongs to another definition/revision.
    ForeignVariant(TemplateVariantRef),
    /// A serialized/constructed revision repeats a variant identity.
    DuplicateVariant(TemplateVariantRef),
    /// A published revision already has a different complete body.
    RevisionConflict(TemplateRevisionRef),
    /// An exact revision is unavailable.
    MissingRevision(TemplateRevisionRef),
    /// A recommendation belongs to another definition.
    WrongLineage(TemplateDefinitionId, TemplateRevisionRef),
    /// A serialized catalogue repeats an immutable revision.
    DuplicateRevision(TemplateRevisionRef),
    /// A serialized catalogue repeats a recommendation definition.
    DuplicateRecommendation(TemplateDefinitionId),
}
impl fmt::Display for TemplateLibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InUse(pin) => write!(f, "Template revision {pin:?} is still in use"),
            Self::EmptyRevision(pin) => write!(f, "Template revision {pin:?} has no variants"),
            Self::ForeignVariant(pin) => {
                write!(f, "Template variant {pin:?} belongs to another revision")
            }
            Self::DuplicateVariant(pin) => write!(f, "Template variant {pin:?} is repeated"),
            Self::RevisionConflict(pin) => write!(
                f,
                "Published template revision {pin:?} has different content"
            ),
            Self::MissingRevision(pin) => write!(f, "Template revision {pin:?} is unavailable"),
            Self::WrongLineage(id, pin) => {
                write!(f, "Template recommendation {pin:?} does not belong to {id}")
            }
            Self::DuplicateRevision(pin) => write!(f, "Template revision {pin:?} is repeated"),
            Self::DuplicateRecommendation(id) => {
                write!(f, "Template recommendation for {id} is repeated")
            }
        }
    }
}
impl Error for TemplateLibraryError {}
