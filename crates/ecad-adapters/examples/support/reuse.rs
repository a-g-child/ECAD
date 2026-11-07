//! Repeated-instance placement and local interface-edit scaling, with complete validation.

use ecad_adapters::{decode_project, encode_project};
use ecad_app::{Command, Session, adoption_plan, instance_commands};
use ecad_domain::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, error::Error, num::NonZeroUsize, sync::Arc, time::Instant};

fn ids(offset: u128) -> Result<InstanceIds, Box<dyn Error>> {
    Ok(InstanceIds {
        subsystems: [10, 20]
            .map(|id| Ok((SubsystemId::new(id)?, SubsystemId::new(id + offset)?)))
            .into_iter()
            .collect::<Result<BTreeMap<_, _>, IdError>>()?,
        devices: [2, 3]
            .map(|id| Ok((DeviceId::new(id)?, DeviceId::new(id + offset)?)))
            .into_iter()
            .collect::<Result<BTreeMap<_, _>, IdError>>()?,
        terminals: [201, 301]
            .map(|id| Ok((TerminalId::new(id)?, TerminalId::new(id + offset)?)))
            .into_iter()
            .collect::<Result<BTreeMap<_, _>, IdError>>()?,
        ports: [1, 2]
            .map(|id| {
                Ok((
                    InterfacePortId::new(id)?,
                    InterfacePortId::new(id + offset)?,
                ))
            })
            .into_iter()
            .collect::<Result<BTreeMap<_, _>, IdError>>()?,
        conductors: BTreeMap::from([(ConductorId::new(2)?, ConductorId::new(offset + 2)?)]),
        cables: BTreeMap::from([(CableId::new(2)?, CableId::new(offset + 2)?)]),
    })
}

pub(super) fn cases() -> Result<Vec<Value>, Box<dyn Error>> {
    let mut cases = Vec::new();
    for count in [1, 10, 100] {
        cases.push(probe(count)?);
    }
    Ok(cases)
}

fn probe(count: usize) -> Result<Value, Box<dyn Error>> {
    let source = DefinitionRef::new(DefinitionId::new(1)?, DefinitionRevisionId::new(1)?);
    let body = decode_project(include_str!("../../tests/fixtures/sensor-module-v3.json"))?;
    let revision = Arc::new(SubsystemRevision::new(source, SubsystemId::new(10)?, body)?);
    let mut project = Project::new(ProjectId::new(77)?);
    project.insert_definition(revision)?;
    let mut session = Session::with_history_limit(
        project,
        NonZeroUsize::new(super::SAMPLES).ok_or("zero samples")?,
    );
    let start = Instant::now();
    for index in 0..count {
        let commands = instance_commands(
            session.project(),
            source,
            ids(((index + 1) * 1_000) as u128)?,
            None,
            &format!("Module {index}"),
            InstanceLimits::default(),
        )?;
        session.execute_batch(session.revision(), commands)?;
    }
    let placement_ms = super::millis(start.elapsed());
    let external = ConductorId::new(99)?;
    let exposed = TerminalId::new(1_201)?;
    session.execute(
        session.revision(),
        Command::InsertConductor(Conductor::new(external, Some(exposed), None)),
    )?;
    let start = Instant::now();
    session.project().validate()?;
    let validation_ms = super::millis(start.elapsed());
    let baseline = encode_project(session.project())?;
    let start = Instant::now();
    let decoded = decode_project(&baseline)?;
    let decode_ms = super::millis(start.elapsed());
    if decoded != *session.project() || decoded.definition_count() != 1 {
        return Err("repeated instance round trip changed identities or pins".into());
    }
    let mut times = Vec::with_capacity(super::SAMPLES);
    for sample in 0..super::SAMPLES {
        let terminal = if sample % 2 == 0 { None } else { Some(exposed) };
        let start = Instant::now();
        if !session.execute(
            session.revision(),
            Command::SetTermination {
                conductor: external,
                end: End::A,
                terminal,
            },
        )? {
            return Err("interface edit became a no-op".into());
        }
        times.push(start.elapsed().as_nanos());
    }
    for _ in 0..super::SAMPLES {
        if !session.undo(session.revision())? {
            return Err("missing reusable-instance undo".into());
        }
    }
    if encode_project(session.project())? != baseline {
        return Err("reusable-instance undo changed the project".into());
    }
    let adoption = probe_adoption(&mut session, source)?;
    if encode_project(session.project())? != baseline {
        return Err("adoption/import undo changed the project".into());
    }
    times.sort_unstable();
    Ok(json!({
        "instances": count, "bundled_revisions": session.project().definition_count(),
        "placed_devices": session.project().devices().len(), "placement_ms": placement_ms,
        "validation_ms": validation_ms, "decode_ms": decode_ms, "document_bytes": baseline.len(),
        "local_edit_samples": super::SAMPLES, "local_edit_p50_ns": times[super::SAMPLES / 2],
        "local_edit_p95_ns": times[super::SAMPLES * 95 / 100], "local_edit_max_ns": times[super::SAMPLES - 1],
        "adoption": adoption,
        "correctness": "independent mappings, interface edits, round trip, one-instance adoption and complete undo verified"
    }))
}

fn probe_adoption(session: &mut Session, source: DefinitionRef) -> Result<Value, Box<dyn Error>> {
    let original = session
        .project()
        .definition(source)
        .ok_or("missing revision")?;
    let mut body = original.content().clone();
    body.set_termination(ConductorId::new(2)?, End::B, None)?;
    let target = DefinitionRef::new(source.definition(), DefinitionRevisionId::new(2)?);
    let revision = Arc::new(SubsystemRevision::new(target, original.root(), body)?);
    session.execute(session.revision(), Command::InsertDefinition(revision))?;
    let before = session.project().clone();
    let root = SubsystemId::new(1_010)?;
    let start = Instant::now();
    let plan = adoption_plan(
        session.project(),
        root,
        target,
        ids(1_000)?,
        InstanceLimits::default(),
    )?;
    let preview_ms = super::millis(start.elapsed());
    let start = Instant::now();
    session.execute(session.revision(), plan.command)?;
    let commit_ms = super::millis(start.elapsed());
    if session
        .project()
        .instances()
        .filter(|pin| pin.source() == target)
        .count()
        != 1
        || session
            .project()
            .conductor(ConductorId::new(99)?)
            .and_then(|wire| wire.endpoint(End::A))
            != Some(TerminalId::new(1_201)?)
        || decode_project(&encode_project(session.project())?)? != *session.project()
    {
        return Err("adoption changed other pins, external wiring, or round-trip content".into());
    }
    let start = Instant::now();
    session.undo(session.revision())?;
    let undo_ms = super::millis(start.elapsed());
    if session.project() != &before {
        return Err("adoption undo failed to restore exact project".into());
    }
    session.undo(session.revision())?; // Remove the otherwise unused imported revision.
    Ok(json!({"preview_ms": preview_ms, "commit_ms": commit_ms, "undo_ms": undo_ms, "samples": 1}))
}
