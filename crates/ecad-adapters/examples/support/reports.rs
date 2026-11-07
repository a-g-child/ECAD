//! Growing paired devices with individual cables: report construction and each CSV encoded once.
use ecad_adapters::{ConnectionCsvKind, decode_project, encode_connection_csv, encode_project};
use ecad_app::{ConnectionReport, Session};
use ecad_domain::*;
use serde_json::{Value, json};
use std::{error::Error, time::Instant};
pub(super) fn cases() -> Result<Vec<Value>, Box<dyn Error>> {
    [100, 1_000, 10_000].into_iter().map(probe).collect()
}
fn probe(count: u128) -> Result<Value, Box<dyn Error>> {
    let mut project = Project::new(ProjectId::new(1)?);
    project.insert_subsystem(Subsystem::new(SubsystemId::new(1)?, "Harness", None)?)?;
    let scope = SubsystemId::new(2)?;
    project.insert_subsystem(Subsystem::new(scope, "Zone", Some(SubsystemId::new(1)?))?)?;
    for id in 1..=count {
        project.insert_device(
            Device::new(
                DeviceId::new(id)?,
                format!("-X{id}"),
                [Terminal::new(TerminalId::new(id)?, "1")?],
            )?
            .in_subsystem(scope),
        )?;
    }
    for id in 1..=count / 2 {
        let a = id * 2 - 1;
        let b = id * 2;
        let wire = ConductorId::new(id)?;
        project.insert_conductor(Conductor::new(
            wire,
            Some(TerminalId::new(a)?),
            Some(TerminalId::new(b)?),
        ))?;
        project.insert_cable(Cable::new(
            CableId::new(id)?,
            format!("-C{id}"),
            CableEndpoint::Device(DeviceId::new(a)?),
            CableEndpoint::Device(DeviceId::new(b)?),
            [wire],
        )?)?;
    }
    project.validate()?;
    let session = Session::new(project);
    let start = Instant::now();
    let report = ConnectionReport::from_session(&session);
    let report_ns = start.elapsed().as_nanos();
    if report.devices().len() != usize::try_from(count)?
        || report
            .terminals()
            .map(|t| t.connections.len())
            .sum::<usize>()
            != usize::try_from(count)?
        || report.conductors().len() != usize::try_from(count / 2)?
        || report.cables().len() != usize::try_from(count / 2)?
    {
        return Err("report lost or duplicated actual electrical records".into());
    }
    let mut output = Vec::new();
    let mut timings = Vec::new();
    for (kind, rows) in [
        (ConnectionCsvKind::WireList, count / 2),
        (ConnectionCsvKind::TerminalTable, count),
        (ConnectionCsvKind::CableSchedule, count / 2),
    ] {
        let start = Instant::now();
        let csv = encode_connection_csv(&report, kind);
        timings.push(start.elapsed().as_nanos());
        if csv.lines().count() != usize::try_from(rows)? + 1 {
            return Err("report CSV row coverage changed".into());
        }
        output.push(csv);
    }
    let loaded = Session::new(decode_project(&encode_project(session.project())?)?);
    let loaded = ConnectionReport::from_session(&loaded);
    for (index, kind) in [
        ConnectionCsvKind::WireList,
        ConnectionCsvKind::TerminalTable,
        ConnectionCsvKind::CableSchedule,
    ]
    .into_iter()
    .enumerate()
    {
        if encode_connection_csv(&loaded, kind) != output[index] {
            return Err("electrical report output changed after project round trip".into());
        }
    }
    Ok(
        json!({"devices":count,"terminals":count,"conductors":count/2,"cables":count/2,"hierarchy_depth":2,"samples":1,"report_ns":report_ns,"wire_csv_ns":timings[0],"terminal_csv_ns":timings[1],"cable_csv_ns":timings[2],"total_csv_bytes":output.iter().map(String::len).sum::<usize>(),"scope":"full report and three in-memory CSV strings, fixed hierarchy depth, no file IO or UI",
        "correctness":"each actual device/end/core represented once at its defined row granularity; deterministic output after project round trip"}),
    )
}
