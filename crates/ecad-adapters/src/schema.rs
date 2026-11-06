//! Experimental disk schema and explicit conversion at the persistence boundary.

use crate::ProjectFileError;
use ecad_domain::{Conductor, Device, End, Project, ProjectId, Terminal};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct VersionEnvelope {
    pub(crate) schema_version: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectData {
    schema_version: u32,
    project_id: String,
    devices: Vec<DeviceData>,
    conductors: Vec<ConductorData>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LegacyProjectData {
    schema_version: u32,
    devices: Vec<DeviceData>,
    conductors: Vec<ConductorData>,
}

impl LegacyProjectData {
    pub(crate) fn migrate(self, id: ProjectId) -> Result<Project, ProjectFileError> {
        if self.schema_version != 1 {
            return Err(ProjectFileError::UnsupportedVersion(self.schema_version));
        }
        Project::try_from(ProjectData {
            schema_version: 2,
            project_id: id.to_string(),
            devices: self.devices,
            conductors: self.conductors,
        })
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DeviceData {
    id: String,
    designation: String,
    terminals: Vec<TerminalData>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TerminalData {
    id: String,
    number: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ConductorData {
    id: String,
    a: Option<String>,
    b: Option<String>,
}

impl From<&Project> for ProjectData {
    fn from(project: &Project) -> Self {
        let devices = project
            .devices()
            .map(|device| DeviceData {
                id: device.id().to_string(),
                designation: device.designation().to_owned(),
                terminals: device
                    .terminals()
                    .map(|terminal| TerminalData {
                        id: terminal.id().to_string(),
                        number: terminal.number().to_owned(),
                    })
                    .collect(),
            })
            .collect();
        let conductors = project
            .conductors()
            .map(|conductor| ConductorData {
                id: conductor.id().to_string(),
                a: conductor.endpoint(End::A).map(|id| id.to_string()),
                b: conductor.endpoint(End::B).map(|id| id.to_string()),
            })
            .collect();
        Self {
            schema_version: 2,
            project_id: project.id().to_string(),
            devices,
            conductors,
        }
    }
}

impl TryFrom<ProjectData> for Project {
    type Error = ProjectFileError;

    fn try_from(data: ProjectData) -> Result<Self, Self::Error> {
        let mut project = Project::new(data.project_id.parse()?);
        for device in data.devices {
            let terminals = device
                .terminals
                .into_iter()
                .map(|terminal| Ok(Terminal::new(terminal.id.parse()?, terminal.number)?))
                .collect::<Result<Vec<_>, ProjectFileError>>()?;
            project.insert_device(Device::new(
                device.id.parse()?,
                device.designation,
                terminals,
            )?)?;
        }
        for conductor in data.conductors {
            project.insert_conductor(Conductor::new(
                conductor.id.parse()?,
                conductor.a.map(|id| id.parse()).transpose()?,
                conductor.b.map(|id| id.parse()).transpose()?,
            ))?;
        }
        project.validate()?;
        Ok(project)
    }
}
