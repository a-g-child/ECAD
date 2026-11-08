//! Explicit margin fitting for retained template geometry, never schematic/source content.
use crate::{FrameRect, SheetPaper, TemplateError, TemplateLayout, TemplateTable};
impl TemplateLayout {
    /// Fit a proposed template layout inside the paper margins. Tables receive the minimum
    /// translation needed for containment, preserving exact cell geometry/IDs/content. The
    /// explicit generated-content region is intersected with the new interior. Nothing scales.
    /// This is an opt-in complete edit, not a persistent constraint or standards profile.
    /// # Errors
    /// Rejects invalid paper, overflowing content extents or any table too large to fit.
    /// Returns a new validated layout without changing the input bodies.
    pub fn fit_inside_margins(
        paper: SheetPaper,
        content: FrameRect,
        tables: Vec<TemplateTable>,
    ) -> Result<Self, TemplateError> {
        paper.validate().map_err(TemplateError::Paper)?;
        let right = paper.width - paper.right;
        let bottom = paper.height - paper.bottom;
        let content_right = u64::from(content.x) + u64::from(content.width);
        let content_bottom = u64::from(content.y) + u64::from(content.height);
        if content_right > u64::from(u32::MAX) || content_bottom > u64::from(u32::MAX) {
            return Err(TemplateError::Bounds);
        }
        let x = content.x.clamp(paper.left, right);
        let y = content.y.clamp(paper.top, bottom);
        let end_x = u32::try_from(content_right)
            .expect("checked content extent")
            .clamp(paper.left, right);
        let end_y = u32::try_from(content_bottom)
            .expect("checked content extent")
            .clamp(paper.top, bottom);
        let content = FrameRect {
            x,
            y,
            width: end_x - x,
            height: end_y - y,
        };
        let mut fitted = vec![];
        for table in tables {
            let mut b = table.bounds();
            if b.width > right - paper.left || b.height > bottom - paper.top {
                return Err(TemplateError::Bounds);
            }
            b.x = b.x.clamp(paper.left, right - b.width);
            b.y = b.y.clamp(paper.top, bottom - b.height);
            // Translation keeps every local cell exactly; no scaling/rounding is introduced.
            fitted.push(TemplateTable::new(table.id(), b, table.cells().to_vec())?);
        }
        Self::new(paper, content, fitted)
    }
}
