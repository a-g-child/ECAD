//! Exact template-editor DTOs and property proposal conversion, separate from persistence.
use crate::dto::{BridgeError, Token};
use crate::template_text_styles::StylesRow;
use ecad_app::*;
use ecad_domain::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize)]
pub(crate) struct TemplateView {
    pub catalog_error: Option<String>,
    pub revision: Token,
    pub definition: String,
    pub published: Option<String>,
    pub selected: String,
    pub dirty: bool,
    pub pending: bool,
    pub variants: Vec<VariantRow>,
    pub catalog: Vec<(String, String)>,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct VariantRow {
    pub id: String,
    pub name: String,
    pub paper: [String; 6],
    pub content: [String; 4],
    pub tables: Vec<TableRow>,
    pub graphics: Vec<crate::template_graphic_rows::GraphicRow>,
    pub text_defaults: StylesRow,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct TableRow {
    pub id: String,
    pub bounds: [String; 4],
    pub cells: Vec<CellRow>,
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct CellRow {
    pub picture: Option<crate::picture_rows::PictureRow>,
    pub id: String,
    pub caption: String,
    pub bounds: [String; 4],
    pub contents: Vec<RunRow>,
    pub text_styles: StylesRow,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunRow {
    pub(crate) kind: String,
    pub(crate) value: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TemplateEditRow {
    variant: String,
    name: String,
    paper: [String; 6],
    content: [String; 4],
    cell: Option<CellEdit>,
    #[serde(default)]
    text_defaults: Option<StylesRow>,
    #[serde(default)]
    cell_styles: Vec<CellStyleEdit>,
    #[serde(default)]
    fit_margins: bool,
    #[serde(default)]
    graphics: Option<Vec<crate::template_graphic_rows::GraphicRow>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CellEdit {
    #[serde(default, deserialize_with = "picture_update")]
    picture: Option<Option<crate::picture_rows::PictureRow>>,
    table: String,
    id: String,
    caption: String,
    contents: Vec<RunRow>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CellStyleEdit {
    id: String,
    styles: StylesRow,
}
fn strings(r: FrameRect) -> [String; 4] {
    [r.x, r.y, r.width, r.height].map(|v| v.to_string())
}
impl TemplateView {
    pub(crate) fn new(
        doc: &TemplateDocument,
        selected: TemplateVariantId,
        pending: bool,
        catalog: Vec<(String, String)>,
    ) -> Self {
        Self {
            catalog_error: None,
            revision: doc.revision().into(),
            definition: doc.definition().to_string(),
            published: doc
                .published()
                .map(|p| format!("{} / {}", p.definition, p.revision)),
            selected: selected.to_string(),
            dirty: doc.is_dirty(),
            pending,
            catalog,
            variants: doc
                .variants()
                .map(|(id, v)| {
                    let p = v.layout().paper();
                    VariantRow {
                        id: id.to_string(),
                        name: v.name().into(),
                        text_defaults: v.layout().text_defaults().into(),
                        paper: [p.width, p.height, p.left, p.right, p.top, p.bottom]
                            .map(|v| v.to_string()),
                        content: strings(v.layout().content()),
                        tables: v.layout().tables().iter().map(TableRow::new).collect(),
                        graphics: v.layout().graphics().iter().map(Into::into).collect(),
                    }
                })
                .collect(),
        }
    }
}
impl TemplateEditRow {
    pub(crate) fn proposal(self, doc: &TemplateDocument) -> Result<ActionParameters, BridgeError> {
        let id = self
            .variant
            .parse::<TemplateVariantId>()
            .map_err(|e| BridgeError::new("template", e))?;
        let old = doc
            .variant(id)
            .ok_or_else(|| BridgeError::new("template", "Template variant is unavailable"))?;
        let parse = |s: &str| {
            s.parse::<u32>().map_err(|_| {
                BridgeError::new(
                    "template",
                    "Use nonnegative integer micrometres within the supported range",
                )
            })
        };
        let p = self
            .paper
            .iter()
            .map(|s| parse(s))
            .collect::<Result<Vec<_>, _>>()?;
        let c = self
            .content
            .iter()
            .map(|s| parse(s))
            .collect::<Result<Vec<_>, _>>()?;
        let mut tables = old.layout().tables().to_vec();
        if let Some(edit) = self.cell {
            let table = edit
                .table
                .parse::<TemplateTableId>()
                .map_err(|e| BridgeError::new("template", e))?;
            let cell = edit
                .id
                .parse::<TemplateCellId>()
                .map_err(|e| BridgeError::new("template", e))?;
            let table = tables
                .iter_mut()
                .find(|t| t.id() == table)
                .ok_or_else(|| BridgeError::new("template", "Selected table is unavailable"))?;
            let mut cells = table.cells().to_vec();
            let target = cells
                .iter_mut()
                .find(|c| c.id == cell)
                .ok_or_else(|| BridgeError::new("template", "Selected cell is unavailable"))?;
            if let Some(picture) = edit.picture {
                target.picture = picture.map(Picture::try_from).transpose()?;
            }
            target.caption = edit.caption;
            target.contents = edit
                .contents
                .into_iter()
                .map(|run| match run.kind.as_str() {
                    "literal" => Ok(TemplateTextRun::Literal(run.value)),
                    "field" => FrameField::from_key(&run.value)
                        .map(TemplateTextRun::Field)
                        .ok_or_else(|| BridgeError::new("template", "Unknown template variable")),
                    _ => Err(BridgeError::new("template", "Unknown text content type")),
                })
                .collect::<Result<_, _>>()?;
            *table = TemplateTable::new(table.id(), table.bounds(), cells)
                .map_err(|e| BridgeError::new("template", e))?;
        }
        let mut seen = std::collections::BTreeSet::new();
        for edit in self.cell_styles {
            let id = edit
                .id
                .parse::<TemplateCellId>()
                .map_err(|e| BridgeError::new("template", e))?;
            if !seen.insert(id) {
                return Err(BridgeError::new("template", "Repeated style target"));
            }
            let table = tables
                .iter_mut()
                .find(|t| t.cells().iter().any(|c| c.id == id))
                .ok_or_else(|| BridgeError::new("template", "Selected cell is unavailable"))?;
            let mut cells = table.cells().to_vec();
            cells
                .iter_mut()
                .find(|c| c.id == id)
                .expect("checked cell")
                .text_styles = edit.styles.try_into()?;
            *table = TemplateTable::new(table.id(), table.bounds(), cells)
                .map_err(|e| BridgeError::new("template", e))?;
        }
        let defaults = self
            .text_defaults
            .map(TemplateTextStyles::try_from)
            .transpose()?
            .unwrap_or_else(|| old.layout().text_defaults().clone());
        let build = if self.fit_margins {
            TemplateLayout::fit_inside_margins
        } else {
            TemplateLayout::new
        };
        let layout = build(
            SheetPaper {
                width: p[0],
                height: p[1],
                left: p[2],
                right: p[3],
                top: p[4],
                bottom: p[5],
            },
            FrameRect {
                x: c[0],
                y: c[1],
                width: c[2],
                height: c[3],
            },
            tables,
        )
        .map_err(|e| BridgeError::new("template", e))?;
        let graphics = match self.graphics {
            Some(rows) => rows
                .into_iter()
                .map(TemplateGraphic::try_from)
                .collect::<Result<_, _>>()?,
            None => old.layout().graphics().to_vec(),
        };
        let layout = layout
            .with_text_defaults(defaults)
            .and_then(|l| l.with_graphics(graphics))
            .map_err(|e| BridgeError::new("template", e))?;
        Ok(ActionParameters::TemplateEdit {
            variant: id,
            name: self.name,
            layout: Box::new(layout),
        })
    }
}

impl TableRow {
    pub(crate) fn new(t: &TemplateTable) -> Self {
        Self {
            id: t.id().to_string(),
            bounds: strings(t.bounds()),
            cells: t
                .cells()
                .iter()
                .map(|c| CellRow {
                    id: c.id.to_string(),
                    picture: c.picture.as_ref().map(Into::into),
                    caption: c.caption.clone(),
                    text_styles: (&c.text_styles).into(),
                    bounds: strings(c.bounds),
                    contents: c
                        .contents
                        .iter()
                        .map(|r| match r {
                            TemplateTextRun::Literal(s) => RunRow {
                                kind: "literal".into(),
                                value: s.clone(),
                            },
                            TemplateTextRun::Field(f) => RunRow {
                                kind: "field".into(),
                                value: f.key().into(),
                            },
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

// Missing leaves the picture intact; explicit null removes it.
fn picture_update<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<crate::picture_rows::PictureRow>>, D::Error> {
    Option::deserialize(deserializer).map(Some)
}
