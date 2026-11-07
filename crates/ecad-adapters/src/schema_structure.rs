//! Hierarchy/cable disk values and deterministic dependency-ordered loading.

use crate::ProjectFileError;
use ecad_domain::{
    Cable, CableEndpoint, CableId, End, InterfacePort, ModelError, Project, Subsystem, SubsystemId,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SubsystemData {
    id: String,
    name: String,
    parent: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PortData {
    id: String,
    subsystem: String,
    name: String,
    terminal: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CableData {
    id: String,
    designation: String,
    a: EndpointData,
    b: EndpointData,
    conductors: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum EndpointData {
    Device(String),
    Subsystem(String),
}

impl From<CableEndpoint> for EndpointData {
    fn from(value: CableEndpoint) -> Self {
        match value {
            CableEndpoint::Device(id) => Self::Device(id.to_string()),
            CableEndpoint::Subsystem(id) => Self::Subsystem(id.to_string()),
        }
    }
}
impl TryFrom<EndpointData> for CableEndpoint {
    type Error = ProjectFileError;
    fn try_from(value: EndpointData) -> Result<Self, Self::Error> {
        Ok(match value {
            EndpointData::Device(id) => Self::Device(id.parse()?),
            EndpointData::Subsystem(id) => Self::Subsystem(id.parse()?),
        })
    }
}

impl From<&Subsystem> for SubsystemData {
    fn from(s: &Subsystem) -> Self {
        Self {
            id: s.id().to_string(),
            name: s.name().to_owned(),
            parent: s.parent().map(|id| id.to_string()),
        }
    }
}
impl From<&InterfacePort> for PortData {
    fn from(p: &InterfacePort) -> Self {
        Self {
            id: p.id().to_string(),
            subsystem: p.subsystem().to_string(),
            name: p.name().to_owned(),
            terminal: p.terminal().to_string(),
        }
    }
}
impl From<&Cable> for CableData {
    fn from(c: &Cable) -> Self {
        Self {
            id: c.id().to_string(),
            designation: c.designation().to_owned(),
            a: c.endpoint(End::A).into(),
            b: c.endpoint(End::B).into(),
            conductors: c.conductors().map(|id| id.to_string()).collect(),
        }
    }
}

pub(crate) fn load_subsystems(
    project: &mut Project,
    data: Vec<SubsystemData>,
) -> Result<(), ProjectFileError> {
    let mut nodes = BTreeMap::new();
    let mut children: BTreeMap<Option<SubsystemId>, BTreeSet<SubsystemId>> = BTreeMap::new();
    for value in data {
        let node = Subsystem::new(
            value.id.parse()?,
            value.name,
            value.parent.map(|id| id.parse()).transpose()?,
        )?;
        children.entry(node.parent()).or_default().insert(node.id());
        let id = node.id();
        if nodes.insert(id, node).is_some() {
            return Err(ModelError::DuplicateSubsystem(id).into());
        }
    }
    for node in nodes.values() {
        if let Some(parent) = node.parent()
            && !nodes.contains_key(&parent)
        {
            return Err(ModelError::MissingSubsystem(parent).into());
        }
    }
    let mut stack: Vec<_> = children
        .get(&None)
        .into_iter()
        .flat_map(|ids| ids.iter().rev().copied())
        .collect();
    let mut visited = BTreeSet::new();
    while let Some(id) = stack.pop() {
        let node = nodes.get(&id).expect("child index references parsed nodes");
        project.insert_subsystem(node.clone())?;
        visited.insert(id);
        if let Some(ids) = children.get(&Some(id)) {
            stack.extend(ids.iter().rev().copied());
        }
    }
    if let Some(id) = nodes.keys().find(|id| !visited.contains(id)) {
        return Err(ModelError::HierarchyCycle(*id).into());
    }
    Ok(())
}

pub(crate) fn load_ports(
    project: &mut Project,
    data: Vec<PortData>,
) -> Result<(), ProjectFileError> {
    let mut ports = Vec::new();
    for value in data {
        let port = InterfacePort::new(
            value.id.parse()?,
            value.subsystem.parse()?,
            value.name,
            value.terminal.parse()?,
        )?;
        let depth = project.subsystem_path(port.subsystem())?.len();
        ports.push((Reverse(depth), port.id(), port));
    }
    ports.sort_by_key(|(depth, id, _)| (*depth, *id));
    for (_, _, port) in ports {
        project.insert_port(port)?;
    }
    Ok(())
}

pub(crate) fn load_cables(
    project: &mut Project,
    data: Vec<CableData>,
) -> Result<(), ProjectFileError> {
    for value in data {
        let id: CableId = value.id.parse()?;
        let conductors = value
            .conductors
            .into_iter()
            .map(|id| id.parse())
            .collect::<Result<Vec<_>, _>>()?;
        project.insert_cable(Cable::new(
            id,
            value.designation,
            value.a.try_into()?,
            value.b.try_into()?,
            conductors,
        )?)?;
    }
    Ok(())
}
