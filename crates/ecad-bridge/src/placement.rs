//! Shared view-local cursor resolution for creation tools, never electrical attachment.
use crate::dto::BridgeError;
use ecad_app::Session;
use ecad_domain::*;
use serde::Deserialize;
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CursorInput {
    pub x: String,
    pub y: String,
    pub exact: bool,
    pub suppressed: bool,
    pub candidate: Option<usize>,
    pub tolerance_um: u32,
}
#[derive(Clone, Debug)]
pub(crate) struct Cursor {
    pub position: DrawingPoint,
    pub candidates: usize,
    pub chosen: Option<usize>,
    pub snap: Option<String>,
}
pub(crate) fn resolve(
    session: &Session,
    view: ViewId,
    input: CursorInput,
) -> Result<Cursor, BridgeError> {
    let parse = |text: &str| {
        text.parse::<i64>().map_err(|_| {
            BridgeError::new(
                "coordinate",
                "Enter a signed 64-bit integer micrometre coordinate",
            )
        })
    };
    let mut position = DrawingPoint {
        x: parse(&input.x)?,
        y: parse(&input.y)?,
    };
    let candidates = if input.exact || input.suppressed {
        vec![]
    } else {
        session
            .project()
            .drawing()
            .snap_candidates(&SnapQuery {
                view,
                position,
                excluded: Default::default(),
                settings: SnapSettings {
                    tolerance_um: input.tolerance_um,
                    terminals: true,
                    endpoints: true,
                    midpoints: true,
                    grid: Some(DrawingGrid {
                        origin: DrawingPoint { x: 0, y: 0 },
                        spacing: DrawingLength::new(5000).expect("constant"),
                    }),
                },
            })
            .map_err(|e| BridgeError::new("snap", e))?
    };
    let chosen = if input.exact || input.suppressed {
        None
    } else {
        match input.candidate {
            Some(index) if index >= candidates.len() => {
                return Err(BridgeError::new(
                    "snap",
                    "Snap candidate is no longer available",
                ));
            }
            Some(index) => Some(index),
            None => (!candidates.is_empty()).then_some(0),
        }
    };
    let snap = chosen.map(|index| {
        position = candidates[index].position;
        match candidates[index].target {
            SnapTarget::Terminal(_) => "Terminal",
            SnapTarget::RouteEndpoint { .. } | SnapTarget::PolylineEndpoint { .. } => "Endpoint",
            SnapTarget::RouteMidpoint { .. } | SnapTarget::PolylineMidpoint { .. } => "Midpoint",
            SnapTarget::Grid => "Grid",
        }
        .to_string()
    });
    Ok(Cursor {
        position,
        candidates: candidates.len(),
        chosen,
        snap,
    })
}
