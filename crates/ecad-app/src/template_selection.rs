//! Typed template selection and structural eligibility shared by all action surfaces.
use crate::{TableOperation, TemplateDocument};
use ecad_domain::*;
/// Captured editor target; never persisted into a template definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateSelection {
    /// Paper/template properties.
    Paper,
    /// Explicit cells in one variant and table.
    Cells {
        /// Owning variant.
        variant: TemplateVariantId,
        /// Owning table.
        table: TemplateTableId,
        /// Distinct selected cell identities.
        cells: Vec<TemplateCellId>,
    },
    /// Graphical artwork properties.
    Artwork {
        /// Owning variant.
        variant: TemplateVariantId,
        /// Graphical identity.
        id: TemplateGraphicId,
    },
}
impl TemplateSelection {
    /// Check selection identities against the current document without changing it.
    pub fn exists(&self, document: &TemplateDocument) -> bool {
        match self {
            Self::Paper => true,
            Self::Cells {
                variant,
                table,
                cells,
            } => document.variant(*variant).is_some_and(|v| {
                let unique: std::collections::BTreeSet<_> = cells.iter().collect();
                !cells.is_empty()
                    && unique.len() == cells.len()
                    && v.layout().tables().iter().any(|t| {
                        t.id() == *table
                            && cells.iter().all(|id| t.cells().iter().any(|c| c.id == *id))
                    })
            }),
            Self::Artwork { variant, id } => document
                .variant(*variant)
                .is_some_and(|v| v.layout().graphics().iter().any(|g| g.id == *id)),
        }
    }
    /// Shared Split/Merge eligibility. Other table actions keep their explicit numeric targets.
    pub fn allows(&self, document: &TemplateDocument, operation: TableOperation) -> bool {
        let Self::Cells {
            variant,
            table,
            cells,
        } = self
        else {
            return false;
        };
        let Some(table) = document
            .variant(*variant)
            .and_then(|v| v.layout().tables().iter().find(|t| t.id() == *table))
        else {
            return false;
        };
        match operation {
            TableOperation::Split => table.split_column(cells).is_ok(),
            TableOperation::Merge => cells.first().is_some_and(|id| {
                table
                    .merge_cells(cells, *id, MergeContents::KeepPrimary)
                    .is_ok()
            }),
            _ => true,
        }
    }
}
