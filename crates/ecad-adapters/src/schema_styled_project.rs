//! Project v5 adds drawing content while preserving the v4 electrical layout.

use crate::{
    ProjectFileError,
    schema::{ConductorData, DeviceData},
    schema_definition::DefinitionData,
    schema_drawing::DrawingData,
    schema_reuse::{InstanceData, ReusableProjectData},
    schema_structure::{CableData, PortData, SubsystemData},
};
use ecad_domain::Project;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StyledProjectData {
    schema_version: u32,
    project_id: String,
    subsystems: Vec<SubsystemData>,
    devices: Vec<DeviceData>,
    ports: Vec<PortData>,
    conductors: Vec<ConductorData>,
    cables: Vec<CableData>,
    definitions: Vec<DefinitionData>,
    instances: Vec<InstanceData>,
    local_origins: Vec<InstanceData>,
    drawing: DrawingData,
}
impl From<&Project> for StyledProjectData {
    fn from(project: &Project) -> Self {
        let base = ReusableProjectData::from(project);
        Self {
            schema_version: 5,
            project_id: base.project_id,
            subsystems: base.subsystems,
            devices: base.devices,
            ports: base.ports,
            conductors: base.conductors,
            cables: base.cables,
            definitions: base.definitions,
            instances: base.instances,
            local_origins: base.local_origins,
            drawing: project.drawing().into(),
        }
    }
}
impl TryFrom<StyledProjectData> for Project {
    type Error = ProjectFileError;
    fn try_from(data: StyledProjectData) -> Result<Self, Self::Error> {
        if data.schema_version != 5 {
            return Err(ProjectFileError::UnsupportedVersion(data.schema_version));
        }
        let mut project = Project::try_from(ReusableProjectData {
            schema_version: 4,
            project_id: data.project_id,
            subsystems: data.subsystems,
            devices: data.devices,
            ports: data.ports,
            conductors: data.conductors,
            cables: data.cables,
            definitions: data.definitions,
            instances: data.instances,
            local_origins: data.local_origins,
        })?;
        project.replace_drawing(data.drawing.try_into()?)?;
        project.validate()?;
        Ok(project)
    }
}
