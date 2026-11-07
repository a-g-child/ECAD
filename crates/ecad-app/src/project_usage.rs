//! Read-only summaries of bundled revisions, exact pins, and local ancestry.

use ecad_domain::{DefinitionRef, Project, ProjectId, SubsystemId};
use std::collections::{BTreeMap, BTreeSet};

/// Distinct uses of one bundled revision; empty sets mean bundled content only.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RevisionUse {
    /// Occurrence roots pinned exactly to this revision.
    pub pinned: BTreeSet<SubsystemId>,
    /// Editable local roots retaining this revision as historical ancestry.
    pub local_origins: BTreeSet<SubsystemId>,
}

/// A small derived summary, not authoritative project data or proof of global usage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectUsage {
    project: ProjectId,
    revisions: BTreeMap<DefinitionRef, RevisionUse>,
}
impl ProjectUsage {
    /// Extract usage from one validated project, without changing its contents.
    pub fn from_project(project: &Project) -> Self {
        let mut revisions: BTreeMap<_, _> = project
            .definitions()
            .map(|revision| (revision.source(), RevisionUse::default()))
            .collect();
        for pin in project.instances() {
            revisions
                .entry(pin.source())
                .or_default()
                .pinned
                .insert(pin.root());
        }
        for origin in project.local_origins() {
            revisions
                .entry(origin.source())
                .or_default()
                .local_origins
                .insert(origin.root());
        }
        Self {
            project: project.id(),
            revisions,
        }
    }
    /// Persistent lineage; different file locations may legitimately share this ID.
    pub fn project_id(&self) -> ProjectId {
        self.project
    }
    /// Iterate all bundled revision references in stable order, including unused bodies.
    pub fn revisions(&self) -> impl ExactSizeIterator<Item = (DefinitionRef, &RevisionUse)> {
        self.revisions
            .iter()
            .map(|(source, usage)| (*source, usage))
    }
    /// Inspect exact revision usage, distinguishing absence from bundled-only content.
    pub fn revision(&self, source: DefinitionRef) -> Option<&RevisionUse> {
        self.revisions.get(&source)
    }
}
