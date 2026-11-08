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
    Pointer {
        drag: PointerDrag,
        snap_tolerance: Option<String>,
    },
    Insert {
        axis: String,
        at: String,
        size: String,
        anchor: String,
    },
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
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum PointerDrag {
    Move { x: String, y: String },
    Resize { edge: String, coordinate: String },
}
fn signed(s: &str) -> Result<i64, BridgeError> {
    s.parse()
        .map_err(|_| error("Use signed integer micrometres within the supported range"))
}
fn margin_edge(s: &str) -> Result<MarginEdge, BridgeError> {
    match s {
        "left" => Ok(MarginEdge::Left),
        "right" => Ok(MarginEdge::Right),
        "top" => Ok(MarginEdge::Top),
        "bottom" => Ok(MarginEdge::Bottom),
        _ => Err(error("Choose an outer table edge")),
    }
}
fn margin_key(edge: MarginEdge) -> String {
    match edge {
        MarginEdge::Left => "left",
        MarginEdge::Right => "right",
        MarginEdge::Top => "top",
        MarginEdge::Bottom => "bottom",
    }
    .into()
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
    pub(crate) fn target(&self) -> Vec<String> {
        let mut target = vec![self.variant.clone(), self.table.clone()];
        match &self.edit {
            Edit::Pointer { drag, .. } => match drag {
                PointerDrag::Move { .. } => target.push("pointer_move".into()),
                PointerDrag::Resize { edge, .. } => {
                    target.extend(["pointer_resize".into(), edge.clone()])
                }
            },
            Edit::Resize { .. } => target.push("resize".into()),
            Edit::Boundary { axis, from, .. } => {
                target.extend(["boundary".into(), axis.clone(), from.clone()])
            }
            Edit::Insert {
                axis, at, anchor, ..
            } => target.extend(["insert".into(), axis.clone(), at.clone(), anchor.clone()]),
            Edit::Split { cell, .. } => target.extend(["split".into(), cell.clone()]),
            Edit::Merge { cells, primary, .. } => {
                target.extend(["merge".into(), primary.clone()]);
                let mut cells = cells.clone();
                cells.sort();
                target.extend(cells);
            }
        }
        target
    }
    pub(crate) fn proposal(
        self,
        doc: &TemplateDocument,
        operation: TableOperation,
    ) -> Result<ActionParameters, BridgeError> {
        self.proposal_with_snap(doc, operation)
            .map(|(parameters, _)| parameters)
    }
    pub(crate) fn proposal_with_snap(
        self,
        doc: &TemplateDocument,
        operation: TableOperation,
    ) -> Result<(ActionParameters, Vec<String>), BridgeError> {
        let mut snapped = vec![];
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
            (
                TableOperation::Resize,
                Edit::Pointer {
                    drag,
                    snap_tolerance,
                },
            ) => {
                let drag = match drag {
                    PointerDrag::Move { x, y } => TableBorderDrag::Move {
                        x: signed(&x)?,
                        y: signed(&y)?,
                    },
                    PointerDrag::Resize { edge, coordinate } => TableBorderDrag::Resize {
                        edge: margin_edge(&edge)?,
                        coordinate: signed(&coordinate)?,
                    },
                };
                let tolerance = snap_tolerance.map(|s| number(&s)).transpose()?;
                let placement =
                    table_border_placement(old.bounds(), draft.layout().border(), drag, tolerance)
                        .map_err(error)?;
                snapped = placement.snapped.into_iter().map(margin_key).collect();
                old.resized(placement.bounds)
            }
            (
                op @ (TableOperation::InsertRow | TableOperation::InsertColumn),
                Edit::Insert {
                    axis,
                    at,
                    size,
                    anchor,
                },
            ) => {
                let axis = match (op, axis.as_str()) {
                    (TableOperation::InsertRow, "y") => TableAxis::Y,
                    (TableOperation::InsertColumn, "x") => TableAxis::X,
                    _ => return Err(error("Insertion axis does not match the registered action")),
                };
                let at = number(&at)?;
                let count = old.insertion_cell_count(axis, at).map_err(error)?;
                if count > 4096 {
                    return Err(error("Insert at most 4096 blank cells in one operation"));
                }
                let mut used: std::collections::BTreeSet<_> = draft
                    .layout()
                    .tables()
                    .iter()
                    .flat_map(|t| t.cells().iter().map(|c| c.id))
                    .collect();
                let mut fresh = vec![];
                for _ in 0..count {
                    let id = RandomIds::template_cell().map_err(error)?;
                    if !used.insert(id) {
                        return Err(error("Generated cell identity already exists; retry"));
                    }
                    fresh.push(id);
                }
                let anchor = match anchor.as_str() {
                    "start" => TableGrowthAnchor::Start,
                    "end" => TableGrowthAnchor::End,
                    _ => return Err(error("Choose a start or end growth anchor")),
                };
                old.insert_band(axis, at, number(&size)?, anchor, &fresh)
            }
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
        .and_then(|layout| layout.with_graphics(draft.layout().graphics().to_vec()))
        .map_err(error)?;
        Ok((
            ActionParameters::TemplateEdit {
                variant: id,
                name: draft.name().into(),
                layout: Box::new(layout),
            },
            snapped,
        ))
    }
}
