//! Project-owned immutable revisions and indexed occurrence ownership.

use super::Project;
use crate::reuse::OwnedObject;
use crate::{
    DefinitionRef, InstanceBinding, ModelError, ReuseError, SubsystemId, SubsystemRevision,
};
use std::sync::Arc;

impl Project {
    /// Count project-bundled immutable revisions, including uninstantiated history.
    pub fn definition_count(&self) -> usize {
        self.definitions.len()
    }
    /// Iterate bundled revisions in stable lineage/revision order.
    pub fn definitions(&self) -> impl ExactSizeIterator<Item = &SubsystemRevision> {
        self.definitions.values().map(Arc::as_ref)
    }
    /// Resolve a pinned revision without consulting a mutable global library.
    pub fn definition(&self, source: DefinitionRef) -> Option<&SubsystemRevision> {
        self.definitions.get(&source).map(Arc::as_ref)
    }
    /// Iterate bound occurrence roots in stable identity order.
    pub fn instances(&self) -> impl ExactSizeIterator<Item = &InstanceBinding> {
        self.instances.values()
    }
    /// Inspect the pinned revision and identity correspondence of a placed root.
    pub fn instance(&self, root: SubsystemId) -> Option<&InstanceBinding> {
        self.instances.get(&root)
    }

    /// Import an immutable revision; importing identical content again is a no-op.
    /// # Errors
    /// Rejects different content under an already published identity pair.
    pub fn insert_definition(
        &mut self,
        revision: Arc<SubsystemRevision>,
    ) -> Result<bool, ModelError> {
        if let Some(existing) = self.definitions.get(&revision.source()) {
            if existing == &revision {
                return Ok(false);
            }
            return Err(ReuseError::RevisionConflict(revision.source()).into());
        }
        self.definitions.insert(revision.source(), revision);
        Ok(true)
    }

    /// Remove a bundled revision only when no pin or local origin references it.
    /// # Errors
    /// Rejects unavailable or still-used revisions without mutation.
    pub fn remove_definition(
        &mut self,
        source: DefinitionRef,
    ) -> Result<Arc<SubsystemRevision>, ModelError> {
        if self
            .instances
            .values()
            .chain(self.local_origins.values())
            .any(|instance| instance.source() == source)
        {
            return Err(ReuseError::RevisionInUse(source).into());
        }
        self.definitions
            .remove(&source)
            .ok_or_else(|| ReuseError::MissingRevision(source).into())
    }

    /// Bind already-materialized objects to an immutable revision after exact validation.
    /// # Errors
    /// Rejects missing revisions, incomplete/mismatched content, overlaps, or hidden external wiring.
    pub fn bind_instance(&mut self, binding: InstanceBinding) -> Result<(), ModelError> {
        if self.instances.contains_key(&binding.root()) {
            return Err(ReuseError::InstanceExists(binding.root()).into());
        }
        self.set_instance_state(binding.root(), crate::InstanceState::Pinned(binding))?;
        Ok(())
    }

    /// Make one occurrence local, retaining all object IDs, wiring, and bundled revision.
    /// # Errors
    /// Rejects unknown occurrence roots. Undo can rebind after local edits are undone.
    pub fn detach_instance(&mut self, root: SubsystemId) -> Result<InstanceBinding, ModelError> {
        let binding = self
            .instances
            .get(&root)
            .cloned()
            .ok_or(ReuseError::MissingInstance(root))?;
        self.set_instance_state(root, crate::InstanceState::Local(binding.clone()))?;
        Ok(binding)
    }

    pub(super) fn ensure_editable(&self, object: OwnedObject) -> Result<(), ModelError> {
        if let Some(root) = self.instance_owners.get(&object) {
            return Err(ReuseError::ReadOnlyInstance(*root).into());
        }
        Ok(())
    }

    pub(super) fn ensure_local_parent(
        &self,
        parent: Option<SubsystemId>,
    ) -> Result<(), ModelError> {
        if let Some(parent) = parent {
            self.ensure_editable(OwnedObject::Subsystem(parent))?;
        }
        Ok(())
    }
}
