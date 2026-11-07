//! One strict immutable revision DTO shared by project and library formats.

use crate::{ProjectFileError, schema::ProjectData};
use ecad_domain::{DefinitionRef, Project, SubsystemRevision};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DefinitionData {
    definition: String,
    revision: String,
    root: String,
    body: ProjectData,
}
impl From<&SubsystemRevision> for DefinitionData {
    fn from(revision: &SubsystemRevision) -> Self {
        Self {
            definition: revision.source().definition().to_string(),
            revision: revision.source().revision().to_string(),
            root: revision.root().to_string(),
            body: ProjectData::from(revision.content()),
        }
    }
}
impl TryFrom<DefinitionData> for SubsystemRevision {
    type Error = ProjectFileError;
    fn try_from(data: DefinitionData) -> Result<Self, Self::Error> {
        let source = DefinitionRef::new(data.definition.parse()?, data.revision.parse()?);
        Ok(Self::new(
            source,
            data.root.parse()?,
            Project::try_from(data.body)?,
        )?)
    }
}
