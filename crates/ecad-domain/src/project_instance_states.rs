//! Atomic transitions between pinned, traced-local, and ordinary local content.

use super::Project;
use crate::reuse::OwnedObject;
use crate::{InstanceBinding, InstanceLimits, InstanceState, ModelError, ReuseError, SubsystemId};

impl Project {
    /// Iterate editable local variants retaining their original source correspondence.
    pub fn local_origins(&self) -> impl ExactSizeIterator<Item = &InstanceBinding> {
        self.local_origins.values()
    }
    /// Inspect the historical baseline of a local variant; it grants no edit locks.
    pub fn local_origin(&self, root: SubsystemId) -> Option<&InstanceBinding> {
        self.local_origins.get(&root)
    }
    /// Snapshot reversible link/provenance state without exposing mutable storage.
    pub fn instance_state(&self, root: SubsystemId) -> InstanceState {
        if let Some(binding) = self.instances.get(&root) {
            InstanceState::Pinned(binding.clone())
        } else if let Some(origin) = self.local_origins.get(&root) {
            InstanceState::Local(origin.clone())
        } else {
            InstanceState::Unlinked
        }
    }

    /// Validate and atomically change link/provenance state, returning its previous value.
    ///
    /// Pinned targets must exactly match their source. Local targets preserve a
    /// historical mapping but allow changed/deleted internals. Unlinking deliberately
    /// forgets provenance. This operation does not delete or regenerate any objects.
    /// # Errors
    /// Invalid roots, missing sources, malformed mappings, overlaps, or mismatched
    /// pinned content reject the change before touching ownership or metadata.
    pub fn set_instance_state(
        &mut self,
        root: SubsystemId,
        state: InstanceState,
    ) -> Result<InstanceState, ModelError> {
        let previous = self.instance_state(root);
        if previous == state {
            return Ok(previous);
        }
        match &state {
            InstanceState::Pinned(binding) => {
                if binding.root() != root {
                    return Err(ReuseError::InstanceMismatch(root).into());
                }
                self.check_binding(binding)?;
                for object in binding.ids().owned() {
                    if self
                        .instance_owners
                        .get(&object)
                        .is_some_and(|owner| *owner != root)
                    {
                        return Err(ReuseError::InstanceOverlap(root).into());
                    }
                }
            }
            InstanceState::Local(origin) => {
                if origin.root() != root {
                    return Err(ReuseError::InstanceMismatch(root).into());
                }
                self.check_local_origin(origin)?;
            }
            InstanceState::Unlinked => {}
        }
        if let Some(old) = self.instances.remove(&root) {
            for object in old.ids().owned() {
                self.instance_owners.remove(&object);
            }
        }
        self.local_origins.remove(&root);
        match state {
            InstanceState::Pinned(binding) => {
                for object in binding.ids().owned() {
                    self.instance_owners.insert(object, root);
                }
                self.instances.insert(root, binding);
            }
            InstanceState::Local(origin) => {
                self.local_origins.insert(root, origin);
            }
            InstanceState::Unlinked => {}
        }
        Ok(previous)
    }

    /// Restore/import local ancestry without silently replacing an active pin or origin.
    /// # Errors
    /// Rejects duplicate metadata, missing sources, invalid roots, or malformed mappings.
    pub fn remember_local_origin(&mut self, origin: InstanceBinding) -> Result<(), ModelError> {
        if self.instances.contains_key(&origin.root()) {
            return Err(ReuseError::InstanceExists(origin.root()).into());
        }
        if self.local_origins.contains_key(&origin.root()) {
            return Err(ReuseError::LocalOriginExists(origin.root()).into());
        }
        self.set_instance_state(origin.root(), InstanceState::Local(origin))?;
        Ok(())
    }

    /// Forget a local variant's baseline deliberately; electrical objects are untouched.
    /// # Errors
    /// Rejects roots without local ancestry. The returned value supports exact undo.
    pub fn forget_local_origin(
        &mut self,
        root: SubsystemId,
    ) -> Result<InstanceBinding, ModelError> {
        let origin = self
            .local_origins
            .get(&root)
            .cloned()
            .ok_or(ReuseError::MissingLocalOrigin(root))?;
        self.set_instance_state(root, InstanceState::Unlinked)?;
        Ok(origin)
    }

    pub(super) fn check_local_origin(&self, origin: &InstanceBinding) -> Result<(), ModelError> {
        let definition = self
            .definition(origin.source())
            .ok_or(ReuseError::MissingRevision(origin.source()))?;
        let root = self
            .subsystem(origin.root())
            .ok_or(ReuseError::InstanceMismatch(origin.root()))?;
        let plan = definition.materialize(
            origin.ids().clone(),
            root.parent(),
            root.name(),
            InstanceLimits::default(),
        )?;
        if plan.binding.root() != origin.root() {
            return Err(ReuseError::InstanceMismatch(origin.root()).into());
        }
        if self
            .instance_owners
            .get(&OwnedObject::Subsystem(origin.root()))
            .is_some_and(|owner| *owner != origin.root())
        {
            return Err(ReuseError::InstanceOverlap(origin.root()).into());
        }
        for object in origin.ids().owned() {
            if self
                .instance_owners
                .get(&object)
                .is_some_and(|owner| *owner != origin.root())
            {
                return Err(ReuseError::InstanceOverlap(origin.root()).into());
            }
        }
        Ok(())
    }
}
