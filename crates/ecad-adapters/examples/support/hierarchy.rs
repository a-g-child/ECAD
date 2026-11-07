//! Local-edit scaling with unrelated blocks and deep explicitly exposed interfaces.

use ecad_adapters::{decode_project, encode_project};
use ecad_app::{Command, Session};
use ecad_domain::{
    Cable, CableEndpoint, CableId, Conductor, ConductorId, Device, DeviceId, End, InterfacePort,
    InterfacePortId, Project, ProjectId, Subsystem, SubsystemId, Terminal, TerminalId,
};
use serde_json::{Value, json};
use std::{error::Error, num::NonZeroUsize, time::Instant};

pub(super) fn cases() -> Result<Vec<Value>, Box<dyn Error>> {
    let mut cases = Vec::new();
    for unrelated in [100, 1_000, 10_000] {
        cases.push(probe(3, unrelated)?);
    }
    for depth in [16, 128, ecad_domain::MAX_SUBSYSTEM_DEPTH] {
        cases.push(probe(depth, 0)?);
    }
    Ok(cases)
}

fn probe(depth: usize, unrelated: usize) -> Result<Value, Box<dyn Error>> {
    let mut project = Project::new(ProjectId::new(1)?);
    for index in 1..=depth {
        let parent = if index == 1 {
            None
        } else {
            Some(SubsystemId::new((index - 1) as u128)?)
        };
        project.insert_subsystem(Subsystem::new(
            SubsystemId::new(index as u128)?,
            "Layer",
            parent,
        )?)?;
    }
    for index in 1..=unrelated {
        project.insert_subsystem(Subsystem::new(
            SubsystemId::new((index + 1_000) as u128)?,
            "Unrelated",
            None,
        )?)?;
    }
    for index in 1..=3 {
        let mut device = Device::new(
            DeviceId::new(index)?,
            format!("-X{index}"),
            [Terminal::new(TerminalId::new(index)?, "1")?],
        )?;
        if index > 1 {
            device = device.in_subsystem(SubsystemId::new(depth as u128)?);
        }
        project.insert_device(device)?;
    }
    for layer in (1..=depth).rev() {
        for terminal in [2, 3] {
            project.insert_port(InterfacePort::new(
                InterfacePortId::new((layer as u128) * 10 + terminal)?,
                SubsystemId::new(layer as u128)?,
                format!("T{terminal}"),
                TerminalId::new(terminal)?,
            )?)?;
        }
    }
    let conductor = ConductorId::new(1)?;
    project.insert_conductor(Conductor::new(
        conductor,
        Some(TerminalId::new(1)?),
        Some(TerminalId::new(2)?),
    ))?;
    project.insert_cable(Cable::new(
        CableId::new(1)?,
        "-C1",
        CableEndpoint::Device(DeviceId::new(1)?),
        CableEndpoint::Subsystem(SubsystemId::new(1)?),
        [conductor],
    )?)?;
    let start = Instant::now();
    project.validate()?;
    let validation_ms = super::millis(start.elapsed());
    let baseline = encode_project(&project)?;
    if decode_project(&baseline)? != project {
        return Err("hierarchy round trip changed the fixture".into());
    }
    let mut session = Session::with_history_limit(
        project,
        NonZeroUsize::new(super::SAMPLES).ok_or("zero samples")?,
    );
    let mut times = Vec::with_capacity(super::SAMPLES);
    for sample in 0..super::SAMPLES {
        let terminal = TerminalId::new(if sample % 2 == 0 { 3 } else { 2 })?;
        let start = Instant::now();
        if !session.execute(
            session.revision(),
            Command::SetTermination {
                conductor,
                end: End::B,
                terminal: Some(terminal),
            },
        )? {
            return Err("expected a changed core termination".into());
        }
        times.push(start.elapsed().as_nanos());
    }
    session.project().validate()?;
    for _ in 0..super::SAMPLES {
        if !session.undo(session.revision())? {
            return Err("missing hierarchy undo".into());
        }
    }
    if encode_project(session.project())? != baseline {
        return Err("hierarchy undo lost electrical state".into());
    }
    times.sort_unstable();
    Ok(json!({
        "depth": depth, "unrelated_subsystems": unrelated, "subsystems": depth + unrelated,
        "devices": 3, "conductors": 1, "cables": 1, "ports": depth * 2,
        "validation_ms": validation_ms, "document_bytes": baseline.len(), "local_edit_samples": super::SAMPLES,
        "local_edit_p50_ns": times[super::SAMPLES / 2], "local_edit_p95_ns": times[super::SAMPLES * 95 / 100], "local_edit_max_ns": times[super::SAMPLES - 1],
        "correctness": "nested boundaries, cable endpoints, round trip and complete undo verified"
    }))
}
