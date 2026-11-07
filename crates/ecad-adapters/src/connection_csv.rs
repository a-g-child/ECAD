//! Deterministic electrical snapshot CSV; output is derived and never a project storage format.

use ecad_app::{ConnectionReport, ConnectionState, EditError, Session};
use ecad_domain::*;
use std::collections::BTreeMap;

/// Explicit row semantics for the initial electrical engineering outputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionCsvKind {
    /// One row per actual conductor, including partially or wholly free drafts.
    WireList,
    /// One row per attached conductor end, or one row for an unconnected terminal.
    TerminalTable,
    /// One row per cable core, or one row for an empty assembly; retains overview and actual ends.
    CableSchedule,
}

/// Encode an explicitly chosen immutable report, including historical snapshots.
///
/// UTF-8, fixed headers, quoted cells with doubled embedded quotes, and LF record
/// endings produce stable diffs. Embedded field newlines remain literal quoted data.
/// Hierarchy IDs are slash-separated; names are a compact JSON array, avoiding
/// ambiguity from names containing path separators. Session epochs/revisions,
/// timestamps, paths, and drawing data are intentionally absent from output.
/// Strings are preserved as data without spreadsheet-specific interpretation.
pub fn encode_connection_csv(report: &ConnectionReport, kind: ConnectionCsvKind) -> String {
    let scopes = Scopes::new(report);
    match kind {
        ConnectionCsvKind::WireList => wires(report, &scopes),
        ConnectionCsvKind::TerminalTable => terminals(report, &scopes),
        ConnectionCsvKind::CableSchedule => cables(report, &scopes),
    }
}

/// Encode only a report still current for the supplied document session.
/// Performs no file IO or document mutation. Use the unguarded encoder deliberately
/// when exporting an identified historical snapshot rather than current live values.
/// # Errors
/// Rejects stale/different-session reports, including after undo or reopening equal contents.
pub fn encode_current_connection_csv(
    session: &Session,
    report: &ConnectionReport,
    kind: ConnectionCsvKind,
) -> Result<String, EditError> {
    if !report.is_current(session) {
        return Err(EditError::StaleRevision {
            expected: report.revision(),
            actual: session.revision(),
        });
    }
    Ok(encode_connection_csv(report, kind))
}

fn row(output: &mut String, cells: impl IntoIterator<Item = impl AsRef<str>>) {
    let mut first = true;
    for cell in cells {
        if !first {
            output.push(',');
        }
        first = false;
        output.push('"');
        for character in cell.as_ref().chars() {
            if character == '"' {
                output.push('"');
            }
            output.push(character);
        }
        output.push('"');
    }
    output.push('\n');
}

/// Cache display paths per functional scope, not per conductor end or cable core.
struct Scopes(BTreeMap<Option<SubsystemId>, [String; 2]>);
impl Scopes {
    fn new(report: &ConnectionReport) -> Self {
        let mut values = BTreeMap::from([(None, [String::new(), "[]".into()])]);
        for subsystem in report.subsystems() {
            let path = report
                .subsystem_path(subsystem.id)
                .expect("report-owned valid scope");
            let ids = path
                .iter()
                .map(|s| s.id.to_string())
                .collect::<Vec<_>>()
                .join("/");
            let names: Vec<_> = path.iter().map(|s| s.name.as_str()).collect();
            values.insert(
                Some(subsystem.id),
                [
                    ids,
                    serde_json::to_string(&names).expect("string arrays serialize"),
                ],
            );
        }
        Self(values)
    }
    fn get(&self, id: Option<SubsystemId>) -> [String; 2] {
        self.0[&id].clone()
    }
}
fn state(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::BothAttached => "both_attached",
        ConnectionState::AFree => "a_free",
        ConnectionState::BFree => "b_free",
        ConnectionState::BothFree => "both_free",
    }
}
fn terminal_fields(
    report: &ConnectionReport,
    scopes: &Scopes,
    id: Option<TerminalId>,
) -> [String; 6] {
    let Some(id) = id else {
        return std::array::from_fn(|_| String::new());
    };
    let terminal = report.terminal(id).expect("validated terminal reference");
    let device = report
        .device(terminal.device)
        .expect("validated device owner");
    let [scope_ids, scope_names] = scopes.get(device.subsystem);
    [
        id.to_string(),
        device.id.to_string(),
        device.designation.clone(),
        terminal.number.clone(),
        scope_ids,
        scope_names,
    ]
}
fn cable_fields(report: &ConnectionReport, id: Option<CableId>) -> [String; 2] {
    id.map_or_else(
        || [String::new(), String::new()],
        |id| {
            [
                id.to_string(),
                report
                    .cable(id)
                    .expect("validated membership")
                    .designation
                    .clone(),
            ]
        },
    )
}
fn overview_fields(
    report: &ConnectionReport,
    scopes: &Scopes,
    endpoint: CableEndpoint,
) -> [String; 5] {
    let (kind, id, name, scope) = match endpoint {
        CableEndpoint::Device(id) => {
            let device = report.device(id).expect("validated overview device");
            (
                "device",
                id.to_string(),
                device.designation.clone(),
                device.subsystem,
            )
        }
        CableEndpoint::Subsystem(id) => {
            let block = report.subsystem(id).expect("validated overview block");
            ("subsystem", id.to_string(), block.name.clone(), Some(id))
        }
    };
    let [scope_ids, scope_names] = scopes.get(scope);
    [kind.into(), id, name, scope_ids, scope_names]
}
fn wires(report: &ConnectionReport, scopes: &Scopes) -> String {
    let mut output = String::new();
    row(
        &mut output,
        [
            "project_id",
            "conductor_id",
            "connection_state",
            "cable_id",
            "cable_designation",
            "a_terminal_id",
            "a_device_id",
            "a_device_designation",
            "a_terminal_number",
            "a_scope_ids",
            "a_scope_names_json",
            "b_terminal_id",
            "b_device_id",
            "b_device_designation",
            "b_terminal_number",
            "b_scope_ids",
            "b_scope_names_json",
        ],
    );
    for wire in report.conductors() {
        let mut cells = vec![
            report.project_id().to_string(),
            wire.id.to_string(),
            state(wire.state()).into(),
        ];
        cells.extend(cable_fields(report, wire.cable));
        cells.extend(terminal_fields(report, scopes, wire.endpoints[0]));
        cells.extend(terminal_fields(report, scopes, wire.endpoints[1]));
        row(&mut output, cells);
    }
    output
}
fn terminals(report: &ConnectionReport, scopes: &Scopes) -> String {
    let mut output = String::new();
    row(
        &mut output,
        [
            "project_id",
            "device_id",
            "device_designation",
            "device_scope_ids",
            "device_scope_names_json",
            "terminal_id",
            "terminal_number",
            "connection_state",
            "conductor_id",
            "local_end",
            "cable_id",
            "cable_designation",
            "destination_terminal_id",
            "destination_device_id",
            "destination_device_designation",
            "destination_terminal_number",
            "destination_scope_ids",
            "destination_scope_names_json",
        ],
    );
    for terminal in report.terminals() {
        let device = report
            .device(terminal.device)
            .expect("validated terminal owner");
        let mut prefix = vec![
            report.project_id().to_string(),
            device.id.to_string(),
            device.designation.clone(),
        ];
        prefix.extend(scopes.get(device.subsystem));
        prefix.extend([terminal.id.to_string(), terminal.number.clone()]);
        if terminal.connections.is_empty() {
            prefix.push("unconnected".into());
            prefix.extend(std::iter::repeat_n(String::new(), 10));
            row(&mut output, prefix);
        } else {
            for link in &terminal.connections {
                let wire = report
                    .conductor(link.conductor)
                    .expect("validated attached end");
                let destination = wire.destination(link.end);
                let mut cells = prefix.clone();
                cells.extend([
                    if destination.is_some() {
                        "connected"
                    } else {
                        "destination_free"
                    }
                    .into(),
                    wire.id.to_string(),
                    match link.end {
                        End::A => "A",
                        End::B => "B",
                    }
                    .into(),
                ]);
                cells.extend(cable_fields(report, wire.cable));
                cells.extend(terminal_fields(report, scopes, destination));
                row(&mut output, cells);
            }
        }
    }
    output
}
fn cables(report: &ConnectionReport, scopes: &Scopes) -> String {
    let mut output = String::new();
    row(
        &mut output,
        [
            "project_id",
            "cable_id",
            "cable_designation",
            "core_state",
            "conductor_id",
            "a_overview_kind",
            "a_overview_id",
            "a_overview_name",
            "a_overview_scope_ids",
            "a_overview_scope_names_json",
            "b_overview_kind",
            "b_overview_id",
            "b_overview_name",
            "b_overview_scope_ids",
            "b_overview_scope_names_json",
            "a_terminal_id",
            "a_device_id",
            "a_device_designation",
            "a_terminal_number",
            "a_scope_ids",
            "a_scope_names_json",
            "b_terminal_id",
            "b_device_id",
            "b_device_designation",
            "b_terminal_number",
            "b_scope_ids",
            "b_scope_names_json",
        ],
    );
    for cable in report.cables() {
        let core_ids: Vec<_> = if cable.conductors.is_empty() {
            vec![None]
        } else {
            cable.conductors.iter().copied().map(Some).collect()
        };
        for id in core_ids {
            let wire = id.map(|id| report.conductor(id).expect("validated cable core"));
            let mut cells = vec![
                report.project_id().to_string(),
                cable.id.to_string(),
                cable.designation.clone(),
                wire.map_or("empty_cable", |w| state(w.state())).into(),
                id.map_or_else(String::new, |id| id.to_string()),
            ];
            cells.extend(overview_fields(report, scopes, cable.endpoints[0]));
            cells.extend(overview_fields(report, scopes, cable.endpoints[1]));
            cells.extend(terminal_fields(
                report,
                scopes,
                wire.and_then(|w| w.endpoints[0]),
            ));
            cells.extend(terminal_fields(
                report,
                scopes,
                wire.and_then(|w| w.endpoints[1]),
            ));
            row(&mut output, cells);
        }
    }
    output
}
