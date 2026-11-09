//! Display-only paper-frame DTOs, separate from persistence and exact sheet editor values.
use ecad_domain::{FrameRect, SheetFrame, SheetId};
use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
pub(crate) struct FrameRow {
    sheet: String,
    template: String,
    border: RectRow,
    cells: Vec<CellRow>,
    graphics: Vec<crate::template_graphic_rows::GraphicRow>,
    title_block_too_small: bool,
    font: String,
    text_height: f64,
    stroke_width: f64,
    colour: String,
}
#[derive(Clone, Debug, Serialize)]
struct RectRow {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
impl From<FrameRect> for RectRow {
    fn from(r: FrameRect) -> Self {
        Self {
            x: mm(r.x),
            y: mm(r.y),
            width: mm(r.width),
            height: mm(r.height),
        }
    }
}
#[derive(Clone, Debug, Serialize)]
struct CellRow {
    key: String,
    caption: String,
    text_styles: crate::template_text_styles::StylesRow,
    value: Option<String>,
    bounds: RectRow,
}
fn mm(value: u32) -> f64 {
    f64::from(value) / 1000.0
}
impl FrameRow {
    pub(crate) fn new(id: SheetId, frame: &SheetFrame) -> Self {
        let a = &frame.appearance;
        Self {
            sheet: id.to_string(),
            template: frame.template.key(),
            graphics: frame.graphics.iter().map(Into::into).collect(),
            border: frame.border.into(),
            cells: frame
                .cells
                .iter()
                .map(|c| CellRow {
                    key: c
                        .field
                        .map(|f| f.key().to_owned())
                        .unwrap_or_else(|| c.id.to_string()),
                    caption: c.caption.clone(),
                    text_styles: (&c.text_styles).into(),
                    value: c.value.clone(),
                    bounds: c.bounds.into(),
                })
                .collect(),
            title_block_too_small: frame.title_block_too_small,
            font: a.font.name().into(),
            text_height: mm(a.text_height.micrometres()),
            stroke_width: mm(a.stroke_width.micrometres()),
            colour: format!(
                "#{:02x}{:02x}{:02x}",
                a.colour.red, a.colour.green, a.colour.blue
            ),
        }
    }
}
