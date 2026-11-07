//! Atomic replacement of a pinned occurrence, preserving compatible identity.

use super::Project;
use crate::reuse::OwnedObject;
use crate::{InstanceBinding, InstanceLimits, InstanceState, ModelError, ReuseError};
use std::collections::{BTreeMap, BTreeSet};

fn compatible<K: Copy + Ord>(old: &BTreeMap<K, K>, new: &BTreeMap<K, K>) -> bool {
    let previous: BTreeSet<_> = old.values().collect();
    new.iter().all(|(local, placed)| match old.get(local) {
        Some(previous) => previous == placed,
        None => !previous.contains(placed),
    })
}

impl Project {
    /// Replace one pinned occurrence atomically, leaving other occurrences untouched.
    ///
    /// Surviving definition-local objects retain placed identities; a new local object
    /// cannot inherit a removed object's ID. Root name/parent remain local parameters.
    /// External wiring, outer interfaces, and cables must remain valid. Local variants
    /// require a future explicit merge workflow. A temporary candidate is discarded on
    /// failure; history need only retain the two bindings, not a document snapshot.
    /// # Errors
    /// Rejects stale pins, different lineages/roots, identity reassignment, moved terminal
    /// ownership, collisions, missing sources, or invalid resulting electrical content.
    pub fn adopt_instance(
        &mut self,
        expected: &InstanceBinding,
        target: &InstanceBinding,
    ) -> Result<(), ModelError> {
        *self = self.adoption_candidate(expected, target)?;
        Ok(())
    }

    /// Validate a proposed adoption without changing the project or retaining a snapshot.
    /// # Errors
    /// Returns the same identity, source, and electrical conflicts as [`Self::adopt_instance`].
    pub fn validate_adoption(
        &self,
        expected: &InstanceBinding,
        target: &InstanceBinding,
    ) -> Result<(), ModelError> {
        self.adoption_candidate(expected, target).map(|_| ())
    }

    fn adoption_candidate(
        &self,
        expected: &InstanceBinding,
        target: &InstanceBinding,
    ) -> Result<Self, ModelError> {
        if self.instance(expected.root()) != Some(expected) {
            return Err(ReuseError::StaleAdoption(expected.root()).into());
        }
        let old = self.definition(expected.source()).expect("pin has source");
        let new = self
            .definition(target.source())
            .ok_or(ReuseError::MissingRevision(target.source()))?;
        let a = expected.ids();
        let b = target.ids();
        if expected.root() != target.root()
            || expected.source().definition() != target.source().definition()
            || old.root() != new.root()
            || !compatible(&a.subsystems, &b.subsystems)
            || !compatible(&a.devices, &b.devices)
            || !compatible(&a.terminals, &b.terminals)
            || !compatible(&a.ports, &b.ports)
            || !compatible(&a.conductors, &b.conductors)
            || !compatible(&a.cables, &b.cables)
            || a.terminals.keys().any(|id| {
                b.terminals.contains_key(id)
                    && old.content().terminal_owner(*id) != new.content().terminal_owner(*id)
            })
        {
            return Err(ReuseError::IncompatibleAdoption.into());
        }
        let root = self.subsystem(expected.root()).expect("pin has root");
        let plan = new.materialize(
            b.clone(),
            root.parent(),
            root.name(),
            InstanceLimits::default(),
        )?;
        if &plan.binding != target {
            return Err(ReuseError::IncompatibleAdoption.into());
        }
        let mut candidate = self.clone();
        candidate.set_instance_state(expected.root(), InstanceState::Unlinked)?;
        // Only the validated source-owned objects are replaced. External references
        // remain intact throughout candidate construction and are checked at the end.
        let owned: BTreeSet<_> = a.owned().collect();
        candidate
            .subsystems
            .retain(|id, _| !owned.contains(&OwnedObject::Subsystem(*id)));
        candidate
            .devices
            .retain(|id, _| !owned.contains(&OwnedObject::Device(*id)));
        candidate
            .terminal_owners
            .retain(|id, _| !owned.contains(&OwnedObject::Terminal(*id)));
        candidate
            .ports
            .retain(|id, _| !owned.contains(&OwnedObject::Port(*id)));
        candidate.exports.retain(|_, ports| {
            ports.retain(|id| !owned.contains(&OwnedObject::Port(*id)));
            !ports.is_empty()
        });
        candidate
            .conductors
            .retain(|id, _| !owned.contains(&OwnedObject::Conductor(*id)));
        candidate
            .cables
            .retain(|id, _| !owned.contains(&OwnedObject::Cable(*id)));
        candidate
            .conductor_cables
            .retain(|_, id| !owned.contains(&OwnedObject::Cable(*id)));
        for block in plan.subsystems {
            candidate.insert_subsystem(block)?;
        }
        for device in plan.devices {
            candidate.insert_device(device)?;
        }
        for port in plan.ports {
            candidate.insert_port(port)?;
        }
        for wire in plan.conductors {
            candidate.insert_conductor(wire)?;
        }
        for cable in plan.cables {
            candidate.insert_cable(cable)?;
        }
        candidate.bind_instance(plan.binding)?;
        candidate.validate()?;
        Ok(candidate)
    }
}
