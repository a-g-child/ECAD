//! Project ownership and atomic, invariant-preserving single-object operations.

use crate::{Conductor, ConductorId, Device, DeviceId, End, ModelError, TerminalId};
use std::collections::BTreeMap;

/// A structurally valid electrical document, independent of drawing geometry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Project {
    devices: BTreeMap<DeviceId, Device>,
    terminal_owners: BTreeMap<TerminalId, DeviceId>,
    conductors: BTreeMap<ConductorId, Conductor>,
}

impl Project {
    /// Iterate devices in canonical identity order.
    pub fn devices(&self) -> impl ExactSizeIterator<Item = &Device> {
        self.devices.values()
    }

    /// Iterate conductors in canonical identity order.
    pub fn conductors(&self) -> impl ExactSizeIterator<Item = &Conductor> {
        self.conductors.values()
    }

    /// Find an existing device without exposing mutable model storage.
    pub fn device(&self, id: DeviceId) -> Option<&Device> {
        self.devices.get(&id)
    }

    /// Find a conductor without exposing mutable model storage.
    pub fn conductor(&self, id: ConductorId) -> Option<&Conductor> {
        self.conductors.get(&id)
    }

    /// Resolve a project-wide terminal identity to its owning device.
    pub fn terminal_owner(&self, id: TerminalId) -> Option<DeviceId> {
        self.terminal_owners.get(&id).copied()
    }

    /// Insert a device and all its terminals as one operation.
    /// # Errors
    /// Duplicate device or terminal identities reject the entire insertion.
    pub fn insert_device(&mut self, device: Device) -> Result<(), ModelError> {
        if self.devices.contains_key(&device.id()) {
            return Err(ModelError::DuplicateDevice(device.id()));
        }
        for terminal in device.terminals() {
            if self.terminal_owners.contains_key(&terminal.id()) {
                return Err(ModelError::DuplicateTerminal(terminal.id()));
            }
        }
        for terminal in device.terminals() {
            self.terminal_owners.insert(terminal.id(), device.id());
        }
        self.devices.insert(device.id(), device);
        Ok(())
    }

    /// Remove an unconnected device, returning its exact value for undo.
    /// # Errors
    /// Missing or connected devices are rejected without altering any data.
    /// Currently scans conductors; the scale harness should inform a reverse index.
    pub fn remove_device(&mut self, id: DeviceId) -> Result<Device, ModelError> {
        let device = self.devices.get(&id).ok_or(ModelError::MissingDevice(id))?;
        for conductor in self.conductors.values() {
            for end in [End::A, End::B] {
                if conductor
                    .endpoint(end)
                    .is_some_and(|t| self.terminal_owner(t) == Some(id))
                {
                    return Err(ModelError::DeviceConnected(id));
                }
            }
        }
        for terminal in device.terminals() {
            self.terminal_owners.remove(&terminal.id());
        }
        self.devices
            .remove(&id)
            .ok_or(ModelError::MissingDevice(id))
    }

    /// Insert a conductor; free ends are allowed, dangling references are not.
    /// # Errors
    /// Rejects duplicate conductor IDs or nonexistent attached terminals atomically.
    pub fn insert_conductor(&mut self, conductor: Conductor) -> Result<(), ModelError> {
        if self.conductors.contains_key(&conductor.id()) {
            return Err(ModelError::DuplicateConductor(conductor.id()));
        }
        for end in [End::A, End::B] {
            self.check_terminal(conductor.endpoint(end))?;
        }
        self.conductors.insert(conductor.id(), conductor);
        Ok(())
    }

    /// Remove a conductor, retaining its identity and endpoints in the return value.
    /// # Errors
    /// Returns [`ModelError::MissingConductor`] without mutation if absent.
    pub fn remove_conductor(&mut self, id: ConductorId) -> Result<Conductor, ModelError> {
        self.conductors
            .remove(&id)
            .ok_or(ModelError::MissingConductor(id))
    }

    /// Set one termination, returning the previous attachment for undo.
    /// # Errors
    /// Unknown conductors or terminals are rejected before mutation.
    pub fn set_termination(
        &mut self,
        id: ConductorId,
        end: End,
        terminal: Option<TerminalId>,
    ) -> Result<Option<TerminalId>, ModelError> {
        self.check_terminal(terminal)?;
        let conductor = self
            .conductors
            .get_mut(&id)
            .ok_or(ModelError::MissingConductor(id))?;
        let previous = conductor.endpoint(end);
        conductor.set_endpoint(end, terminal);
        Ok(previous)
    }

    /// Recheck references across the complete document, including all free ends.
    /// # Errors
    /// Returns the first missing terminal attachment in stable conductor/end order.
    pub fn validate(&self) -> Result<(), ModelError> {
        for conductor in self.conductors.values() {
            for end in [End::A, End::B] {
                self.check_terminal(conductor.endpoint(end))?;
            }
        }
        Ok(())
    }

    fn check_terminal(&self, terminal: Option<TerminalId>) -> Result<(), ModelError> {
        if let Some(id) = terminal
            && !self.terminal_owners.contains_key(&id)
        {
            return Err(ModelError::MissingTerminal(id));
        }
        Ok(())
    }
}
