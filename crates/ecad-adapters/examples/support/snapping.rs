//! Linear snap-query growth with explicit source/radius policy and deterministic correctness.

use ecad_domain::*;
use serde_json::{Value, json};
use std::{collections::BTreeSet, error::Error, time::Instant};

pub(super) fn cases() -> Result<Vec<Value>, Box<dyn Error>> {
    [100, 1_000, 10_000].into_iter().map(probe).collect()
}
fn probe(count: u128) -> Result<Value, Box<dyn Error>> {
    const SAMPLES: usize = 100;
    let mut drawing = DrawingDocument::default();
    let view = ViewId::new(1)?;
    drawing.insert_view(DrawingView {
        id: view,
        name: "Snap sweep".into(),
        overrides: StylePatch::default(),
    })?;
    for id in 1..=count {
        let x = i64::try_from(id)? * 10_000;
        drawing.insert_polyline(DrawingPolyline {
            id: PolylineId::new(id)?,
            view,
            points: vec![DrawingPoint { x, y: 0 }, DrawingPoint { x: x + 1000, y: 0 }],
            appearance: Appearance::default(),
        })?;
    }
    let query = SnapQuery {
        view,
        position: DrawingPoint { x: 10_001, y: 1 },
        settings: SnapSettings {
            tolerance_um: 10,
            endpoints: true,
            midpoints: true,
            ..SnapSettings::default()
        },
        excluded: BTreeSet::new(),
    };
    let expected = drawing.snap_candidates(&query)?;
    if expected.len() != 1
        || expected[0].target
            != (SnapTarget::PolylineEndpoint {
                polyline: PolylineId::new(1)?,
                end: SnapEnd::First,
            })
    {
        return Err("snap sweep lost the expected endpoint".into());
    }
    let mut times = Vec::new();
    for _ in 0..SAMPLES {
        let start = Instant::now();
        let actual = drawing.snap_candidates(&query)?;
        times.push(start.elapsed().as_nanos());
        if actual != expected {
            return Err("snap query was not deterministic".into());
        }
    }
    drawing.validate()?;
    times.sort_unstable();
    Ok(
        json!({"graphical_paths":count,"samples":SAMPLES,"query_p50_ns":times[SAMPLES/2],"query_p95_ns":times[SAMPLES*95/100],"candidate_count":expected.len(),
        "scope":"linear endpoint/midpoint scan with fixed tolerance; no spatial index or renderer", "correctness":"exact expected endpoint and deterministic repeated results verified"}),
    )
}
