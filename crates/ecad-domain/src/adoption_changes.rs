//! Deterministic definition-local changes for revision review, independent of UI.

use crate::{
    CableId, ConductorId, DeviceId, InterfacePortId, SubsystemId, SubsystemRevision, TerminalId,
};
use std::collections::{BTreeMap, BTreeSet};

/// Definition-local IDs added, removed, or modified between two immutable bodies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectChanges<K: Ord> {
    /// IDs present only in the target body.
    pub added: BTreeSet<K>,
    /// IDs present only in the previous body.
    pub removed: BTreeSet<K>,
    /// Surviving IDs whose source values differ.
    pub modified: BTreeSet<K>,
}

fn changes<K: Copy + Ord, V: PartialEq>(
    before: impl Iterator<Item = (K, V)>,
    after: impl Iterator<Item = (K, V)>,
) -> ObjectChanges<K> {
    let before: BTreeMap<_, _> = before.collect();
    let after: BTreeMap<_, _> = after.collect();
    ObjectChanges {
        added: after
            .keys()
            .filter(|k| !before.contains_key(k))
            .copied()
            .collect(),
        removed: before
            .keys()
            .filter(|k| !after.contains_key(k))
            .copied()
            .collect(),
        modified: before
            .iter()
            .filter_map(|(k, v)| after.get(k).filter(|new| *new != v).map(|_| *k))
            .collect(),
    }
}

/// Source-body review; IDs are definition-local, not placed occurrence identities.
/// Root name changes are reported even though adoption preserves the occurrence name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevisionChanges {
    /// Containment and source names.
    pub subsystems: ObjectChanges<SubsystemId>,
    /// Device values, including terminal lists.
    pub devices: ObjectChanges<DeviceId>,
    /// Terminal values and owning device identity.
    pub terminals: ObjectChanges<TerminalId>,
    /// Exposed anchors, names, and containing scopes.
    pub ports: ObjectChanges<InterfacePortId>,
    /// Internal conductor attachments.
    pub conductors: ObjectChanges<ConductorId>,
    /// Cable endpoints, core membership, and designation.
    pub cables: ObjectChanges<CableId>,
}

impl RevisionChanges {
    /// Compare immutable bodies by stable local identity, never names or array order.
    pub fn between(before: &SubsystemRevision, after: &SubsystemRevision) -> Self {
        let a = before.content();
        let b = after.content();
        Self {
            subsystems: changes(
                a.subsystems().map(|s| (s.id(), s)),
                b.subsystems().map(|s| (s.id(), s)),
            ),
            devices: changes(
                a.devices().map(|s| (s.id(), s)),
                b.devices().map(|s| (s.id(), s)),
            ),
            terminals: changes(
                a.devices()
                    .flat_map(|d| d.terminals().map(move |t| (t.id(), (d.id(), t)))),
                b.devices()
                    .flat_map(|d| d.terminals().map(move |t| (t.id(), (d.id(), t)))),
            ),
            ports: changes(
                a.ports().map(|s| (s.id(), s)),
                b.ports().map(|s| (s.id(), s)),
            ),
            conductors: changes(
                a.conductors().map(|s| (s.id(), s)),
                b.conductors().map(|s| (s.id(), s)),
            ),
            cables: changes(
                a.cables().map(|s| (s.id(), s)),
                b.cables().map(|s| (s.id(), s)),
            ),
        }
    }
}
