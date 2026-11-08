//! Project-owned creation defaults and sheet roles, separate from electrical/drawing topology.
use crate::{Project, SheetTemplate};
/// Content role, independent of its chosen paper-frame template.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SheetKind {
    /// Editable source view; legacy unplaced/blank sheets also keep this role.
    #[default]
    Schematic,
    /// Project/drawing information without a schematic source.
    Cover,
    /// Generated numbered navigation table, without a schematic source.
    DrawingList,
}
impl SheetKind {
    /// Resolve a known role without inferring it from title or view presence.
    pub fn from_key(key: &str) -> Option<Self> {
        [Self::Schematic, Self::Cover, Self::DrawingList]
            .into_iter()
            .find(|k| k.key() == key)
    }
    /// Stable semantic key, unrelated to the editable drawing title.
    pub fn key(self) -> &'static str {
        match self {
            Self::Schematic => "schematic",
            Self::Cover => "cover",
            Self::DrawingList => "drawing-list",
        }
    }
}
/// Pinned bundled templates copied into newly created pages; existing pages retain their choice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingTemplateDefaults {
    /// Default frame for new schematic sheets.
    pub schematic: SheetTemplate,
    /// Default frame for new cover sheets.
    pub cover: SheetTemplate,
    /// Default frame for new generated drawing-list sheets.
    pub drawing_list: SheetTemplate,
}
impl Default for DrawingTemplateDefaults {
    fn default() -> Self {
        Self {
            schematic: SheetTemplate::StandardV1,
            cover: SheetTemplate::CoverV1,
            drawing_list: SheetTemplate::DrawingListV1,
        }
    }
}
impl DrawingTemplateDefaults {
    /// Resolve creation intent without changing existing sheet overrides or document defaults.
    pub fn for_kind(&self, kind: SheetKind) -> SheetTemplate {
        match kind {
            SheetKind::Schematic => self.schematic,
            SheetKind::Cover => self.cover,
            SheetKind::DrawingList => self.drawing_list,
        }
    }
}
impl Project {
    /// Inspect portable project defaults, not the current user's personal profile.
    pub fn template_defaults(&self) -> &DrawingTemplateDefaults {
        &self.template_defaults
    }
    /// Replace creation defaults, returning exact prior choices for undo. Does not restyle sheets.
    /// # Errors
    /// Saved choices must resolve in the project and be eligible for new assignments.
    pub fn replace_template_defaults(
        &mut self,
        value: DrawingTemplateDefaults,
    ) -> Result<DrawingTemplateDefaults, crate::TemplateAssignmentError> {
        for choice in [value.schematic, value.cover, value.drawing_list] {
            self.check_template_default(choice)?;
        }
        Ok(std::mem::replace(&mut self.template_defaults, value))
    }
}
