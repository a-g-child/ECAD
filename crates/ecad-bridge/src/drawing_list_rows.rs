//! Generated paper-table DTOs. Destinations are stable strings; dimensions are display millimetres.
use ecad_domain::DrawingListPage;
use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
pub(crate) struct DrawingListRow {
    sheet: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    row_height: f64,
    capacity: String,
    ordinal: String,
    page_count: String,
    total_entries: String,
    remaining: String,
    rows: Vec<Entry>,
}
#[derive(Clone, Debug, Serialize)]
struct Entry {
    sheet: String,
    document: String,
    number: String,
    title: String,
}
impl From<&DrawingListPage> for DrawingListRow {
    fn from(p: &DrawingListPage) -> Self {
        Self {
            sheet: p.sheet.to_string(),
            x: f64::from(p.bounds.x) / 1000.0,
            y: f64::from(p.bounds.y) / 1000.0,
            width: f64::from(p.bounds.width) / 1000.0,
            height: f64::from(p.bounds.height) / 1000.0,
            row_height: p.row_height as f64 / 1000.0,
            capacity: p.capacity.to_string(),
            ordinal: p.ordinal.to_string(),
            page_count: p.page_count.to_string(),
            total_entries: p.total_entries.to_string(),
            remaining: p.remaining.to_string(),
            rows: p
                .rows
                .iter()
                .map(|r| Entry {
                    sheet: r.sheet.to_string(),
                    document: r.document.to_string(),
                    number: r.number.clone(),
                    title: r.title.clone(),
                })
                .collect(),
        }
    }
}
