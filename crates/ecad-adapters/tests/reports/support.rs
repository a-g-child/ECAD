//! Independent CSV reader and known controller/conveyor input for export contracts.
use super::*;
pub fn project() -> Project {
    decode_project(include_str!("../fixtures/controller-conveyor-v6.json")).unwrap()
}
pub fn id(value: u128) -> String {
    format!("{value:032x}")
}
pub fn kind_list() -> [ConnectionCsvKind; 3] {
    [
        ConnectionCsvKind::WireList,
        ConnectionCsvKind::TerminalTable,
        ConnectionCsvKind::CableSchedule,
    ]
}
pub fn report_csv(session: &Session) -> Vec<String> {
    let report = ConnectionReport::from_session(session);
    kind_list()
        .into_iter()
        .map(|kind| encode_current_connection_csv(session, &report, kind).unwrap())
        .collect()
}
/// Parse quoted/unquoted cells independently of the production writer, including embedded newlines.
pub fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut chars = text.chars().peekable();
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                chars.next();
                cell.push('"');
            }
            '"' => quoted = !quoted,
            ',' if !quoted => row.push(std::mem::take(&mut cell)),
            '\n' if !quoted => {
                row.push(std::mem::take(&mut cell));
                rows.push(std::mem::take(&mut row));
            }
            _ => cell.push(c),
        }
    }
    assert!(!quoted);
    assert!(row.is_empty() && cell.is_empty());
    assert!(
        rows.iter().all(|r| r.len() == rows[0].len()),
        "ragged CSV output"
    );
    rows
}
pub fn column(rows: &[Vec<String>], name: &str) -> usize {
    rows[0].iter().position(|v| v == name).unwrap()
}
pub fn set(session: &mut Session, target: ObjectRef, property: PropertyId, value: &str) {
    session
        .execute(
            session.revision(),
            Command::SetProperty {
                target,
                property,
                value: PropertyValue::Text(value.into()),
            },
        )
        .unwrap();
}
