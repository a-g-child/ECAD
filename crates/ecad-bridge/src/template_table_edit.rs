//! Strict structural proposals, captured by variant/table/cell identity before dispatch.
use crate::dto::BridgeError;
use ecad_adapters::RandomIds;
use ecad_app::{ActionParameters, TableOperation, TemplateDocument};
use ecad_domain::*;
use serde::Deserialize;

/// Bound each interactive split; large documents may still contain many smaller edits.
const MAX_SPLIT_CELLS: u64 = 4096;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TableEditRow {
    variant: String,
    table: String,
    edit: Edit,
}
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Edit {
    Resize {
        bounds: [String; 4],
    },
    Split {
        cell: String,
        columns: String,
        rows: String,
    },
    Merge {
        cells: Vec<String>,
        primary: String,
        contents: String,
    },
    Boundary {
        axis: String,
        from: String,
        to: String,
    },
}
fn error(e: impl std::fmt::Display) -> BridgeError {
    BridgeError::new("template", e)
}
fn number(s: &str) -> Result<u32, BridgeError> {
    s.parse()
        .map_err(|_| error("Use nonnegative integer micrometres/counts within the supported range"))
}
fn cell(s: &str) -> Result<TemplateCellId, BridgeError> {
    s.parse().map_err(error)
}
impl TableEditRow {
    pub(crate) fn proposal(
        self,
        doc: &TemplateDocument,
        operation: TableOperation,
    ) -> Result<ActionParameters, BridgeError> {
        let id = self.variant.parse::<TemplateVariantId>().map_err(error)?;
        let draft = doc
            .variant(id)
            .ok_or_else(|| error("Template variant is unavailable"))?;
        let table_id = self.table.parse::<TemplateTableId>().map_err(error)?;
        let old = draft
            .layout()
            .tables()
            .iter()
            .find(|t| t.id() == table_id)
            .ok_or_else(|| error("Selected table is unavailable"))?;
        let changed = match (operation, self.edit) {
            (TableOperation::Resize, Edit::Resize { bounds }) => {
                let b = bounds
                    .iter()
                    .map(|v| number(v))
                    .collect::<Result<Vec<_>, _>>()?;
                old.resized(FrameRect {
                    x: b[0],
                    y: b[1],
                    width: b[2],
                    height: b[3],
                })
            }
            (
                TableOperation::Split,
                Edit::Split {
                    cell: target,
                    columns,
                    rows,
                },
            ) => {
                let columns = number(&columns)?;
                let rows = number(&rows)?;
                let count = u64::from(columns) * u64::from(rows);
                if !(2..=MAX_SPLIT_CELLS).contains(&count) {
                    return Err(error("A split must create between 2 and 4096 cells"));
                }
                let mut fresh = Vec::new();
                let mut used: std::collections::BTreeSet<_> = draft
                    .layout()
                    .tables()
                    .iter()
                    .flat_map(|t| t.cells().iter().map(|c| c.id))
                    .collect();
                for _ in 1..count {
                    let id = RandomIds::template_cell().map_err(error)?;
                    if !used.insert(id) {
                        return Err(error("Generated cell identity already exists; retry"));
                    }
                    fresh.push(id);
                }
                old.split_cell(cell(&target)?, columns, rows, &fresh)
            }
            (
                TableOperation::Merge,
                Edit::Merge {
                    cells,
                    primary,
                    contents,
                },
            ) => {
                let selected = cells
                    .iter()
                    .map(|s| cell(s))
                    .collect::<Result<Vec<_>, _>>()?;
                let policy = match contents.as_str() {
                    "combine" => MergeContents::Combine,
                    "keep_primary" => MergeContents::KeepPrimary,
                    _ => {
                        return Err(error(
                            "Choose how merged captions and contents are retained",
                        ));
                    }
                };
                old.merge_cells(&selected, cell(&primary)?, policy)
            }
            (TableOperation::Boundary, Edit::Boundary { axis, from, to }) => {
                let axis = match axis.as_str() {
                    "x" => TableAxis::X,
                    "y" => TableAxis::Y,
                    _ => return Err(error("Choose an X or Y boundary")),
                };
                old.move_boundary(axis, number(&from)?, number(&to)?)
            }
            _ => return Err(error("Table values do not match the registered operation")),
        }
        .map_err(error)?;
        let tables = draft
            .layout()
            .tables()
            .iter()
            .map(|t| {
                if t.id() == table_id {
                    changed.clone()
                } else {
                    t.clone()
                }
            })
            .collect();
        let layout = TemplateLayout::new(
            draft.layout().paper().clone(),
            draft.layout().content(),
            tables,
        )
        .map_err(error)?;
        Ok(ActionParameters::TemplateEdit {
            variant: id,
            name: draft.name().into(),
            layout: Box::new(layout),
        })
    }
}
