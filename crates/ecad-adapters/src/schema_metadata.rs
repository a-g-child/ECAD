//! Strict v7 project metadata; legacy routed payload conversion remains independent.
use crate::{
    ProjectFileError,
    schema::{ConductorData, DeviceData},
    schema_definition::DefinitionData,
    schema_reuse::InstanceData,
    schema_routes::RoutedDrawingData,
    schema_structure::{CableData, PortData, SubsystemData},
    schema_styled_project::RoutedProjectData,
};
use ecad_domain::{MetadataField, Project, ProjectMetadata};
use serde::{
    Deserialize, Serialize,
    de::{self, MapAccess, Visitor},
};
use std::{collections::BTreeMap, fmt};

#[derive(Serialize)]
#[serde(transparent)]
struct MetadataData(BTreeMap<String, String>);
impl<'de> Deserialize<'de> for MetadataData {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Fields;
        impl<'de> Visitor<'de> for Fields {
            type Value = MetadataData;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a unique map of known metadata keys to literal strings")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, String>()? {
                    if MetadataField::from_key(&key).is_none() {
                        return Err(de::Error::custom(format!("unknown metadata field {key}")));
                    }
                    if values.insert(key.clone(), value).is_some() {
                        return Err(de::Error::custom(format!("duplicate metadata field {key}")));
                    }
                }
                Ok(MetadataData(values))
            }
        }
        deserializer.deserialize_map(Fields)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetadataProjectData {
    schema_version: u32,
    project_id: String,
    metadata: MetadataData,
    subsystems: Vec<SubsystemData>,
    devices: Vec<DeviceData>,
    ports: Vec<PortData>,
    conductors: Vec<ConductorData>,
    cables: Vec<CableData>,
    definitions: Vec<DefinitionData>,
    instances: Vec<InstanceData>,
    local_origins: Vec<InstanceData>,
    drawing: RoutedDrawingData,
}
impl From<&Project> for MetadataProjectData {
    fn from(project: &Project) -> Self {
        let base = RoutedProjectData::from(project);
        Self {
            schema_version: 7,
            project_id: base.project_id,
            metadata: MetadataData(
                project
                    .metadata()
                    .values()
                    .iter()
                    .map(|(field, value)| (field.key().into(), value.clone()))
                    .collect(),
            ),
            subsystems: base.subsystems,
            devices: base.devices,
            ports: base.ports,
            conductors: base.conductors,
            cables: base.cables,
            definitions: base.definitions,
            instances: base.instances,
            local_origins: base.local_origins,
            drawing: base.drawing,
        }
    }
}
impl TryFrom<MetadataProjectData> for Project {
    type Error = ProjectFileError;
    fn try_from(data: MetadataProjectData) -> Result<Self, Self::Error> {
        if data.schema_version != 7 {
            return Err(ProjectFileError::UnsupportedVersion(data.schema_version));
        }
        let mut project = Project::try_from(RoutedProjectData {
            schema_version: 6,
            project_id: data.project_id,
            subsystems: data.subsystems,
            devices: data.devices,
            ports: data.ports,
            conductors: data.conductors,
            cables: data.cables,
            definitions: data.definitions,
            instances: data.instances,
            local_origins: data.local_origins,
            drawing: data.drawing,
        })?;
        project.replace_metadata(ProjectMetadata::new(
            data.metadata
                .0
                .into_iter()
                .map(|(key, value)| (MetadataField::from_key(&key).expect("validated key"), value))
                .collect(),
        ));
        Ok(project)
    }
}
