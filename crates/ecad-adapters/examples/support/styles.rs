//! Shared-style propagation and intent-aware mixed graphical editing measurements.

use ecad_adapters::{decode_project, encode_project};
use ecad_app::{Command, DrawingCommand, Session, property_edit_plan};
use ecad_domain::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, error::Error, time::Instant};

pub(super) fn cases() -> Result<Vec<Value>, Box<dyn Error>> {
    [10, 100, 1_000].into_iter().map(probe).collect()
}
fn probe(count: u128) -> Result<Value, Box<dyn Error>> {
    let mut project = Project::new(ProjectId::new(1)?);
    let view = ViewId::new(1)?;
    let style = StyleId::new(1)?;
    project.insert_drawing_view(DrawingView {
        id: view,
        name: "Overview".into(),
        overrides: StylePatch::default(),
    })?;
    project.insert_drawing_style(DrawingStyle::new(
        style,
        "Annotations",
        DrawingRole::Text,
        StylePatch {
            text_height: Some(DrawingLength::new(3_000)?),
            ..StylePatch::default()
        },
    )?)?;
    let mut selection = Vec::new();
    for id in 1..=count {
        let label = LabelId::new(id)?;
        let line = PolylineId::new(id)?;
        let overrides = StylePatch {
            text_height: if id == 1 {
                Some(DrawingLength::new(3_000)?)
            } else {
                None
            },
            ..StylePatch::default()
        };
        project.insert_drawing_label(DrawingLabel {
            id: label,
            view,
            text: format!("Label {id}"),
            position: DrawingPoint::default(),
            appearance: Appearance {
                style: Some(style),
                overrides,
            },
        })?;
        project.insert_drawing_polyline(DrawingPolyline {
            id: line,
            view,
            points: vec![DrawingPoint::default(), DrawingPoint { x: 10_000, y: 0 }],
            appearance: Appearance::default(),
        })?;
        selection.extend([ObjectRef::Label(label), ObjectRef::Polyline(line)]);
    }
    let before = project.clone();
    let mut session = Session::new(project);
    let start = Instant::now();
    session.execute(
        session.revision(),
        Command::Drawing(DrawingCommand::ReplaceStyle(DrawingStyle::new(
            style,
            "Annotations",
            DrawingRole::Text,
            StylePatch {
                text_height: Some(DrawingLength::new(6_000)?),
                ..StylePatch::default()
            },
        )?)),
    )?;
    let shared_style_commit_ms = super::millis(start.elapsed());
    let start = Instant::now();
    for id in 1..=count {
        let value = session
            .project()
            .property(ObjectRef::Label(LabelId::new(id)?), PropertyId::TextHeight)?;
        let expected = if id == 1 { 3_000 } else { 6_000 };
        if value.value != PropertyValue::Length(DrawingLength::new(expected)?) {
            return Err("shared style lost inheritance/override intent".into());
        }
    }
    let resolve_ms = super::millis(start.elapsed());
    if session
        .project()
        .drawing()
        .labels()
        .ne(before.drawing().labels())
    {
        return Err("shared style rewrote dependent objects".into());
    }
    let start = Instant::now();
    let plan = property_edit_plan(
        session.project(),
        selection,
        BTreeMap::from([(
            PropertyId::DrawingColour,
            PropertyValue::Colour(DrawingColour {
                red: 0,
                green: 0,
                blue: 255,
            }),
        )]),
    )?;
    let preview_ms = super::millis(start.elapsed());
    if plan.changes.len() != usize::try_from(count * 2)? {
        return Err("mixed drawing plan lost applicable objects".into());
    }
    let start = Instant::now();
    session.execute_batch(session.revision(), plan.commands)?;
    let bulk_commit_ms = super::millis(start.elapsed());
    if decode_project(&encode_project(session.project())?)? != *session.project() {
        return Err("drawing round trip lost intent".into());
    }
    let start = Instant::now();
    session.undo(session.revision())?;
    session.undo(session.revision())?;
    let undo_ms = super::millis(start.elapsed());
    if session.project() != &before {
        return Err("style/bulk undo failed exact restoration".into());
    }
    Ok(
        json!({"drawing_objects": count * 2, "shared_style_commit_ms": shared_style_commit_ms, "resolve_ms": resolve_ms, "bulk_preview_ms": preview_ms, "bulk_commit_ms": bulk_commit_ms, "undo_ms": undo_ms, "samples": 1, "correctness": "no dependent rewrites, per-property intent, mixed edits, round trip and exact undo verified"}),
    )
}
