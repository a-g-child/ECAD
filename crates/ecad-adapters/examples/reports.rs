//! Derive terminal/wire/cable schedules from real electrical identities, then refresh after edits.
use ecad_adapters::{
    ConnectionCsvKind, JsonFile, decode_project, encode_connection_csv,
    encode_current_connection_csv,
};
use ecad_app::*;
use ecad_domain::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project = decode_project(include_str!(
        "../tests/fixtures/controller-conveyor-v6.json"
    ))?;
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("reports.json"));
    let mut document = Document::new(project, store.clone());
    document.save()?;
    let report = ConnectionReport::from_session(document.session());
    let wire_list =
        encode_current_connection_csv(document.session(), &report, ConnectionCsvKind::WireList)?;
    println!("{wire_list}");
    println!(
        "{} devices; {} terminals; {} conductors; {} cable assemblies",
        report.devices().len(),
        report.terminals().len(),
        report.conductors().len(),
        report.cables().len()
    );
    document.execute(
        document.session().revision(),
        Command::SetProperty {
            target: ObjectRef::Device(DeviceId::new(2)?),
            property: PropertyId::Designation,
            value: PropertyValue::Text("-Renamed junction".into()),
        },
    )?;
    if encode_current_connection_csv(document.session(), &report, ConnectionCsvKind::WireList)
        .is_ok()
    {
        return Err("stale report was accepted as current".into());
    }
    let refreshed = ConnectionReport::from_session(document.session());
    if refreshed
        .device(DeviceId::new(2)?)
        .ok_or("missing junction")?
        .designation
        != "-Renamed junction"
    {
        return Err("report used stale display text".into());
    }
    document.undo(document.session().revision())?;
    let current = ConnectionReport::from_session(document.session());
    if encode_current_connection_csv(document.session(), &current, ConnectionCsvKind::WireList)?
        != wire_list
    {
        return Err("undo did not restore identical engineering output".into());
    }
    let reopened = Document::open(store)?;
    let loaded = ConnectionReport::from_session(reopened.session());
    for kind in [
        ConnectionCsvKind::WireList,
        ConnectionCsvKind::TerminalTable,
        ConnectionCsvKind::CableSchedule,
    ] {
        if encode_connection_csv(&loaded, kind) != encode_connection_csv(&current, kind) {
            return Err("output changed after reopening equal project content".into());
        }
    }
    println!("Current-value freshness, rename/undo, deterministic CSV, and save/reopen passed.");
    Ok(())
}
