//! Experimental disk schema and explicit conversion at the persistence boundary.

use crate::ProjectFileError;
use crate::schema_structure::{
    CableData, PortData, SubsystemData, load_cables, load_ports, load_subsystems,
};
use ecad_domain::{Conductor, Device, End, Project, ProjectId, Terminal};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct VersionEnvelope {
    pub(crate) schema_version: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectData {
    pub(crate) schema_version: u32,
    pub(crate) project_id: String,
    pub(crate) subsystems: Vec<SubsystemData>,
    pub(crate) devices: Vec<DeviceData>,
    pub(crate) ports: Vec<PortData>,
    pub(crate) conductors: Vec<ConductorData>,
    pub(crate) cables: Vec<CableData>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LegacyProjectData {
    schema_version: u32,
    devices: Vec<FlatDeviceData>,
    conductors: Vec<ConductorData>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FlatProjectData {
    schema_version: u32,
    project_id: String,
    devices: Vec<FlatDeviceData>,
    conductors: Vec<ConductorData>,
}

impl FlatProjectData {
    pub(crate) fn upgrade(self) -> Result<Project, ProjectFileError> {
        if self.schema_version != 2 {
            return Err(ProjectFileError::UnsupportedVersion(self.schema_version));
        }
        flat_project(self.project_id, self.devices, self.conductors)
    }
}

impl LegacyProjectData {
    pub(crate) fn migrate(self, id: ProjectId) -> Result<Project, ProjectFileError> {
        if self.schema_version != 1 {
            return Err(ProjectFileError::UnsupportedVersion(self.schema_version));
        }
        flat_project(id.to_string(), self.devices, self.conductors)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeviceData {
    id: String,
    designation: String,
    terminals: Vec<TerminalData>,
    subsystem: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FlatDeviceData {
    id: String,
    designation: String,
    terminals: Vec<TerminalData>,
}

fn flat_project(
    project_id: String,
    devices: Vec<FlatDeviceData>,
    conductors: Vec<ConductorData>,
) -> Result<Project, ProjectFileError> {
    Project::try_from(ProjectData {
        schema_version: 3,
        project_id,
        subsystems: vec![],
        ports: vec![],
        cables: vec![],
        conductors,
        devices: devices
            .into_iter()
            .map(|d| DeviceData {
                id: d.id,
                designation: d.designation,
                terminals: d.terminals,
                subsystem: None,
            })
            .collect(),
    })
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TerminalData {
    id: String,
    number: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConductorData {
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
                subsystem: device.subsystem().map(|id| id.to_string()),
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
            schema_version: 3,
            project_id: project.id().to_string(),
            subsystems: project.subsystems().map(SubsystemData::from).collect(),
            devices,
            ports: project.ports().map(PortData::from).collect(),
            conductors,
            cables: project.cables().map(CableData::from).collect(),
        }
    }
}

impl TryFrom<ProjectData> for Project {
    type Error = ProjectFileError;

    fn try_from(data: ProjectData) -> Result<Self, Self::Error> {
        if data.schema_version != 3 {
            return Err(ProjectFileError::UnsupportedVersion(data.schema_version));
        }
        let mut project = Project::new(data.project_id.parse()?);
        load_subsystems(&mut project, data.subsystems)?;
        for device in data.devices {
            let terminals = device
                .terminals
                .into_iter()
                .map(|terminal| Ok(Terminal::new(terminal.id.parse()?, terminal.number)?))
                .collect::<Result<Vec<_>, ProjectFileError>>()?;
            let mut object = Device::new(device.id.parse()?, device.designation, terminals)?;
            if let Some(subsystem) = device.subsystem {
                object = object.in_subsystem(subsystem.parse()?);
            }
            project.insert_device(object)?;
        }
        load_ports(&mut project, data.ports)?;
        for conductor in data.conductors {
            project.insert_conductor(Conductor::new(
                conductor.id.parse()?,
                conductor.a.map(|id| id.parse()).transpose()?,
                conductor.b.map(|id| id.parse()).transpose()?,
            ))?;
        }
        load_cables(&mut project, data.cables)?;
        project.validate()?;
        Ok(project)
    }
}
