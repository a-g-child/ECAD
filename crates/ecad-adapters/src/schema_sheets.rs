//! Project-v8 paper data, with strict v7 metadata/electrical conversion retained independently.
use crate::{
    ProjectFileError,
    schema::{ConductorData, DeviceData},
    schema_definition::DefinitionData,
    schema_metadata::{MetadataData, MetadataProjectData},
    schema_reuse::InstanceData,
    schema_routes::RoutedDrawingData,
    schema_structure::{CableData, PortData, SubsystemData},
};
use ecad_domain::*;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PaperData {
    width_um: u32,
    height_um: u32,
    margins_um: [u32; 4],
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PlacementData {
    view: String,
    paper_origin_um: [i64; 2],
    view_origin_um: [i64; 2],
    scale: [u32; 2],
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SheetData {
    pub(crate) id: String,
    pub(crate) number: String,
    pub(crate) title: String,
    pub(crate) paper: PaperData,
    pub(crate) placement: Option<PlacementData>,
}
impl From<&DrawingSheet> for SheetData {
    fn from(s: &DrawingSheet) -> Self {
        Self {
            id: s.id.to_string(),
            number: s.number.clone(),
            title: s.title.clone(),
            paper: PaperData {
                width_um: s.paper.width,
                height_um: s.paper.height,
                margins_um: [s.paper.left, s.paper.right, s.paper.top, s.paper.bottom],
            },
            placement: s.placement.as_ref().map(|p| PlacementData {
                view: p.view.to_string(),
                paper_origin_um: [p.paper_origin.x, p.paper_origin.y],
                view_origin_um: [p.view_origin.x, p.view_origin.y],
                scale: [p.numerator, p.denominator],
            }),
        }
    }
}
impl TryFrom<SheetData> for DrawingSheet {
    type Error = ProjectFileError;
    fn try_from(s: SheetData) -> Result<Self, Self::Error> {
        let placement = s
            .placement
            .map(|p| {
                Ok::<_, ProjectFileError>(SheetViewPlacement {
                    view: p.view.parse()?,
                    paper_origin: DrawingPoint {
                        x: p.paper_origin_um[0],
                        y: p.paper_origin_um[1],
                    },
                    view_origin: DrawingPoint {
                        x: p.view_origin_um[0],
                        y: p.view_origin_um[1],
                    },
                    numerator: p.scale[0],
                    denominator: p.scale[1],
                })
            })
            .transpose()?;
        let sheet = Self {
            kind: SheetKind::Schematic,
            template: None,
            id: s.id.parse()?,
            number: s.number,
            title: s.title,
            paper: SheetPaper {
                width: s.paper.width_um,
                height: s.paper.height_um,
                left: s.paper.margins_um[0],
                right: s.paper.margins_um[1],
                top: s.paper.margins_um[2],
                bottom: s.paper.margins_um[3],
            },
            placement,
        };
        sheet.validate().map_err(ModelError::from)?;
        Ok(sheet)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SheetProjectData {
    pub(crate) schema_version: u32,
    pub(crate) project_id: String,
    pub(crate) metadata: MetadataData,
    pub(crate) sheets: Vec<SheetData>,
    pub(crate) subsystems: Vec<SubsystemData>,
    pub(crate) devices: Vec<DeviceData>,
    pub(crate) ports: Vec<PortData>,
    pub(crate) conductors: Vec<ConductorData>,
    pub(crate) cables: Vec<CableData>,
    pub(crate) definitions: Vec<DefinitionData>,
    pub(crate) instances: Vec<InstanceData>,
    pub(crate) local_origins: Vec<InstanceData>,
    pub(crate) drawing: RoutedDrawingData,
}
impl From<&Project> for SheetProjectData {
    fn from(project: &Project) -> Self {
        let b = MetadataProjectData::from(project);
        Self {
            schema_version: 8,
            project_id: b.project_id,
            metadata: b.metadata,
            sheets: project.sheets().iter().map(Into::into).collect(),
            subsystems: b.subsystems,
            devices: b.devices,
            ports: b.ports,
            conductors: b.conductors,
            cables: b.cables,
            definitions: b.definitions,
            instances: b.instances,
            local_origins: b.local_origins,
            drawing: b.drawing,
        }
    }
}
impl TryFrom<SheetProjectData> for Project {
    type Error = ProjectFileError;
    fn try_from(b: SheetProjectData) -> Result<Self, Self::Error> {
        if b.schema_version != 8 {
            return Err(ProjectFileError::UnsupportedVersion(b.schema_version));
        }
        let mut project = Self::try_from(MetadataProjectData {
            schema_version: 7,
            project_id: b.project_id,
            metadata: b.metadata,
            subsystems: b.subsystems,
            devices: b.devices,
            ports: b.ports,
            conductors: b.conductors,
            cables: b.cables,
            definitions: b.definitions,
            instances: b.instances,
            local_origins: b.local_origins,
            drawing: b.drawing,
        })?;
        for (index, sheet) in b.sheets.into_iter().enumerate() {
            project
                .insert_sheet(sheet.try_into()?, index)
                .map_err(ModelError::from)?;
        }
        project.validate()?;
        Ok(project)
    }
}
