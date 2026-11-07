//! Bounded, deterministic headless scale probe; not a native UI capacity claim.

#[path = "support/actions.rs"]
mod actions;
#[path = "support/hierarchy.rs"]
mod hierarchy;
#[path = "support/properties.rs"]
mod properties;
#[path = "support/reuse.rs"]
mod reuse;
#[path = "support/routes.rs"]
mod routes;
#[path = "support/styles.rs"]
mod styles;

use ecad_adapters::{decode_project, encode_project};
use ecad_app::{Command, Session};
use ecad_domain::{Conductor, ConductorId, Device, DeviceId, End, Project, Terminal, TerminalId};
use serde_json::{Value, json};
use std::{
    error::Error,
    num::NonZeroUsize,
    process::Command as Process,
    time::{Duration, Instant},
};

const SAMPLES: usize = 1_000;

fn main() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for count in [100, 1_000, 10_000] {
        cases.push(probe(count)?);
    }
    let hierarchy_cases = hierarchy::cases()?;
    let reuse_cases = reuse::cases()?;
    let property_cases = properties::cases()?;
    let style_cases = styles::cases()?;
    let action_cases = actions::cases()?;
    let route_cases = routes::cases()?;
    let rustc = Process::new("rustc").arg("--version").output()?;
    if !rustc.status.success() {
        return Err("could not identify rustc".into());
    }
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_revision = if workspace.join(".git").exists() {
        let output = Process::new("git")
            .current_dir(&workspace)
            .args(["rev-parse", "HEAD"])
            .output()?;
        if output.status.success() {
            Some(String::from_utf8(output.stdout)?.trim().to_owned())
        } else {
            None
        }
    } else {
        None
    };
    let source_dirty = if source_revision.is_some() {
        let output = Process::new("git")
            .current_dir(&workspace)
            .args(["status", "--porcelain"])
            .output()?;
        if !output.status.success() {
            return Err("could not determine Git working-tree state".into());
        }
        Some(!output.stdout.is_empty())
    } else {
        None
    };
    let report = json!({
        "report_version": 9,
        "recipe": "electrical-routes-v9",
        "project_schema": 6,
        "seed": 0,
        "build": if cfg!(debug_assertions) { "debug" } else { "release" },
        "os": std::env::consts::OS,
        "architecture": std::env::consts::ARCH,
        "source_revision": source_revision,
        "source_dirty": source_dirty,
        "rustc": String::from_utf8(rustc.stdout)?.trim(),
        "scope": "headless engine only; no native UI, disk IO, or memory measurements",
        "cache": "fresh in-memory project per case; no persistent index",
        "cases": cases,
        "hierarchy_cases": hierarchy_cases,
        "reuse_cases": reuse_cases,
        "property_cases": property_cases,
        "style_cases": style_cases,
        "action_cases": action_cases,
        "route_cases": route_cases,
    });
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/scale");
    std::fs::create_dir_all(&root)?;
    let destination = root.join("latest.json");
    std::fs::write(&destination, serde_json::to_string_pretty(&report)? + "\n")?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    println!("Saved {}", destination.display());
    Ok(())
}

fn probe(count: u128) -> Result<Value, Box<dyn Error>> {
    let start = Instant::now();
    let mut project = Project::new(ecad_domain::ProjectId::new(1).unwrap());
    for value in 1..=count {
        let terminal = Terminal::new(TerminalId::new(value)?, "1")?;
        project.insert_device(Device::new(
            DeviceId::new(value)?,
            format!("-X{value}"),
            [terminal],
        )?)?;
        if value > 1 {
            project.insert_conductor(Conductor::new(
                ConductorId::new(value - 1)?,
                Some(TerminalId::new(value - 1)?),
                Some(TerminalId::new(value)?),
            ))?;
        }
    }
    let build_ms = millis(start.elapsed());
    let start = Instant::now();
    project.validate()?;
    let validation_ms = millis(start.elapsed());
    let start = Instant::now();
    let baseline = encode_project(&project)?;
    let encode_ms = millis(start.elapsed());
    let start = Instant::now();
    let decoded = decode_project(&baseline)?;
    let decode_ms = millis(start.elapsed());
    if decoded != project {
        return Err("scale fixture changed during round trip".into());
    }
    drop(decoded);

    let mut session = Session::with_history_limit(
        project,
        NonZeroUsize::new(SAMPLES).ok_or("sample count is zero")?,
    );
    let mut times = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let terminal = TerminalId::new(if sample % 2 == 0 { 2 } else { 1 })?;
        let start = Instant::now();
        if !session.execute(
            session.revision(),
            Command::SetTermination {
                conductor: ConductorId::new(1)?,
                end: End::A,
                terminal: Some(terminal),
            },
        )? {
            return Err("scale operation unexpectedly became a no-op".into());
        }
        times.push(start.elapsed().as_nanos());
    }
    session.project().validate()?;
    for _ in 0..SAMPLES {
        if !session.undo(session.revision())? {
            return Err("missing scale undo record".into());
        }
    }
    if encode_project(session.project())? != baseline {
        return Err("scale undo changed electrical state".into());
    }
    times.sort_unstable();
    Ok(json!({
        "devices": count,
        "terminals": count,
        "conductors": count - 1,
        "unrelated_devices_added": count - 2,
        "document_bytes": baseline.len(),
        "build_ms": build_ms,
        "validation_ms": validation_ms,
        "encode_ms": encode_ms,
        "decode_ms": decode_ms,
        "local_edit_samples": SAMPLES,
        "local_edit_p50_ns": times[SAMPLES / 2],
        "local_edit_p95_ns": times[SAMPLES * 95 / 100],
        "local_edit_max_ns": times[SAMPLES - 1],
        "correctness": "round trip, references, and complete undo verified",
    }))
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}
