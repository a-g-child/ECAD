//! Typed production mapping generation, kept outside the deterministic core.

use crate::RandomIds;
use ecad_domain::{
    InstanceIds, InstanceLimits, MAX_INSTANCE_OBJECTS, ReuseError, SubsystemRevision,
};
use std::{error::Error, fmt};

/// Production identity mapping could not be generated.
#[derive(Debug)]
pub enum InstanceIdError {
    /// Per-occurrence object budget was exceeded before allocating the mapping.
    ExpansionLimit,
    /// OS randomness failed; no weak fallback is used.
    Entropy(getrandom::Error),
}
impl fmt::Display for InstanceIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpansionLimit => ReuseError::ExpansionLimit.fmt(f),
            Self::Entropy(error) => error.fmt(f),
        }
    }
}
impl Error for InstanceIdError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Entropy(error) => Some(error),
            _ => None,
        }
    }
}
impl From<getrandom::Error> for InstanceIdError {
    fn from(error: getrandom::Error) -> Self {
        Self::Entropy(error)
    }
}

impl RandomIds {
    /// Generate a complete fresh mapping for one immutable revision.
    /// # Errors
    /// Rejects object budgets before mapping allocation and returns entropy failures.
    pub fn instance_ids(
        revision: &SubsystemRevision,
        limits: InstanceLimits,
    ) -> Result<InstanceIds, InstanceIdError> {
        if revision.object_count() > limits.max_objects.get().min(MAX_INSTANCE_OBJECTS) {
            return Err(InstanceIdError::ExpansionLimit);
        }
        let mut ids = InstanceIds::default();
        let body = revision.content();
        for block in body.subsystems() {
            ids.subsystems.insert(block.id(), Self::subsystem()?);
        }
        for device in body.devices() {
            ids.devices.insert(device.id(), Self::device()?);
            for terminal in device.terminals() {
                ids.terminals.insert(terminal.id(), Self::terminal()?);
            }
        }
        for port in body.ports() {
            ids.ports.insert(port.id(), Self::interface_port()?);
        }
        for conductor in body.conductors() {
            ids.conductors.insert(conductor.id(), Self::conductor()?);
        }
        for cable in body.cables() {
            ids.cables.insert(cable.id(), Self::cable()?);
        }
        Ok(ids)
    }
}
