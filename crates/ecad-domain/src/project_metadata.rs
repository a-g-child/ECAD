//! Portable project-owned values for future sheet bindings; no machine-profile lookups.
use std::collections::BTreeMap;

/// Stable semantic field identity, separate from a template cell or its caption.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetadataField {
    /// Company identity copied into a project.
    CompanyName,
    /// Literal multiline postal address.
    CompanyAddress,
    /// Contact engineer; not a sheet's drawn/checked/approved responsibility.
    EngineerName,
    /// Human-readable project title, independent of its file name.
    ProjectTitle,
    /// Product or equipment designation.
    Product,
    /// Client or customer.
    Client,
    /// Contract reference.
    Contract,
    /// Explicit engineering issue, unrelated to Git or session revisions.
    Issue,
}
impl MetadataField {
    /// Complete catalog in stable presentation order.
    pub const ALL: [Self; 8] = [
        Self::CompanyName,
        Self::CompanyAddress,
        Self::EngineerName,
        Self::ProjectTitle,
        Self::Product,
        Self::Client,
        Self::Contract,
        Self::Issue,
    ];
    /// Stable namespaced placeholder key.
    pub fn key(self) -> &'static str {
        match self {
            Self::CompanyName => "company.name",
            Self::CompanyAddress => "company.address",
            Self::EngineerName => "engineer.name",
            Self::ProjectTitle => "project.title",
            Self::Product => "project.product",
            Self::Client => "project.client",
            Self::Contract => "project.contract",
            Self::Issue => "project.issue",
        }
    }
    /// Editor caption; never used as persistent field identity.
    pub fn label(self) -> &'static str {
        match self {
            Self::CompanyName => "Company name",
            Self::CompanyAddress => "Company address",
            Self::EngineerName => "Engineer / contact",
            Self::ProjectTitle => "Project title",
            Self::Product => "Product",
            Self::Client => "Client",
            Self::Contract => "Contract reference",
            Self::Issue => "Project issue",
        }
    }
    /// Resolve a known key without guessing unknown bindings.
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|field| field.key() == key)
    }
}
/// Values carried with the document. Empty strings are absent; other text is literal.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectMetadata(BTreeMap<MetadataField, String>);
impl ProjectMetadata {
    /// Create a value set, omitting empty entries and preserving multiline text exactly.
    pub fn new(values: BTreeMap<MetadataField, String>) -> Self {
        Self(
            values
                .into_iter()
                .filter(|(_, value)| !value.is_empty())
                .collect(),
        )
    }
    /// Resolve a field; absence stays distinct from a successful nonempty binding.
    pub fn value(&self, field: MetadataField) -> Option<&str> {
        self.0.get(&field).map(String::as_str)
    }
    /// Stored values in canonical field order.
    pub fn values(&self) -> &BTreeMap<MetadataField, String> {
        &self.0
    }
}
impl crate::Project {
    /// Inspect portable project metadata without reading personal settings.
    pub fn metadata(&self) -> &ProjectMetadata {
        &self.metadata
    }
    /// Replace project metadata, returning exact previous values for undo.
    pub fn replace_metadata(&mut self, metadata: ProjectMetadata) -> ProjectMetadata {
        std::mem::replace(&mut self.metadata, metadata)
    }
}
