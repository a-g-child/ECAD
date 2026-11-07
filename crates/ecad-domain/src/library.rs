//! Append-only subsystem revisions and explicitly mutable recommendations.

use crate::{DefinitionId, DefinitionRef, SubsystemRevision};
use std::{collections::BTreeMap, error::Error, fmt, sync::Arc};

/// A shared catalog operation violates immutability or recommendation resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LibraryError {
    /// An existing key already identifies different content.
    RevisionConflict(DefinitionRef),
    /// The requested revision is not published here.
    MissingRevision(DefinitionRef),
    /// A recommendation must point into the selected lineage.
    WrongLineage(DefinitionId, DefinitionRef),
    /// Serialized catalogs must list each immutable key once.
    DuplicateRevision(DefinitionRef),
    /// Serialized catalogs must list each recommendation lineage once.
    DuplicateRecommendation(DefinitionId),
}
impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RevisionConflict(id) => {
                write!(f, "published revision {id} has different content")
            }
            Self::MissingRevision(id) => write!(f, "revision {id} is not published here"),
            Self::WrongLineage(id, source) => {
                write!(f, "recommendation {source} does not belong to {id}")
            }
            Self::DuplicateRevision(id) => write!(f, "catalog repeats revision {id}"),
            Self::DuplicateRecommendation(id) => {
                write!(f, "catalog repeats recommendation for {id}")
            }
        }
    }
}
impl Error for LibraryError {}

/// Validated immutable bodies plus optional recommendations, independent of projects.
/// No deletion or in-place replacement API is provided. A recommendation is advisory;
/// it never changes a project's bundled revisions, pins, or local variants.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SubsystemLibrary {
    revisions: BTreeMap<DefinitionRef, Arc<SubsystemRevision>>,
    recommended: BTreeMap<DefinitionId, DefinitionRef>,
}
impl SubsystemLibrary {
    /// Iterate every published body in stable lineage/revision order.
    pub fn revisions(&self) -> impl ExactSizeIterator<Item = &SubsystemRevision> {
        self.revisions.values().map(Arc::as_ref)
    }
    /// Resolve an exact published revision, without following recommendations.
    pub fn revision(&self, source: DefinitionRef) -> Option<&SubsystemRevision> {
        self.revisions.get(&source).map(Arc::as_ref)
    }
    /// Inspect the explicit recommended revision, if any, for new review/import.
    pub fn recommended(&self, lineage: DefinitionId) -> Option<DefinitionRef> {
        self.recommended.get(&lineage).copied()
    }
    /// Iterate recommendation pointers in stable lineage order.
    pub fn recommendations(
        &self,
    ) -> impl ExactSizeIterator<Item = (DefinitionId, DefinitionRef)> + '_ {
        self.recommended.iter().map(|(id, source)| (*id, *source))
    }
    /// Append a validated revision; identical re-publication is a no-op.
    /// # Errors
    /// Rejects different content under the same immutable identity without mutation.
    pub fn publish(&mut self, revision: Arc<SubsystemRevision>) -> Result<bool, LibraryError> {
        if let Some(existing) = self.revisions.get(&revision.source()) {
            if existing == &revision {
                return Ok(false);
            }
            return Err(LibraryError::RevisionConflict(revision.source()));
        }
        self.revisions.insert(revision.source(), revision);
        Ok(true)
    }
    /// Explicitly recommend a published revision or clear the pointer, retaining history.
    /// Returns whether the pointer changed. No numeric/timestamp ordering is inferred.
    /// # Errors
    /// Rejects foreign-lineage or missing revisions before changing the old pointer.
    pub fn set_recommended(
        &mut self,
        lineage: DefinitionId,
        source: Option<DefinitionRef>,
    ) -> Result<bool, LibraryError> {
        if let Some(source) = source {
            if source.definition() != lineage {
                return Err(LibraryError::WrongLineage(lineage, source));
            }
            if !self.revisions.contains_key(&source) {
                return Err(LibraryError::MissingRevision(source));
            }
        }
        if self.recommended(lineage) == source {
            return Ok(false);
        }
        match source {
            Some(source) => {
                self.recommended.insert(lineage, source);
            }
            None => {
                self.recommended.remove(&lineage);
            }
        }
        Ok(true)
    }
}
