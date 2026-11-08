//! Strict v9 ownership and v10 document template defaults. Sheet records are canonical; membership alone owns order.
use crate::{
    ProjectFileError,
    schema::{ConductorData, DeviceData},
    schema_definition::DefinitionData,
    schema_metadata::MetadataData,
    schema_reuse::InstanceData,
    schema_routes::RoutedDrawingData,
    schema_sheets::{SheetData, SheetProjectData},
    schema_structure::{CableData, PortData, SubsystemData},
};
use ecad_domain::*;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocumentData {
    id: String,
    number: String,
    title: String,
    revision: String,
    sheets: Vec<String>,
}
impl From<DrawingDocumentLayout> for DocumentData {
    fn from(layout: DrawingDocumentLayout) -> Self {
        let d = layout.document;
        Self {
            id: d.id.to_string(),
            number: d.number,
            title: d.title,
            revision: d.revision,
            sheets: layout.sheets.into_iter().map(|id| id.to_string()).collect(),
        }
    }
}
impl TryFrom<DocumentData> for DrawingDocumentLayout {
    type Error = ProjectFileError;
    fn try_from(d: DocumentData) -> Result<Self, Self::Error> {
        Ok(Self {
            document: ProjectDrawingDocument {
                id: d.id.parse()?,
                number: d.number,
                title: d.title,
                revision: d.revision,
                default_template: SheetTemplate::StandardV1,
            },
            sheets: d
                .sheets
                .into_iter()
                .map(|s| s.parse().map_err(ProjectFileError::from))
                .collect::<Result<_, _>>()?,
        })
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocumentProjectData<D = DocumentData> {
    schema_version: u32,
    project_id: String,
    metadata: MetadataData,
    drawing_documents: Vec<D>,
    sheets: Vec<SheetData>,
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
impl From<&Project> for DocumentProjectData<TemplatedDocumentData> {
    fn from(project: &Project) -> Self {
        let b = SheetProjectData::from(project);
        let mut sheets: Vec<_> = project.sheets().iter().collect();
        sheets.sort_by_key(|s| s.id);
        Self {
            schema_version: 10,
            project_id: b.project_id,
            metadata: b.metadata,
            drawing_documents: project
                .drawing_document_layout()
                .into_iter()
                .map(Into::into)
                .collect(),
            sheets: sheets.into_iter().map(Into::into).collect(),
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
impl<D: DocumentRecord> TryFrom<DocumentProjectData<D>> for Project {
    type Error = ProjectFileError;
    fn try_from(b: DocumentProjectData<D>) -> Result<Self, Self::Error> {
        if b.schema_version != D::VERSION {
            return Err(ProjectFileError::UnsupportedVersion(b.schema_version));
        }
        let mut project = Self::try_from(SheetProjectData {
            schema_version: 8,
            project_id: b.project_id,
            metadata: b.metadata,
            sheets: b.sheets,
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
        let layout = b
            .drawing_documents
            .into_iter()
            .map(DocumentRecord::layout)
            .collect::<Result<_, ProjectFileError>>()?;
        project
            .replace_drawing_document_layout(layout)
            .map_err(ModelError::from)?;
        project.validate()?;
        Ok(project)
    }
}

/// Separate strict document records keep unknown v9 fields rejected during upgrade.
pub(crate) trait DocumentRecord {
    const VERSION: u32;
    fn layout(self) -> Result<DrawingDocumentLayout, ProjectFileError>;
}
impl DocumentRecord for DocumentData {
    const VERSION: u32 = 9;
    fn layout(self) -> Result<DrawingDocumentLayout, ProjectFileError> {
        self.try_into()
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum TemplateData {
    StandardV1,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TemplatedDocumentData {
    id: String,
    number: String,
    title: String,
    revision: String,
    default_template: TemplateData,
    sheets: Vec<String>,
}
impl From<DrawingDocumentLayout> for TemplatedDocumentData {
    fn from(layout: DrawingDocumentLayout) -> Self {
        let default_template = match layout.document.default_template {
            SheetTemplate::StandardV1 => TemplateData::StandardV1,
        };
        let d = DocumentData::from(layout);
        Self {
            id: d.id,
            number: d.number,
            title: d.title,
            revision: d.revision,
            default_template,
            sheets: d.sheets,
        }
    }
}
impl DocumentRecord for TemplatedDocumentData {
    const VERSION: u32 = 10;
    fn layout(self) -> Result<DrawingDocumentLayout, ProjectFileError> {
        let template = match self.default_template {
            TemplateData::StandardV1 => SheetTemplate::StandardV1,
        };
        let mut layout = DrawingDocumentLayout::try_from(DocumentData {
            id: self.id,
            number: self.number,
            title: self.title,
            revision: self.revision,
            sheets: self.sheets,
        })?;
        layout.document.default_template = template;
        Ok(layout)
    }
}
