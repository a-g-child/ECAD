//! Total, typed remapping into independently editable electrical identities.

use super::*;
use crate::{CableEndpoint, End, ReuseError, Terminal};
use std::{cmp::Reverse, collections::BTreeSet};

/// Hard provisional bound for one materialized occurrence, including terminals.
pub const MAX_INSTANCE_OBJECTS: usize = 100_000;

fn complete<K: Copy + Ord>(map: &BTreeMap<K, K>, keys: impl Iterator<Item = K>) -> bool {
    let expected: BTreeSet<_> = keys.collect();
    map.len() == expected.len()
        && map.keys().all(|key| expected.contains(key))
        && map.values().copied().collect::<BTreeSet<_>>().len() == map.len()
}

impl SubsystemRevision {
    /// Materialize one occurrence using a complete one-to-one identity mapping.
    ///
    /// Root name and parent are occurrence parameters. Internal designations are
    /// retained and qualified by hierarchy; no timestamp or random generation occurs.
    /// # Errors
    /// Rejects malformed mappings, blank names, and explicit/hard object-budget limits.
    /// Destination ID collisions and parent validity are checked during transaction application.
    pub fn materialize(
        &self,
        ids: InstanceIds,
        parent: Option<SubsystemId>,
        name: &str,
        limits: InstanceLimits,
    ) -> Result<InstancePlan, ModelError> {
        if self.object_count() > limits.max_objects.get().min(MAX_INSTANCE_OBJECTS) {
            return Err(ReuseError::ExpansionLimit.into());
        }
        let body = self.content();
        if !complete(&ids.subsystems, body.subsystems().map(Subsystem::id))
            || !complete(&ids.devices, body.devices().map(Device::id))
            || !complete(
                &ids.terminals,
                body.devices().flat_map(Device::terminals).map(Terminal::id),
            )
            || !complete(&ids.ports, body.ports().map(InterfacePort::id))
            || !complete(&ids.conductors, body.conductors().map(Conductor::id))
            || !complete(&ids.cables, body.cables().map(Cable::id))
        {
            return Err(ReuseError::InvalidMapping.into());
        }
        let mut blocks = Vec::new();
        for block in body.subsystems() {
            let depth = body.subsystem_path(block.id())?.len();
            let root = block.id() == self.root();
            let owner = if root {
                parent
            } else {
                block.parent().map(|id| ids.subsystems[&id])
            };
            blocks.push((
                depth,
                block.id(),
                Subsystem::new(
                    ids.subsystems[&block.id()],
                    if root { name } else { block.name() },
                    owner,
                )?,
            ));
        }
        blocks.sort_by_key(|(depth, id, _)| (*depth, *id));
        let mut devices = Vec::new();
        for device in body.devices() {
            let terminals = device
                .terminals()
                .map(|terminal| Terminal::new(ids.terminals[&terminal.id()], terminal.number()))
                .collect::<Result<Vec<_>, _>>()?;
            let owner = device
                .subsystem()
                .ok_or(ReuseError::InvalidDefinitionRoot)?;
            devices.push(
                Device::new(ids.devices[&device.id()], device.designation(), terminals)?
                    .in_subsystem(ids.subsystems[&owner]),
            );
        }
        let mut ports = Vec::new();
        for port in body.ports() {
            let depth = body.subsystem_path(port.subsystem())?.len();
            ports.push((
                Reverse(depth),
                port.id(),
                InterfacePort::new(
                    ids.ports[&port.id()],
                    ids.subsystems[&port.subsystem()],
                    port.name(),
                    ids.terminals[&port.terminal()],
                )?,
            ));
        }
        ports.sort_by_key(|(depth, id, _)| (*depth, *id));
        let conductors = body
            .conductors()
            .map(|wire| {
                Conductor::new(
                    ids.conductors[&wire.id()],
                    wire.endpoint(End::A).map(|id| ids.terminals[&id]),
                    wire.endpoint(End::B).map(|id| ids.terminals[&id]),
                )
            })
            .collect();
        let endpoint = |value| match value {
            CableEndpoint::Device(id) => CableEndpoint::Device(ids.devices[&id]),
            CableEndpoint::Subsystem(id) => CableEndpoint::Subsystem(ids.subsystems[&id]),
        };
        let cables = body
            .cables()
            .map(|cable| {
                Cable::new(
                    ids.cables[&cable.id()],
                    cable.designation(),
                    endpoint(cable.endpoint(End::A)),
                    endpoint(cable.endpoint(End::B)),
                    cable.conductors().map(|id| ids.conductors[&id]),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let binding = InstanceBinding::new(ids.subsystems[&self.root()], self.source(), ids);
        Ok(InstancePlan {
            subsystems: blocks.into_iter().map(|(_, _, block)| block).collect(),
            devices,
            ports: ports.into_iter().map(|(_, _, port)| port).collect(),
            conductors,
            cables,
            binding,
        })
    }
}
