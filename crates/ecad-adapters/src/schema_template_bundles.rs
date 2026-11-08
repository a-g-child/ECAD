//! Strict v12 project-owned template bodies; legacy v11 fields retain their exact contracts.
use crate::{
    ProjectFileError,
    schema_documents::TemplatedDocumentData,
    schema_project_templates::{CurrentProjectData, DefaultsData, RoleSheetData},
    template_library_json::RevisionData,
};
use ecad_domain::{ModelError, Project, TemplateLibraryError, TemplateRevision};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundledProjectData {
    schema_version: u32,
    project_id: String,
    metadata: crate::schema_metadata::MetadataData,
    template_defaults: DefaultsData,
    drawing_documents: Vec<TemplatedDocumentData>,
    sheets: Vec<RoleSheetData>,
    subsystems: Vec<crate::schema_structure::SubsystemData>,
    devices: Vec<crate::schema::DeviceData>,
    ports: Vec<crate::schema_structure::PortData>,
    conductors: Vec<crate::schema::ConductorData>,
    cables: Vec<crate::schema_structure::CableData>,
    definitions: Vec<crate::schema_definition::DefinitionData>,
    instances: Vec<crate::schema_reuse::InstanceData>,
    local_origins: Vec<crate::schema_reuse::InstanceData>,
    drawing: crate::schema_routes::RoutedDrawingData,
    template_revisions: Vec<RevisionData>,
}
impl From<&Project> for BundledProjectData {
    fn from(project: &Project) -> Self {
        let b = CurrentProjectData::from(project);
        Self {
            schema_version: 12,
            project_id: b.project_id,
            metadata: b.metadata,
            template_defaults: b.template_defaults,
            drawing_documents: b.drawing_documents,
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
            template_revisions: project
                .template_revisions()
                .map(RevisionData::from)
                .collect(),
        }
    }
}
impl TryFrom<BundledProjectData> for Project {
    type Error = ProjectFileError;
    fn try_from(b: BundledProjectData) -> Result<Self, Self::Error> {
        if b.schema_version != 12 {
            return Err(ProjectFileError::UnsupportedVersion(b.schema_version));
        }
        let mut project = Project::try_from(CurrentProjectData {
            schema_version: 11,
            project_id: b.project_id,
            metadata: b.metadata,
            template_defaults: b.template_defaults,
            drawing_documents: b.drawing_documents,
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
        for data in b.template_revisions {
            let revision = TemplateRevision::try_from(data)
                .map_err(|error| ProjectFileError::TemplateBundle(Box::new(error)))?;
            if project.template_revision(revision.reference()).is_some() {
                return Err(ModelError::from(TemplateLibraryError::DuplicateRevision(
                    revision.reference(),
                ))
                .into());
            }
            project
                .insert_template_revision(Arc::new(revision))
                .map_err(ModelError::from)?;
        }
        project.validate()?;
        Ok(project)
    }
}
