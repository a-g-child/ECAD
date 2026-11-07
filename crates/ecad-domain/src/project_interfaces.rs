//! Boundary exposure and cross-scope connectivity over real terminal identities.

use super::Project;
use crate::{InterfacePort, InterfacePortId, ModelError, SubsystemId, TerminalId};
use std::collections::BTreeSet;

impl Project {
    /// Iterate external interfaces in stable identity order.
    pub fn ports(&self) -> impl ExactSizeIterator<Item = &InterfacePort> {
        self.ports.values()
    }
    /// Find an interface exposure without manufacturing a second terminal.
    pub fn port(&self, id: InterfacePortId) -> Option<&InterfacePort> {
        self.ports.get(&id)
    }

    /// Add an interface for a contained terminal, respecting intervening boundaries.
    /// # Errors
    /// Rejects duplicates, reused names, foreign terminals, or missing child exposures.
    pub fn insert_port(&mut self, port: InterfacePort) -> Result<(), ModelError> {
        if self.ports.contains_key(&port.id()) {
            return Err(ModelError::DuplicatePort(port.id()));
        }
        if self
            .ports
            .values()
            .any(|p| p.subsystem() == port.subsystem() && p.name() == port.name())
        {
            return Err(ModelError::PortNameInUse(
                port.subsystem(),
                port.name().to_owned(),
            ));
        }
        self.check_port(&port)?;
        self.exports
            .entry((port.subsystem(), port.terminal()))
            .or_default()
            .insert(port.id());
        self.ports.insert(port.id(), port);
        Ok(())
    }

    /// Remove an unused exposure; dependencies are validated before committing.
    /// # Errors
    /// Missing ports or invalidated outer interfaces, conductors, or cable ends restore it.
    pub fn remove_port(&mut self, id: InterfacePortId) -> Result<InterfacePort, ModelError> {
        let port = self.ports.remove(&id).ok_or(ModelError::MissingPort(id))?;
        let key = (port.subsystem(), port.terminal());
        let ids = self
            .exports
            .get_mut(&key)
            .expect("stored ports have an export index");
        ids.remove(&id);
        if ids.is_empty() {
            self.exports.remove(&key);
        }
        if let Err(error) = self.validate() {
            self.exports.entry(key).or_default().insert(id);
            self.ports.insert(id, port);
            return Err(error);
        }
        Ok(port)
    }

    /// Whether this scope explicitly exposes a particular physical terminal.
    pub fn exposes_terminal(&self, subsystem: SubsystemId, terminal: TerminalId) -> bool {
        self.exports.contains_key(&(subsystem, terminal))
    }

    pub(super) fn check_port(&self, port: &InterfacePort) -> Result<(), ModelError> {
        self.require_subsystem(Some(port.subsystem()))?;
        let path = self.terminal_path(port.terminal())?;
        let index = path.iter().position(|id| *id == port.subsystem()).ok_or(
            ModelError::TerminalOutsideSubsystem(port.subsystem(), port.terminal()),
        )?;
        for scope in &path[..index] {
            if !self.exposes_terminal(*scope, port.terminal()) {
                return Err(ModelError::InterfaceRequired(*scope, port.terminal()));
            }
        }
        Ok(())
    }

    fn terminal_path(&self, terminal: TerminalId) -> Result<Vec<SubsystemId>, ModelError> {
        let owner = self
            .terminal_owner(terminal)
            .ok_or(ModelError::MissingTerminal(terminal))?;
        let device = self
            .devices
            .get(&owner)
            .ok_or(ModelError::MissingDevice(owner))?;
        self.ancestors(device.subsystem())
    }

    pub(super) fn check_boundaries(
        &self,
        a: Option<TerminalId>,
        b: Option<TerminalId>,
    ) -> Result<(), ModelError> {
        let (Some(a), Some(b)) = (a, b) else {
            return Ok(());
        };
        let first = self.terminal_path(a)?;
        let second = self.terminal_path(b)?;
        let second_set: BTreeSet<_> = second.iter().copied().collect();
        let common = first.iter().find(|id| second_set.contains(id)).copied();
        for (terminal, path) in [(a, &first), (b, &second)] {
            for scope in path {
                if Some(*scope) == common {
                    break;
                }
                if !self.exposes_terminal(*scope, terminal) {
                    return Err(ModelError::InterfaceRequired(*scope, terminal));
                }
            }
        }
        Ok(())
    }
}
