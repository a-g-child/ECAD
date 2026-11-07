//! Measure origin-layout preflight separately from commit/undo over growing selections.

use ecad_adapters::{decode_project, encode_project};
use ecad_app::{LayoutAxis, LayoutOperation, Session, layout_plan};
use ecad_domain::*;
use serde_json::{Value, json};
use std::{error::Error, time::Instant};

pub(super) fn cases() -> Result<Vec<Value>, Box<dyn Error>> {
    [20, 200, 2_000].into_iter().map(probe).collect()
}
fn probe(count: u128) -> Result<Value, Box<dyn Error>> {
    let mut project = Project::new(ProjectId::new(1)?);
    let view = ViewId::new(1)?;
    project.insert_drawing_view(DrawingView {
        id: view,
        name: "Origin layout".into(),
        overrides: StylePatch::default(),
    })?;
    let mut selection = Vec::new();
    for id in 1..=count {
        let target = LabelId::new(id)?;
        project.insert_drawing_label(DrawingLabel {
            id: target,
            view,
            text: format!("Label {id}"),
            position: DrawingPoint {
                x: i64::try_from(id)? * 37,
                y: i64::try_from(id)? * 11,
            },
            appearance: Appearance::default(),
        })?;
        selection.push(ObjectRef::Label(target));
    }
    let before = project.clone();
    let mut session = Session::new(project);
    let start = Instant::now();
    let plan = layout_plan(
        session.project(),
        selection,
        LayoutOperation::Align {
            axis: LayoutAxis::Y,
            coordinate_um: 500,
        },
    )?;
    let preflight_ns = start.elapsed().as_nanos();
    if plan.changes.len() != usize::try_from(count)? || !plan.affected_routes.is_empty() {
        return Err("layout probe did not cover its full selection".into());
    }
    let start = Instant::now();
    session.execute_batch(session.revision(), plan.commands)?;
    let commit_ns = start.elapsed().as_nanos();
    if session
        .project()
        .drawing()
        .labels()
        .any(|label| label.position.y != 500)
    {
        return Err("alignment missed a selected origin".into());
    }
    if decode_project(&encode_project(session.project())?)? != *session.project() {
        return Err("layout did not round trip".into());
    }
    let start = Instant::now();
    session.undo(session.revision())?;
    let undo_ns = start.elapsed().as_nanos();
    if session.project() != &before {
        return Err("layout undo changed the document".into());
    }
    Ok(
        json!({"selected_annotations":count, "preflight_ns":preflight_ns, "commit_ns":commit_ns, "undo_ns":undo_ns,
        "samples":1, "scope":"origin alignment; preflight includes a temporary full-project candidate",
        "correctness":"all origins aligned; round trip and exact single-transaction undo verified"}),
    )
}
