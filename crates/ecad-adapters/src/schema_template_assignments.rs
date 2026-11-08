//! Strict v13 saved assignments; v12 normalization is temporary and all choices restore atomically.
use crate::{
    ProjectFileError,
    schema_documents::{TemplateData, TemplatedDocumentData},
    schema_project_templates::{DefaultsData, KindData, RoleSheetData},
    schema_sheets::{PaperData, PlacementData},
    schema_template_bundles::BundledProjectData,
    template_library_json::RevisionData,
};
use ecad_domain::*;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssignedSheetData {
    id: String,
    number: String,
    title: String,
    kind: KindData,
    template: Option<TemplateData>,
    inherited_variant: Option<String>,
    paper: PaperData,
    placement: Option<PlacementData>,
}
impl From<&DrawingSheet> for AssignedSheetData {
    fn from(sheet: &DrawingSheet) -> Self {
        let s = RoleSheetData::from(sheet);
        Self {
            id: s.id,
            number: s.number,
            title: s.title,
            kind: s.kind,
            template: s.template,
            inherited_variant: sheet.inherited_variant.map(|id| id.to_string()),
            paper: s.paper,
            placement: s.placement,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AssignedProjectData {
    schema_version: u32,
    project_id: String,
    metadata: crate::schema_metadata::MetadataData,
    template_defaults: DefaultsData,
    drawing_documents: Vec<TemplatedDocumentData>,
    sheets: Vec<AssignedSheetData>,
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
impl From<&Project> for AssignedProjectData {
    fn from(project: &Project) -> Self {
        let b = BundledProjectData::from(project);
        let mut sheets: Vec<_> = project
            .sheets()
            .iter()
            .map(AssignedSheetData::from)
            .collect();
        sheets.sort_by(|a, b| a.id.cmp(&b.id));
        Self {
            schema_version: 13,
            project_id: b.project_id,
            metadata: b.metadata,
            template_defaults: b.template_defaults,
            drawing_documents: b.drawing_documents,
            sheets,
            subsystems: b.subsystems,
            devices: b.devices,
            ports: b.ports,
            conductors: b.conductors,
            cables: b.cables,
            definitions: b.definitions,
            instances: b.instances,
            local_origins: b.local_origins,
            drawing: b.drawing,
            template_revisions: b.template_revisions,
        }
    }
}
impl TryFrom<AssignedProjectData> for Project {
    type Error = ProjectFileError;
    fn try_from(mut b: AssignedProjectData) -> Result<Self, Self::Error> {
        if b.schema_version != 13 {
            return Err(ProjectFileError::UnsupportedVersion(b.schema_version));
        }
        let defaults = DrawingTemplateDefaults {
            schematic: std::mem::replace(
                &mut b.template_defaults.schematic,
                TemplateData::StandardV1,
            )
            .into(),
            cover: std::mem::replace(&mut b.template_defaults.cover, TemplateData::StandardV1)
                .into(),
            drawing_list: std::mem::replace(
                &mut b.template_defaults.drawing_list,
                TemplateData::StandardV1,
            )
            .into(),
        };
        let mut documents = Vec::new();
        for d in &mut b.drawing_documents {
            documents.push((
                d.id.parse::<DrawingDocumentId>()?,
                SheetTemplate::from(std::mem::replace(
                    &mut d.default_template,
                    TemplateData::StandardV1,
                )),
            ));
        }
        let mut assignments = Vec::new();
        let sheets = b
            .sheets
            .into_iter()
            .map(|s| {
                let choice = s.template.map(SheetTemplate::from);
                assignments.push((s.id.clone(), choice, s.inherited_variant));
                RoleSheetData {
                    id: s.id,
                    number: s.number,
                    title: s.title,
                    kind: s.kind,
                    template: None,
                    paper: s.paper,
                    placement: s.placement,
                }
            })
            .collect();
        let mut project = Project::try_from(BundledProjectData {
            schema_version: 12,
            project_id: b.project_id,
            metadata: b.metadata,
            template_defaults: b.template_defaults,
            drawing_documents: b.drawing_documents,
            sheets,
            subsystems: b.subsystems,
            devices: b.devices,
            ports: b.ports,
            conductors: b.conductors,
            cables: b.cables,
            definitions: b.definitions,
            instances: b.instances,
            local_origins: b.local_origins,
            drawing: b.drawing,
            template_revisions: b.template_revisions,
        })?;
        let mut value = project.template_assignments();
        value.defaults = defaults;
        value.documents = documents;
        for (id, template, variant) in assignments {
            let id = id.parse::<SheetId>()?;
            let row = value
                .sheets
                .iter_mut()
                .find(|row| row.sheet == id)
                .expect("validated sheet");
            row.template = template;
            row.inherited_variant = variant.map(|v| v.parse()).transpose()?;
        }
        project.replace_template_assignments(value)?;
        Ok(project)
    }
}
