//! Exact pin validation and external-interface guards; no per-edit instance scan.

use super::Project;
use crate::reuse::OwnedObject;
use crate::{
    Cable, CableEndpoint, Conductor, End, InstanceBinding, InstanceLimits, ModelError, ReuseError,
};
use std::collections::{BTreeMap, BTreeSet};

impl Project {
    pub(super) fn check_binding(&self, binding: &InstanceBinding) -> Result<(), ModelError> {
        let revision = self
            .definition(binding.source())
            .ok_or(ReuseError::MissingRevision(binding.source()))?;
        let root = self
            .subsystem(binding.root())
            .ok_or(ReuseError::InstanceMismatch(binding.root()))?;
        let plan = revision.materialize(
            binding.ids().clone(),
            root.parent(),
            root.name(),
            InstanceLimits::default(),
        )?;
        let mismatch = || ModelError::from(ReuseError::InstanceMismatch(binding.root()));
        if plan.binding.root() != binding.root()
            || plan
                .subsystems
                .iter()
                .any(|s| self.subsystem(s.id()) != Some(s))
            || plan.devices.iter().any(|d| self.device(d.id()) != Some(d))
            || plan.ports.iter().any(|p| self.port(p.id()) != Some(p))
            || plan
                .conductors
                .iter()
                .any(|c| self.conductor(c.id()) != Some(c))
            || plan.cables.iter().any(|c| self.cable(c.id()) != Some(c))
        {
            return Err(mismatch());
        }
        let owned: BTreeSet<_> = binding.ids().owned().collect();
        for block in self.subsystems.values() {
            if self.ancestors(Some(block.id()))?.contains(&binding.root())
                && !owned.contains(&OwnedObject::Subsystem(block.id()))
            {
                return Err(mismatch());
            }
        }
        for device in self.devices.values() {
            if self
                .ancestors(device.subsystem())?
                .contains(&binding.root())
                && !owned.contains(&OwnedObject::Device(device.id()))
            {
                return Err(mismatch());
            }
        }
        for port in self.ports.values() {
            if self
                .ancestors(Some(port.subsystem()))?
                .contains(&binding.root())
                && !owned.contains(&OwnedObject::Port(port.id()))
            {
                return Err(mismatch());
            }
        }
        for wire in self.conductors.values() {
            if owned.contains(&OwnedObject::Conductor(wire.id())) {
                continue;
            }
            for end in [End::A, End::B] {
                if let Some(terminal) = wire.endpoint(end)
                    && owned.contains(&OwnedObject::Terminal(terminal))
                    && !self.exposes_terminal(binding.root(), terminal)
                {
                    return Err(ReuseError::UnexposedInstanceTerminal(binding.root()).into());
                }
            }
        }
        for cable in self.cables.values() {
            if owned.contains(&OwnedObject::Cable(cable.id())) {
                continue;
            }
            for end in [End::A, End::B] {
                let hidden = match cable.endpoint(end) {
                    CableEndpoint::Device(id) => owned.contains(&OwnedObject::Device(id)),
                    CableEndpoint::Subsystem(id) => {
                        id != binding.root() && owned.contains(&OwnedObject::Subsystem(id))
                    }
                };
                if hidden {
                    return Err(ReuseError::ReadOnlyInstance(binding.root()).into());
                }
            }
            if cable
                .conductors()
                .any(|id| owned.contains(&OwnedObject::Conductor(id)))
            {
                return Err(mismatch());
            }
        }
        Ok(())
    }

    pub(super) fn validate_instances(&self) -> Result<(), ModelError> {
        let mut expected = BTreeMap::new();
        for binding in self.instances.values() {
            self.check_binding(binding)?;
            for object in binding.ids().owned() {
                if expected.insert(object, binding.root()).is_some() {
                    return Err(ReuseError::InstanceOverlap(binding.root()).into());
                }
            }
        }
        if expected != self.instance_owners {
            return Err(ReuseError::InvalidMapping.into());
        }
        for origin in self.local_origins.values() {
            if self.instances.contains_key(&origin.root()) {
                return Err(ReuseError::InstanceExists(origin.root()).into());
            }
            self.check_local_origin(origin)?;
        }
        Ok(())
    }

    pub(super) fn check_instance_wire(&self, wire: &Conductor) -> Result<(), ModelError> {
        let owner = self.instance_owners.get(&OwnedObject::Conductor(wire.id()));
        for end in [End::A, End::B] {
            if let Some(terminal) = wire.endpoint(end)
                && let Some(root) = self.instance_owners.get(&OwnedObject::Terminal(terminal))
                && owner != Some(root)
                && !self.exposes_terminal(*root, terminal)
            {
                return Err(ReuseError::UnexposedInstanceTerminal(*root).into());
            }
        }
        Ok(())
    }

    pub(super) fn check_instance_cable(&self, cable: &Cable) -> Result<(), ModelError> {
        let owner = self.instance_owners.get(&OwnedObject::Cable(cable.id()));
        for end in [End::A, End::B] {
            let hidden = match cable.endpoint(end) {
                CableEndpoint::Device(id) => {
                    self.instance_owners.get(&OwnedObject::Device(id)).copied()
                }
                CableEndpoint::Subsystem(id) => self
                    .instance_owners
                    .get(&OwnedObject::Subsystem(id))
                    .copied()
                    .filter(|root| *root != id),
            };
            if let Some(root) = hidden
                && owner != Some(&root)
            {
                return Err(ReuseError::ReadOnlyInstance(root).into());
            }
        }
        for core in cable.conductors() {
            if let Some(root) = self.instance_owners.get(&OwnedObject::Conductor(core))
                && owner != Some(root)
            {
                return Err(ReuseError::ReadOnlyInstance(*root).into());
            }
        }
        Ok(())
    }
}
