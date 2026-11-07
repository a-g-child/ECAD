//! V4 persistence bundles immutable local bodies once and pins instance mappings.

use crate::schema_definition::DefinitionData;
use crate::{
    ProjectFileError,
    schema::{ConductorData, DeviceData, ProjectData},
    schema_structure::{CableData, PortData, SubsystemData},
};
use ecad_domain::{DefinitionRef, InstanceBinding, InstanceIds, ModelError, Project, ReuseError};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Display, str::FromStr, sync::Arc};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReusableProjectData {
    pub(crate) schema_version: u32,
    pub(crate) project_id: String,
    pub(crate) subsystems: Vec<SubsystemData>,
    pub(crate) devices: Vec<DeviceData>,
    pub(crate) ports: Vec<PortData>,
    pub(crate) conductors: Vec<ConductorData>,
    pub(crate) cables: Vec<CableData>,
    pub(crate) definitions: Vec<DefinitionData>,
    pub(crate) instances: Vec<InstanceData>,
    pub(crate) local_origins: Vec<InstanceData>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstanceData {
    root: String,
    definition: String,
    revision: String,
    ids: MappingData,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pair {
    local: String,
    placed: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MappingData {
    subsystems: Vec<Pair>,
    devices: Vec<Pair>,
    terminals: Vec<Pair>,
    ports: Vec<Pair>,
    conductors: Vec<Pair>,
    cables: Vec<Pair>,
}

fn pairs<K: Display>(map: &BTreeMap<K, K>) -> Vec<Pair> {
    map.iter()
        .map(|(local, placed)| Pair {
            local: local.to_string(),
            placed: placed.to_string(),
        })
        .collect()
}
fn mapping<K: Ord + FromStr<Err = ecad_domain::IdError>>(
    pairs: Vec<Pair>,
) -> Result<BTreeMap<K, K>, ProjectFileError> {
    let mut map = BTreeMap::new();
    for pair in pairs {
        if map
            .insert(pair.local.parse()?, pair.placed.parse()?)
            .is_some()
        {
            return Err(ModelError::from(ReuseError::InvalidMapping).into());
        }
    }
    Ok(map)
}

impl From<&InstanceIds> for MappingData {
    fn from(ids: &InstanceIds) -> Self {
        Self {
            subsystems: pairs(&ids.subsystems),
            devices: pairs(&ids.devices),
            terminals: pairs(&ids.terminals),
            ports: pairs(&ids.ports),
            conductors: pairs(&ids.conductors),
            cables: pairs(&ids.cables),
        }
    }
}
impl TryFrom<MappingData> for InstanceIds {
    type Error = ProjectFileError;
    fn try_from(data: MappingData) -> Result<Self, Self::Error> {
        Ok(Self {
            subsystems: mapping(data.subsystems)?,
            devices: mapping(data.devices)?,
            terminals: mapping(data.terminals)?,
            ports: mapping(data.ports)?,
            conductors: mapping(data.conductors)?,
            cables: mapping(data.cables)?,
        })
    }
}

impl From<&Project> for ReusableProjectData {
    fn from(project: &Project) -> Self {
        let core = ProjectData::from(project);
        let definitions = project.definitions().map(DefinitionData::from).collect();
        let instances = project
            .instances()
            .map(|instance| InstanceData {
                root: instance.root().to_string(),
                definition: instance.source().definition().to_string(),
                revision: instance.source().revision().to_string(),
                ids: instance.ids().into(),
            })
            .collect();
        let local_origins = project
            .local_origins()
            .map(|origin| InstanceData {
                root: origin.root().to_string(),
                definition: origin.source().definition().to_string(),
                revision: origin.source().revision().to_string(),
                ids: origin.ids().into(),
            })
            .collect();
        Self {
            schema_version: 4,
            project_id: core.project_id,
            subsystems: core.subsystems,
            devices: core.devices,
            ports: core.ports,
            conductors: core.conductors,
            cables: core.cables,
            definitions,
            instances,
            local_origins,
        }
    }
}

impl TryFrom<ReusableProjectData> for Project {
    type Error = ProjectFileError;
    fn try_from(data: ReusableProjectData) -> Result<Self, Self::Error> {
        if data.schema_version != 4 {
            return Err(ProjectFileError::UnsupportedVersion(data.schema_version));
        }
        let core = ProjectData {
            schema_version: 3,
            project_id: data.project_id,
            subsystems: data.subsystems,
            devices: data.devices,
            ports: data.ports,
            conductors: data.conductors,
            cables: data.cables,
        };
        let mut project = Project::try_from(core)?;
        for definition in data.definitions {
            let revision = ecad_domain::SubsystemRevision::try_from(definition)?;
            let source = revision.source();
            if project.definition(source).is_some() {
                return Err(ModelError::from(ReuseError::DuplicateRevision(source)).into());
            }
            project.insert_definition(Arc::new(revision))?;
        }
        for instance in data.instances {
            let source =
                DefinitionRef::new(instance.definition.parse()?, instance.revision.parse()?);
            project.bind_instance(InstanceBinding::new(
                instance.root.parse()?,
                source,
                instance.ids.try_into()?,
            ))?;
        }
        for origin in data.local_origins {
            let source = DefinitionRef::new(origin.definition.parse()?, origin.revision.parse()?);
            project.remember_local_origin(InstanceBinding::new(
                origin.root.parse()?,
                source,
                origin.ids.try_into()?,
            ))?;
        }
        project.validate()?;
        Ok(project)
    }
}
