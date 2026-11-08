//! Generated drawing navigation: typed targets, natural drawing-number order and explicit overflow.
use crate::{DrawingDocumentId, FrameRect, Project, SheetError, SheetId, SheetKind};
use std::cmp::Ordering;
/// One actual project sheet; generated rows never become physical electrical/drawing objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingListEntry {
    /// Stable navigation/PDF destination, independent of number or title.
    pub sheet: SheetId,
    /// Actual owning document.
    pub document: DrawingDocumentId,
    /// Literal unique drawing number.
    pub number: String,
    /// Literal multiline drawing title.
    pub title: String,
}
/// One physical drawing-list sheet's portion of the current project index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingListPage {
    /// Existing drawing-list sheet identity.
    pub sheet: SheetId,
    /// Available paper region in micrometres.
    pub bounds: FrameRect,
    /// Physical row height; uses u64 to handle unusually large inherited text heights safely.
    pub row_height: u64,
    /// Maximum rows after reserving one header row.
    pub capacity: usize,
    /// One-based position among list sheets in document/sheet order.
    pub ordinal: usize,
    /// Number of physical list sheets currently present.
    pub page_count: usize,
    /// Total non-list sheets in the project.
    pub total_entries: usize,
    /// Rows on this page only, naturally sorted by drawing number.
    pub rows: Vec<DrawingListEntry>,
    /// Entries beyond the capacity of all existing list sheets. Never silently truncated.
    pub remaining: usize,
}
impl Project {
    /// Resolve a generated list without changing sheets or storing copied titles/numbers.
    /// List sheets consume rows in document/sheet order; capacities may differ by paper/template.
    /// All non-list sheets, including covers, appear once; list sheets exclude themselves.
    /// # Errors
    /// Rejects missing sheet/owner identities. A non-list sheet returns None.
    pub fn drawing_list(&self, id: SheetId) -> Result<Option<DrawingListPage>, SheetError> {
        let sheet = self.sheet(id).ok_or(SheetError::Missing(id))?;
        if sheet.kind != SheetKind::DrawingList {
            return Ok(None);
        }
        let mut entries: Vec<_> = self
            .sheets()
            .iter()
            .filter(|s| s.kind != SheetKind::DrawingList)
            .map(|s| DrawingListEntry {
                sheet: s.id,
                document: self.sheet_document(s.id).expect("validated owner"),
                number: s.number.clone(),
                title: s.title.clone(),
            })
            .collect();
        entries.sort_by(|a, b| {
            natural_number(&a.number, &b.number)
                .then_with(|| a.title.cmp(&b.title))
                .then_with(|| a.sheet.cmp(&b.sheet))
        });
        let row_height =
            10_000u64.max(u64::from(self.drawing().settings().base.text_height.micrometres()) * 3);
        let mut consumed = 0usize;
        let mut selected = None;
        let mut pages = 0;
        for list in self
            .sheets()
            .iter()
            .filter(|s| s.kind == SheetKind::DrawingList)
        {
            pages += 1;
            let frame = self.sheet_frame(list.id)?;
            let capacity = if frame.content.width < 20_000 || frame.title_block_too_small {
                0
            } else {
                (u64::from(frame.content.height) / row_height).saturating_sub(1) as usize
            };
            if list.id == id {
                selected = Some((frame.content, capacity, consumed, pages));
            }
            consumed = consumed.saturating_add(capacity);
        }
        let (bounds, capacity, start, ordinal) = selected.expect("existing list sheet");
        Ok(Some(DrawingListPage {
            sheet: id,
            bounds,
            row_height,
            capacity,
            ordinal,
            page_count: pages,
            total_entries: entries.len(),
            rows: entries.iter().skip(start).take(capacity).cloned().collect(),
            remaining: entries.len().saturating_sub(consumed),
        }))
    }
}
/// ASCII digit runs compare by numeric magnitude without integer parsing or overflow.
fn natural_number(a: &str, b: &str) -> Ordering {
    let (mut a_rest, mut b_rest) = (a.as_bytes(), b.as_bytes());
    while !a_rest.is_empty() && !b_rest.is_empty() {
        if a_rest[0].is_ascii_digit() && b_rest[0].is_ascii_digit() {
            let an = a_rest.iter().take_while(|c| c.is_ascii_digit()).count();
            let bn = b_rest.iter().take_while(|c| c.is_ascii_digit()).count();
            let ar = trim_zero(&a_rest[..an]);
            let br = trim_zero(&b_rest[..bn]);
            let cmp = ar.len().cmp(&br.len()).then_with(|| ar.cmp(br));
            if cmp != Ordering::Equal {
                return cmp;
            }
            a_rest = &a_rest[an..];
            b_rest = &b_rest[bn..];
        } else {
            let cmp = a_rest[0].cmp(&b_rest[0]);
            if cmp != Ordering::Equal {
                return cmp;
            }
            a_rest = &a_rest[1..];
            b_rest = &b_rest[1..];
        }
    }
    a_rest.len().cmp(&b_rest.len()).then_with(|| a.cmp(b))
}

fn trim_zero(bytes: &[u8]) -> &[u8] {
    &bytes[bytes.iter().take_while(|c| **c == b'0').count()..]
}
