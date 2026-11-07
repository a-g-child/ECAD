//! Synthetic registry growth and hot-key routing, without native UI timing claims.

use ecad_app::*;
use ecad_domain::{Project, ProjectId};
use serde_json::{Value, json};
use std::{error::Error, num::NonZeroU64, time::Instant};

pub(super) fn cases() -> Result<Vec<Value>, Box<dyn Error>> {
    [100, 1_000, 10_000].into_iter().map(probe).collect()
}
fn probe(count: usize) -> Result<Value, Box<dyn Error>> {
    let mut registry = ActionRegistry::builtins();
    let template = registry
        .get(&ActionId::new("workspace.palette")?)
        .ok_or("missing palette")?
        .clone();
    let start = Instant::now();
    for number in 0..count {
        let mut extra = template.clone();
        extra.id = ActionId::new(format!("probe.action_{number}"))?;
        extra.label = format!("Synthetic operation {number}");
        extra.description = "Synthetic registry scale entry, not a shipped user operation".into();
        extra.defaults = DefaultBindings::Unbound;
        registry.register(extra)?;
    }
    let registration_ms = super::millis(start.elapsed());
    let start = Instant::now();
    let map = compile_keymap(
        &registry,
        &KeymapOverrides::default(),
        ShortcutPlatform::Linux,
    )
    .map_err(|issues| format!("invalid scale keymap: {issues:?}"))?;
    let compile_ms = super::millis(start.elapsed());
    let session = Session::new(Project::new(ProjectId::new(1)?));
    let context = ActionContext::for_session(&session);
    let start = Instant::now();
    let matches = registry.search("synthetic", &context, Some(&session));
    let search_ms = super::millis(start.elapsed());
    if matches.len() != count {
        return Err("palette dropped synthetic actions".into());
    }
    let registered = registry.definitions().len();
    let bindings = map.bindings().len();
    let mut router = ActionInputRouter::new(map, NonZeroU64::new(1_000).ok_or("zero timeout")?);
    let target = ActionId::new("drawing.label.create")?;
    let mut times = Vec::new();
    for sample in 0..super::SAMPLES {
        let number = u64::try_from(sample)? + 1;
        let event = ActionKeyEvent {
            id: InputId::new(number).ok_or("zero event ID")?,
            chord: ResolvedChord {
                key: ShortcutKey::character('t')?,
                modifiers: PhysicalModifiers::default(),
            },
            repeat: false,
        };
        let start = Instant::now();
        let KeyRoute::Invoke(invocation) = router.route(&registry, event, &context, number) else {
            return Err("direct-key routing failed".into());
        };
        times.push(start.elapsed().as_nanos());
        if invocation.action != target {
            return Err("router changed the action target".into());
        }
    }
    times.sort_unstable();
    Ok(
        json!({"synthetic_actions": count, "registered_actions": registered, "effective_bindings": bindings, "registration_ms": registration_ms, "compile_ms": compile_ms, "palette_search_ms": search_ms, "route_samples": super::SAMPLES, "route_p50_ns": times[super::SAMPLES / 2], "route_p95_ns": times[super::SAMPLES * 95 / 100], "correctness": "all synthetic entries discoverable; direct keys retain their target; no document mutation or native timing measured"}),
    )
}
