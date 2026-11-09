//! Retained paper artwork, independent of electrical drawings and resolved sheet values.
use crate::{FrameRect, TemplateError, TemplateGraphicId, TemplateTextRun};

/// Geometry in integer paper micrometres. Polylines remain a single editable object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateGraphicShape {
    /// Multiline text composed of literal and typed variable runs.
    Text(Vec<TemplateTextRun>),
    /// Unfilled rectangular outline.
    Rectangle,
    /// Unfilled ellipse inscribed in the object's bounds.
    Ellipse,
    /// Ordered local points; at least two distinct consecutive vertices are required.
    Polyline(Vec<[u32; 2]>),
}
/// One retained template object. Text uses an anchor and renderer-derived visible bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateGraphic {
    /// Stable identity independent of geometry or content.
    pub id: TemplateGraphicId,
    /// Paper-space origin and extent, in micrometres.
    /// For text, extents are retained legacy hints, not clipping or movement limits.
    pub bounds: FrameRect,
    /// Retained geometry/content, never resolved project values.
    pub shape: TemplateGraphicShape,
    /// Literal font family; used by text only.
    pub font: String,
    /// Positive text height in micrometres.
    pub text_height: u32,
    /// Positive outline width in micrometres.
    pub stroke_width: u32,
    /// RGB colour shared by text and outlines.
    pub colour: [u8; 3],
}
impl TemplateGraphic {
    pub(crate) fn validate(&self, paper: &crate::SheetPaper) -> Result<(), TemplateError> {
        let b = self.bounds;
        let text = matches!(self.shape, TemplateGraphicShape::Text(_));
        let outside = if text {
            b.x >= paper.width || b.y >= paper.height
        } else {
            u64::from(b.x) + u64::from(b.width) > u64::from(paper.width)
                || u64::from(b.y) + u64::from(b.height) > u64::from(paper.height)
        };
        if outside || self.text_height == 0 || self.stroke_width == 0 || self.font.trim().is_empty()
        {
            return Err(TemplateError::Graphic);
        }
        match &self.shape {
            TemplateGraphicShape::Polyline(points) => {
                if points.len() < 2
                    || points.len() > 4096
                    || points.windows(2).any(|p| p[0] == p[1])
                    || points.iter().any(|p| p[0] > b.width || p[1] > b.height)
                {
                    return Err(TemplateError::Graphic);
                }
            }
            _ if b.width == 0 || b.height == 0 => return Err(TemplateError::Graphic),
            _ => {}
        }
        Ok(())
    }
}
