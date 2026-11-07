//! Cable membership ownership and consistency between overview ends and core wiring.

use super::Project;
use crate::reuse::OwnedObject;
use crate::{Cable, CableEndpoint, CableId, Conductor, ConductorId, End, ModelError};

impl Project {
    /// Iterate cable assemblies in stable identity order.
    pub fn cables(&self) -> impl ExactSizeIterator<Item = &Cable> {
        self.cables.values()
    }
    /// Find an overview cable and its core identities.
    pub fn cable(&self, id: CableId) -> Option<&Cable> {
        self.cables.get(&id)
    }
    /// Find the unique owning cable, if the conductor is a cable core.
    pub fn conductor_cable(&self, id: ConductorId) -> Option<CableId> {
        self.conductor_cables.get(&id).copied()
    }

    /// Group existing conductors without copying their electrical state.
    /// # Errors
    /// Rejects duplicates, nonexistent endpoints/cores, reused cores, or mismatched wiring.
    pub fn insert_cable(&mut self, cable: Cable) -> Result<(), ModelError> {
        if self.cables.contains_key(&cable.id()) {
            return Err(ModelError::DuplicateCable(cable.id()));
        }
        self.check_cable(&cable)?;
        for conductor in cable.conductors() {
            self.conductor_cables.insert(conductor, cable.id());
        }
        self.cables.insert(cable.id(), cable);
        Ok(())
    }

    /// Ungroup an assembly, preserving its individual conductors and terminations.
    /// # Errors
    /// Returns a missing-cable error without mutation when absent.
    pub fn remove_cable(&mut self, id: CableId) -> Result<Cable, ModelError> {
        self.ensure_editable(OwnedObject::Cable(id))?;
        let cable = self
            .cables
            .remove(&id)
            .ok_or(ModelError::MissingCable(id))?;
        for conductor in cable.conductors() {
            self.conductor_cables.remove(&conductor);
        }
        Ok(cable)
    }

    pub(super) fn check_cable(&self, cable: &Cable) -> Result<(), ModelError> {
        self.check_instance_cable(cable)?;
        for end in [End::A, End::B] {
            match cable.endpoint(end) {
                CableEndpoint::Device(id) if !self.devices.contains_key(&id) => {
                    return Err(ModelError::MissingDevice(id));
                }
                CableEndpoint::Subsystem(id) => self.require_subsystem(Some(id))?,
                _ => {}
            }
        }
        for id in cable.conductors() {
            if self
                .conductor_cables
                .get(&id)
                .is_some_and(|owner| *owner != cable.id())
            {
                return Err(ModelError::ConductorInCable(id));
            }
            let conductor = self
                .conductors
                .get(&id)
                .ok_or(ModelError::MissingConductor(id))?;
            self.check_core_endpoints(cable, conductor)?;
        }
        Ok(())
    }

    pub(super) fn check_core_endpoints(
        &self,
        cable: &Cable,
        conductor: &Conductor,
    ) -> Result<(), ModelError> {
        for end in [End::A, End::B] {
            let Some(terminal) = conductor.endpoint(end) else {
                continue;
            };
            let matches = match cable.endpoint(end) {
                CableEndpoint::Device(id) => self.terminal_owner(terminal) == Some(id),
                CableEndpoint::Subsystem(id) => self.exposes_terminal(id, terminal),
            };
            if !matches {
                return Err(ModelError::CableEndpointMismatch(
                    cable.id(),
                    conductor.id(),
                    end,
                ));
            }
        }
        Ok(())
    }
}
