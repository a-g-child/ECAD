//! Strict v11 project creation-template defaults and sheet content roles; legacy DTOs stay strict.
use crate::{
    ProjectFileError,
    schema_documents::{DocumentProjectData, TemplateData, TemplatedDocumentData},
    schema_sheets::{PaperData, PlacementData, SheetData},
};
use ecad_domain::*;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DefaultsData {
    pub(crate) schematic: TemplateData,
    pub(crate) cover: TemplateData,
    pub(crate) drawing_list: TemplateData,
}
impl From<&DrawingTemplateDefaults> for DefaultsData {
    fn from(t: &DrawingTemplateDefaults) -> Self {
        Self {
            schematic: t.schematic.into(),
            cover: t.cover.into(),
            drawing_list: t.drawing_list.into(),
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum KindData {
    Schematic,
    Cover,
    DrawingList,
}
impl From<SheetKind> for KindData {
    fn from(k: SheetKind) -> Self {
        match k {
            SheetKind::Schematic => Self::Schematic,
            SheetKind::Cover => Self::Cover,
            SheetKind::DrawingList => Self::DrawingList,
        }
    }
}
impl From<KindData> for SheetKind {
    fn from(k: KindData) -> Self {
        match k {
            KindData::Schematic => Self::Schematic,
            KindData::Cover => Self::Cover,
            KindData::DrawingList => Self::DrawingList,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoleSheetData {
    pub(crate) id: String,
    pub(crate) number: String,
    pub(crate) title: String,
    pub(crate) kind: KindData,
    pub(crate) template: Option<TemplateData>,
    pub(crate) paper: PaperData,
    pub(crate) placement: Option<PlacementData>,
}
impl From<&DrawingSheet> for RoleSheetData {
    fn from(s: &DrawingSheet) -> Self {
        let b = SheetData::from(s);
        Self {
            id: b.id,
            number: b.number,
            title: b.title,
            kind: s.kind.into(),
            template: s.template.map(Into::into),
            paper: b.paper,
            placement: b.placement,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CurrentProjectData {
    pub(crate) schema_version: u32,
    pub(crate) project_id: String,
    pub(crate) metadata: crate::schema_metadata::MetadataData,
    pub(crate) template_defaults: DefaultsData,
    pub(crate) drawing_documents: Vec<TemplatedDocumentData>,
    pub(crate) sheets: Vec<RoleSheetData>,
    pub(crate) subsystems: Vec<crate::schema_structure::SubsystemData>,
    pub(crate) devices: Vec<crate::schema::DeviceData>,
    pub(crate) ports: Vec<crate::schema_structure::PortData>,
    pub(crate) conductors: Vec<crate::schema::ConductorData>,
    pub(crate) cables: Vec<crate::schema_structure::CableData>,
    pub(crate) definitions: Vec<crate::schema_definition::DefinitionData>,
    pub(crate) instances: Vec<crate::schema_reuse::InstanceData>,
    pub(crate) local_origins: Vec<crate::schema_reuse::InstanceData>,
    pub(crate) drawing: crate::schema_routes::RoutedDrawingData,
}
impl From<&Project> for CurrentProjectData {
    fn from(p: &Project) -> Self {
        let b = DocumentProjectData::<TemplatedDocumentData>::from(p);
        let mut sheets: Vec<_> = p.sheets().iter().collect();
        sheets.sort_by_key(|s| s.id);
        Self {
            schema_version: 11,
            project_id: b.project_id,
            metadata: b.metadata,
            template_defaults: p.template_defaults().into(),
            drawing_documents: b.drawing_documents,
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
impl TryFrom<CurrentProjectData> for Project {
    type Error = ProjectFileError;
    fn try_from(b: CurrentProjectData) -> Result<Self, Self::Error> {
        if b.schema_version != 11 {
            return Err(ProjectFileError::UnsupportedVersion(b.schema_version));
        }
        if b.template_defaults.schematic.is_saved()
            || b.template_defaults.cover.is_saved()
            || b.template_defaults.drawing_list.is_saved()
            || b.sheets
                .iter()
                .any(|s| s.template.as_ref().is_some_and(TemplateData::is_saved))
        {
            return Err(ProjectFileError::TemplateVersion(11));
        }
        let mut roles = Vec::new();
        let sheets = b
            .sheets
            .into_iter()
            .map(|s| {
                roles.push((s.id.clone(), s.kind, s.template));
                SheetData {
                    id: s.id,
                    number: s.number,
                    title: s.title,
                    paper: s.paper,
                    placement: s.placement,
                }
            })
            .collect();
        let mut project = Project::try_from(DocumentProjectData::<TemplatedDocumentData> {
            schema_version: 10,
            project_id: b.project_id,
            metadata: b.metadata,
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
        })?;
        for (id, kind, template) in roles {
            let mut sheet = project.sheet(id.parse()?).expect("validated sheet").clone();
            sheet.kind = kind.into();
            sheet.template = template.map(Into::into);
            project.replace_sheet(sheet).map_err(ModelError::from)?;
        }
        project
            .replace_template_defaults(DrawingTemplateDefaults {
                schematic: b.template_defaults.schematic.into(),
                cover: b.template_defaults.cover.into(),
                drawing_list: b.template_defaults.drawing_list.into(),
            })
            .map_err(ModelError::from)?;
        project.validate()?;
        Ok(project)
    }
}
