//! Exact saved-template resolution and atomic assignment snapshots; source geometry is untouched.
use crate::*;
use std::{collections::BTreeSet, error::Error, fmt};
/// One sheet's frame choice and inherited variant, including its exact paper projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetTemplateAssignment {
    /// Existing stable sheet identity.
    pub sheet: SheetId,
    /// Explicit template override, or owning-document inheritance.
    pub template: Option<SheetTemplate>,
    /// Stable variant within an inherited saved document revision.
    pub inherited_variant: Option<TemplateVariantId>,
    /// Validated read-only projection of the saved variant, or exact legacy paper for undo.
    pub paper: SheetPaper,
}
/// Complete assignment state, used for migration/adoption and exact undo without electrical copies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateAssignments {
    /// Project creation choices; changes affect future sheets only.
    pub defaults: DrawingTemplateDefaults,
    /// One exact default for every logical document.
    pub documents: Vec<(DrawingDocumentId, SheetTemplate)>,
    /// One assignment for every sheet.
    pub sheets: Vec<SheetTemplateAssignment>,
}
/// Invalid saved-template reference, inheritance or paper ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateAssignmentError {
    /// A retained invalid legacy layout must be repaired before new assignment.
    Retained,
    /// An exact saved variant is absent from the project bundle.
    Missing(TemplateVariantRef),
    /// Sheet paper differs from its saved layout's exact dimensions/margins.
    PaperMismatch,
    /// An explicit override cannot simultaneously select an inherited variant.
    Inheritance,
    /// A complete assignment snapshot has missing, repeated or foreign targets.
    Membership,
}
impl fmt::Display for TemplateAssignmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retained => f.write_str(
                "Repair this retained legacy layout in a new saved revision before assigning it",
            ),
            Self::Missing(pin) => write!(
                f,
                "Saved template {} / {} / {} is unavailable",
                pin.definition, pin.revision, pin.variant
            ),
            Self::PaperMismatch => f.write_str(
                "Paper and margins belong to the saved template; select a saved variant",
            ),
            Self::Inheritance => f.write_str(
                "An inherited paper variant requires an inherited saved document template",
            ),
            Self::Membership => f.write_str(
                "Template assignments must address every document and sheet exactly once",
            ),
        }
    }
}
impl Error for TemplateAssignmentError {}
impl Project {
    /// Resolve an exact bundled variant without consulting an external catalogue.
    /// # Errors
    /// Missing bodies/variants reject visibly; there is no legacy fallback.
    pub fn saved_template(
        &self,
        pin: TemplateVariantRef,
    ) -> Result<&SavedTemplateVariant, TemplateAssignmentError> {
        self.template_revision(pin.revision_ref())
            .and_then(|r| r.variant(pin.variant))
            .ok_or(TemplateAssignmentError::Missing(pin))
    }
    /// Resolve a sheet's effective exact saved pin, preserving document inheritance and variant ID.
    pub fn sheet_template_pin(
        &self,
        sheet: &DrawingSheet,
        default: SheetTemplate,
    ) -> Option<TemplateVariantRef> {
        match sheet.template.unwrap_or(default) {
            SheetTemplate::Saved(mut pin) => {
                if sheet.template.is_none() {
                    pin.variant = sheet.inherited_variant.unwrap_or(pin.variant);
                }
                Some(pin)
            }
            _ => None,
        }
    }
    /// Validate layout ownership and resolve paper/table content independently of field values.
    /// # Errors
    /// Missing saved bodies, conflicting inheritance or a changed paper projection are rejected.
    pub fn sheet_template_layout(
        &self,
        sheet: &DrawingSheet,
        default: SheetTemplate,
    ) -> Result<TemplateLayout, SheetError> {
        if sheet.inherited_variant.is_some()
            && (sheet.template.is_some() || !matches!(default, SheetTemplate::Saved(_)))
        {
            return Err(TemplateAssignmentError::Inheritance.into());
        }
        if let Some(pin) = self.sheet_template_pin(sheet, default) {
            let layout = self.saved_template(pin)?.layout();
            if layout.paper() != &sheet.paper {
                return Err(TemplateAssignmentError::PaperMismatch.into());
            }
            Ok(layout.clone())
        } else {
            sheet.template.unwrap_or(default).layout(&sheet.paper)
        }
    }
    pub(crate) fn check_template_default(
        &self,
        choice: SheetTemplate,
    ) -> Result<(), TemplateAssignmentError> {
        self.check_template_choice(choice)?;
        if let SheetTemplate::Saved(pin) = choice
            && !self.saved_template(pin)?.allows_new_assignments()
        {
            return Err(TemplateAssignmentError::Retained);
        }
        Ok(())
    }
    pub(crate) fn check_template_choice(
        &self,
        choice: SheetTemplate,
    ) -> Result<(), TemplateAssignmentError> {
        if let SheetTemplate::Saved(pin) = choice {
            self.saved_template(pin)?;
        }
        Ok(())
    }
    /// Inspect all exact template/default assignments and legacy paper intent for reversible edits.
    pub fn template_assignments(&self) -> TemplateAssignments {
        TemplateAssignments {
            defaults: self.template_defaults().clone(),
            documents: self
                .drawing_documents()
                .iter()
                .map(|d| (d.id, d.default_template))
                .collect(),
            sheets: self
                .sheets()
                .iter()
                .map(|s| SheetTemplateAssignment {
                    sheet: s.id,
                    template: s.template,
                    inherited_variant: s.inherited_variant,
                    paper: s.paper.clone(),
                })
                .collect(),
        }
    }
    /// Atomically replace all assignments, validating every pin and paper before mutation.
    /// This metadata transition uses a validation candidate; undo retains assignment fields only.
    /// # Errors
    /// Missing/duplicate targets, invalid references and paper/inheritance errors leave all state unchanged.
    pub fn replace_template_assignments(
        &mut self,
        value: TemplateAssignments,
    ) -> Result<TemplateAssignments, ModelError> {
        let mut candidate = self.clone();
        let mut docs = BTreeSet::new();
        let mut sheets = BTreeSet::new();
        for (id, choice) in value.documents {
            if !docs.insert(id) {
                return Err(TemplateAssignmentError::Membership.into());
            }
            let doc = candidate
                .drawing_documents
                .iter_mut()
                .find(|d| d.id == id)
                .ok_or(TemplateAssignmentError::Membership)?;
            doc.default_template = choice;
        }
        for row in value.sheets {
            if !sheets.insert(row.sheet) {
                return Err(TemplateAssignmentError::Membership.into());
            }
            let sheet = candidate
                .sheets
                .iter_mut()
                .find(|s| s.id == row.sheet)
                .ok_or(TemplateAssignmentError::Membership)?;
            sheet.template = row.template;
            sheet.inherited_variant = row.inherited_variant;
            sheet.paper = row.paper;
        }
        if docs.len() != candidate.drawing_documents.len() || sheets.len() != candidate.sheets.len()
        {
            return Err(TemplateAssignmentError::Membership.into());
        }
        candidate.template_defaults = value.defaults;
        candidate.validate()?;
        let old = self.template_assignments();
        self.template_defaults = candidate.template_defaults;
        self.drawing_documents = candidate.drawing_documents;
        self.sheets = candidate.sheets;
        Ok(old)
    }
}
impl From<TemplateAssignmentError> for SheetError {
    fn from(e: TemplateAssignmentError) -> Self {
        Self::Template(Box::new(e))
    }
}
impl From<TemplateAssignmentError> for DrawingDocumentError {
    fn from(e: TemplateAssignmentError) -> Self {
        Self::Template(Box::new(e))
    }
}
impl From<TemplateAssignmentError> for ModelError {
    fn from(e: TemplateAssignmentError) -> Self {
        Self::Sheet(e.into())
    }
}
