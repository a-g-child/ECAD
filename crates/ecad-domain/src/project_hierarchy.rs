//! Containment edits and iterative bounded traversal, kept separate from geometry.

use super::Project;
use crate::{
    CableEndpoint, DeviceId, End, MAX_SUBSYSTEM_DEPTH, ModelError, Subsystem, SubsystemId,
};
use std::collections::BTreeSet;

impl Project {
    /// Iterate occurrences in stable identity order (not parent-first order).
    pub fn subsystems(&self) -> impl ExactSizeIterator<Item = &Subsystem> {
        self.subsystems.values()
    }
    /// Find a placed functional block.
    pub fn subsystem(&self, id: SubsystemId) -> Option<&Subsystem> {
        self.subsystems.get(&id)
    }

    /// Return the parent-first path to an occurrence, without recursive traversal.
    /// # Errors
    /// Reports missing blocks, parent cycles, or the depth resource guard.
    pub fn subsystem_path(&self, id: SubsystemId) -> Result<Vec<SubsystemId>, ModelError> {
        let mut path = self.ancestors(Some(id))?;
        path.reverse();
        Ok(path)
    }

    /// Insert a local occurrence; parent references must already exist.
    /// # Errors
    /// Rejects duplicates, missing parents, cycles, or depth overflow atomically.
    pub fn insert_subsystem(&mut self, subsystem: Subsystem) -> Result<(), ModelError> {
        if self.subsystems.contains_key(&subsystem.id()) {
            return Err(ModelError::DuplicateSubsystem(subsystem.id()));
        }
        if subsystem.parent() == Some(subsystem.id()) {
            return Err(ModelError::HierarchyCycle(subsystem.id()));
        }
        if self.ancestors(subsystem.parent())?.len() >= MAX_SUBSYSTEM_DEPTH {
            return Err(ModelError::HierarchyTooDeep);
        }
        self.subsystems.insert(subsystem.id(), subsystem);
        Ok(())
    }

    /// Remove only an empty, unreferenced occurrence and retain its value for undo.
    /// # Errors
    /// Missing blocks or any children, devices, ports, or cable ends reject removal.
    pub fn remove_subsystem(&mut self, id: SubsystemId) -> Result<Subsystem, ModelError> {
        self.require_subsystem(Some(id))?;
        if self.subsystems.values().any(|s| s.parent() == Some(id))
            || self.devices.values().any(|d| d.subsystem() == Some(id))
            || self.ports.values().any(|p| p.subsystem() == id)
            || self.cables.values().any(|c| {
                [End::A, End::B]
                    .into_iter()
                    .any(|e| c.endpoint(e) == CableEndpoint::Subsystem(id))
            })
        {
            return Err(ModelError::SubsystemInUse(id));
        }
        self.subsystems
            .remove(&id)
            .ok_or(ModelError::MissingSubsystem(id))
    }

    /// Reparent an occurrence while preserving identities and existing electrical intent.
    /// Reorganization currently validates the full model; ordinary wire edits do not.
    /// # Errors
    /// Invalid parent chains, boundary mappings, or cable wiring roll back the change.
    pub fn set_subsystem_parent(
        &mut self,
        id: SubsystemId,
        parent: Option<SubsystemId>,
    ) -> Result<Option<SubsystemId>, ModelError> {
        self.require_subsystem(parent)?;
        let block = self
            .subsystems
            .get_mut(&id)
            .ok_or(ModelError::MissingSubsystem(id))?;
        let previous = block.parent();
        block.set_parent(parent);
        if let Err(error) = self.validate() {
            self.subsystems
                .get_mut(&id)
                .expect("reparent does not remove its block")
                .set_parent(previous);
            return Err(error);
        }
        Ok(previous)
    }

    /// Move a device between functional scopes, without moving drawing geometry.
    /// # Errors
    /// Missing scopes/devices or invalidated exports/connections reject the entire move.
    pub fn set_device_subsystem(
        &mut self,
        id: DeviceId,
        subsystem: Option<SubsystemId>,
    ) -> Result<Option<SubsystemId>, ModelError> {
        self.require_subsystem(subsystem)?;
        let device = self
            .devices
            .get_mut(&id)
            .ok_or(ModelError::MissingDevice(id))?;
        let previous = device.subsystem();
        device.set_subsystem(subsystem);
        if let Err(error) = self.validate() {
            self.devices
                .get_mut(&id)
                .expect("scope edit does not remove its device")
                .set_subsystem(previous);
            return Err(error);
        }
        Ok(previous)
    }

    pub(super) fn require_subsystem(&self, id: Option<SubsystemId>) -> Result<(), ModelError> {
        if let Some(id) = id
            && !self.subsystems.contains_key(&id)
        {
            return Err(ModelError::MissingSubsystem(id));
        }
        Ok(())
    }

    pub(super) fn ancestors(
        &self,
        mut scope: Option<SubsystemId>,
    ) -> Result<Vec<SubsystemId>, ModelError> {
        let mut path = Vec::new();
        let mut seen = BTreeSet::new();
        while let Some(id) = scope {
            if !seen.insert(id) {
                return Err(ModelError::HierarchyCycle(id));
            }
            if path.len() >= MAX_SUBSYSTEM_DEPTH {
                return Err(ModelError::HierarchyTooDeep);
            }
            let block = self
                .subsystems
                .get(&id)
                .ok_or(ModelError::MissingSubsystem(id))?;
            path.push(id);
            scope = block.parent();
        }
        Ok(path)
    }

    pub(super) fn validate_hierarchy(&self) -> Result<(), ModelError> {
        for id in self.subsystems.keys() {
            self.ancestors(Some(*id))?;
        }
        for device in self.devices.values() {
            self.require_subsystem(device.subsystem())?;
        }
        for port in self.ports.values() {
            self.check_port(port)?;
        }
        Ok(())
    }
}
