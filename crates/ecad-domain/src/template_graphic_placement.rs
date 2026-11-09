//! Checked two-point and multi-vertex artwork placement; source vertices and variable runs survive translation.
use crate::{
    FrameRect, SheetPaper, TemplateError, TemplateGraphic, TemplateGraphicId, TemplateGraphicShape,
    TemplateTextRun,
};
/// Initial pointer primitive. A line is retained as one two-vertex polyline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplatePrimitive {
    /// Multiline text box, initially containing the literal "Text".
    Text,
    /// Ordered line endpoints, including horizontal and vertical lines.
    Line,
    /// Ordered multi-point graphical path, retained as one polyline.
    Polyline,
    /// Rectangular outline.
    Rectangle,
    /// Ellipse bounded by the two corners.
    Ellipse,
}
impl TemplateGraphic {
    /// Build one ordered paper path, retaining closing and nonconsecutive repeated vertices.
    /// Bounds and local vertices are exact integer micrometres; appearance uses creation defaults.
    /// # Errors
    /// Rejects fewer than two or more than 4096 points, consecutive duplicates, negative,
    /// overflowing or out-of-paper coordinates. No electrical connection is created.
    pub fn from_vertices(
        id: TemplateGraphicId,
        points: &[[i64; 2]],
        paper: &SheetPaper,
    ) -> Result<Self, TemplateError> {
        if !(2..=4096).contains(&points.len()) {
            return Err(TemplateError::Graphic);
        }
        let points = points
            .iter()
            .map(|p| {
                Ok([
                    u32::try_from(p[0]).map_err(|_| TemplateError::Graphic)?,
                    u32::try_from(p[1]).map_err(|_| TemplateError::Graphic)?,
                ])
            })
            .collect::<Result<Vec<_>, TemplateError>>()?;
        let x = points.iter().map(|p| p[0]).min().expect("nonempty path");
        let y = points.iter().map(|p| p[1]).min().expect("nonempty path");
        let right = points.iter().map(|p| p[0]).max().expect("nonempty path");
        let bottom = points.iter().map(|p| p[1]).max().expect("nonempty path");
        let mut graphic = Self::between(
            id,
            TemplatePrimitive::Line,
            [i64::from(x), i64::from(y)],
            [i64::from(right), i64::from(bottom)],
            paper,
        )?;
        graphic.shape = TemplateGraphicShape::Polyline(
            points.into_iter().map(|p| [p[0] - x, p[1] - y]).collect(),
        );
        graphic.validate(paper)?;
        Ok(graphic)
    }
    /// Create artwork between two signed paper points, in micrometres.
    /// Reverse drags normalize bounds while retaining line endpoint order.
    /// # Errors
    /// Rejects negative, collapsed, overflowing or out-of-paper geometry.
    pub fn between(
        id: TemplateGraphicId,
        kind: TemplatePrimitive,
        start: [i64; 2],
        end: [i64; 2],
        paper: &SheetPaper,
    ) -> Result<Self, TemplateError> {
        let point = |p: [i64; 2]| -> Result<[u32; 2], TemplateError> {
            Ok([
                u32::try_from(p[0]).map_err(|_| TemplateError::Graphic)?,
                u32::try_from(p[1]).map_err(|_| TemplateError::Graphic)?,
            ])
        };
        let a = point(start)?;
        let b = point(end)?;
        let x = a[0].min(b[0]);
        let y = a[1].min(b[1]);
        let bounds = FrameRect {
            x,
            y,
            width: a[0].abs_diff(b[0]),
            height: a[1].abs_diff(b[1]),
        };
        let shape = match kind {
            TemplatePrimitive::Text => {
                TemplateGraphicShape::Text(vec![TemplateTextRun::Literal("Text".into())])
            }
            TemplatePrimitive::Line | TemplatePrimitive::Polyline => {
                TemplateGraphicShape::Polyline(vec![[a[0] - x, a[1] - y], [b[0] - x, b[1] - y]])
            }
            TemplatePrimitive::Rectangle => TemplateGraphicShape::Rectangle,
            TemplatePrimitive::Ellipse => TemplateGraphicShape::Ellipse,
        };
        let graphic = Self {
            id,
            bounds,
            shape,
            font: "sans-serif".into(),
            text_height: 3500,
            stroke_width: 250,
            colour: [40, 60, 70],
        };
        graphic.validate(paper)?;
        Ok(graphic)
    }
    /// Translate the whole object to an exact paper origin, retaining all local content/style.
    /// # Errors
    /// Rejects negative, overflowing or out-of-paper placement without modifying the original.
    pub fn placed_at(&self, origin: [i64; 2], paper: &SheetPaper) -> Result<Self, TemplateError> {
        let mut g = self.clone();
        g.bounds.x = u32::try_from(origin[0]).map_err(|_| TemplateError::Graphic)?;
        g.bounds.y = u32::try_from(origin[1]).map_err(|_| TemplateError::Graphic)?;
        g.validate(paper)?;
        Ok(g)
    }
}
