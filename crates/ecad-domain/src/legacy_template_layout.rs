//! Preservation layout for the three v11 frame definitions, shared by sheets and previews.
use crate::*;
impl SheetTemplate {
    /// Materialize the exact legacy paper/table layout without a project or sample values.
    /// Tiny valid sheets retain their border/content and omit the title block, as in v11.
    /// Stable local table/cell identities do not depend on displayed metadata or sheet identity.
    /// This is a preservation profile, not a verified ISO/ASME drawing-border profile.
    /// # Errors
    /// Rejects invalid paper; all valid v11 layouts, including tiny interiors, are retained.
    pub fn layout(self, paper: &SheetPaper) -> Result<TemplateLayout, SheetError> {
        paper.validate()?;
        let p = paper;
        let border = FrameRect {
            x: p.left,
            y: p.top,
            width: p.width - p.left - p.right,
            height: p.height - p.top - p.bottom,
        };
        let template = self;
        let (columns, row_height) = match template {
            SheetTemplate::StandardV1 => (2, 10_000),
            SheetTemplate::CoverV1 => (2, 20_000),
            SheetTemplate::DrawingListV1 => (4, 10_000),
        };
        let block_height = (8 / columns) * row_height;
        let too_small = border.width < 180_000 || border.height < block_height;
        let block_x = if template == SheetTemplate::CoverV1 {
            border.x + border.width.saturating_sub(180_000) / 2
        } else {
            p.width - p.right - 180_000.min(border.width)
        };
        let block_y = if template == SheetTemplate::CoverV1 {
            border.y + border.height.saturating_sub(block_height) / 2
        } else {
            p.height - p.bottom - block_height.min(border.height)
        };
        let content = FrameRect {
            x: border.x + 5_000.min(border.width),
            y: border.y + 5_000.min(border.height),
            width: border.width.saturating_sub(10_000),
            height: if too_small {
                border.height.saturating_sub(10_000)
            } else {
                block_y.saturating_sub(border.y).saturating_sub(10_000)
            },
        };
        let fields = [
            FrameField::Metadata(MetadataField::CompanyName),
            FrameField::Metadata(MetadataField::ProjectTitle),
            FrameField::DocumentTitle,
            FrameField::SheetTitle,
            FrameField::DocumentNumber,
            FrameField::SheetNumber,
            FrameField::DocumentRevision,
            FrameField::DocumentSheetCount,
        ];
        let tables = if too_small {
            Vec::new()
        } else {
            let cells = fields
                .into_iter()
                .enumerate()
                .map(|(i, field)| TemplateCell {
                    id: TemplateCellId::new(i as u128 + 1).expect("nonzero fixed cell ordinal"),
                    bounds: FrameRect {
                        x: (i as u32 % columns) * (180_000 / columns),
                        y: (i as u32 / columns) * row_height,
                        width: 180_000 / columns,
                        height: row_height,
                    },
                    caption: field.caption().to_owned(),
                    contents: vec![TemplateTextRun::Field(field)],
                })
                .collect();
            vec![
                TemplateTable::new(
                    TemplateTableId::new(1).expect("nonzero fixed table identity"),
                    FrameRect {
                        x: block_x,
                        y: block_y,
                        width: 180_000,
                        height: block_height,
                    },
                    cells,
                )
                .expect("fixed legacy cells partition the information block"),
            ]
        };
        Ok(TemplateLayout::new(paper.clone(), content, tables)
            .expect("valid legacy paper produces contained geometry"))
    }
}
