//! Unrelated route growth with fixed-degree indexed anchor movement and exact undo.

use ecad_adapters::{decode_project, encode_project};
use ecad_app::{Command, RouteCommand, Session};
use ecad_domain::*;
use serde_json::{Value, json};
use std::{error::Error, num::NonZeroUsize, time::Instant};

pub(super) fn cases() -> Result<Vec<Value>, Box<dyn Error>> {
    [100, 1_000, 10_000].into_iter().map(probe).collect()
}
fn probe(count: u128) -> Result<Value, Box<dyn Error>> {
    let mut project = Project::new(ProjectId::new(1)?);
    let view = ViewId::new(1)?;
    project.insert_drawing_view(DrawingView {
        id: view,
        name: "Route sweep".into(),
        overrides: StylePatch::default(),
    })?;
    for id in 1..=count + 1 {
        let terminal = TerminalId::new(id)?;
        project.insert_device(Device::new(
            DeviceId::new(id)?,
            format!("-X{id}"),
            [Terminal::new(terminal, "1")?],
        )?)?;
        project.insert_terminal_anchor(TerminalAnchor {
            id: AnchorId::new(id)?,
            view,
            terminal,
            position: DrawingPoint {
                x: i64::try_from(id)? * 10_000,
                y: 0,
            },
        })?;
    }
    for id in 1..=count {
        let conductor = ConductorId::new(id)?;
        project.insert_conductor(Conductor::new(
            conductor,
            Some(TerminalId::new(id)?),
            Some(TerminalId::new(id + 1)?),
        ))?;
        project.insert_drawing_route(DrawingRoute {
            id: RouteId::new(id)?,
            view,
            conductor,
            anchors: [Some(AnchorId::new(id)?), Some(AnchorId::new(id + 1)?)],
            points: vec![
                DrawingPoint {
                    x: i64::try_from(id)? * 10_000,
                    y: 0,
                },
                DrawingPoint {
                    x: i64::try_from(id + 1)? * 10_000,
                    y: 0,
                },
            ],
            mode: RouteMode::Orthogonal,
            appearance: Appearance::default(),
        })?;
    }
    project.validate()?;
    let baseline = project.clone();
    let mut session = Session::with_history_limit(
        project,
        NonZeroUsize::new(super::SAMPLES).ok_or("zero samples")?,
    );
    let anchor = AnchorId::new(2)?;
    if session
        .project()
        .drawing()
        .anchor_layout(anchor)?
        .routes
        .len()
        != 2
    {
        return Err("route sweep did not keep local degree fixed".into());
    }
    let mut times = Vec::new();
    for sample in 0..super::SAMPLES {
        let position = DrawingPoint {
            x: 20_000,
            y: if sample % 2 == 0 { 5_000 } else { 0 },
        };
        let start = Instant::now();
        session.execute(
            session.revision(),
            Command::Route(RouteCommand::MoveAnchor { anchor, position }),
        )?;
        times.push(start.elapsed().as_nanos());
    }
    if session.project() != &baseline {
        return Err("repeated movement accumulated route geometry".into());
    }
    for _ in 0..super::SAMPLES {
        if !session.undo(session.revision())? {
            return Err("route movement undo was missing".into());
        }
    }
    if session.project() != &baseline
        || decode_project(&encode_project(session.project())?)? != baseline
    {
        return Err("route sweep changed identities or failed round trip/undo".into());
    }
    times.sort_unstable();
    Ok(
        json!({"routes":count,"anchors":count+1,"touching_routes":2,"samples":super::SAMPLES,"local_move_p50_ns":times[super::SAMPLES/2],"local_move_p95_ns":times[super::SAMPLES*95/100],"local_move_max_ns":times[super::SAMPLES-1],"correctness":"fixed local degree, bounded bends, unchanged electrical data, round trip and exact undo verified"}),
    )
}
