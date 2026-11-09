//! Standalone versioned template-variant bodies; no project-schema or filesystem side effects.
use crate::template_text_style_data::StylesData;
use ecad_domain::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Failure decoding/encoding an immutable template variant, distinct from project IO.
#[derive(Debug)]
pub enum TemplateFileError {
    /// Invalid JSON, unknown fields or malformed DTO shapes.
    Json(serde_json::Error),
    /// Unsupported standalone template schema.
    Version(u32),
    /// Malformed stable identity.
    Identity(IdError),
    /// Invalid paper, cell partition or layout.
    Layout(TemplateError),
    /// Unsupported field key; never silently converted to literal text.
    Field(String),
}
impl fmt::Display for TemplateFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(f, "Invalid template JSON: {error}"),
            Self::Version(version) => write!(f, "Unsupported template schema {version}"),
            Self::Identity(error) => error.fmt(f),
            Self::Layout(error) => error.fmt(f),
            Self::Field(key) => write!(f, "Unknown template field {key}"),
        }
    }
}
impl Error for TemplateFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::Identity(error) => Some(error),
            Self::Layout(error) => Some(error),
            _ => None,
        }
    }
}
impl From<serde_json::Error> for TemplateFileError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
impl From<IdError> for TemplateFileError {
    fn from(error: IdError) -> Self {
        Self::Identity(error)
    }
}
impl From<TemplateError> for TemplateFileError {
    fn from(error: TemplateError) -> Self {
        Self::Layout(error)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VariantData {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::picture_data::optional_assets"
    )]
    assets: Option<std::collections::BTreeMap<String, String>>,
    schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    new_assignments: Option<bool>,
    definition: String,
    revision: String,
    variant: String,
    name: String,
    paper_um: [u32; 2],
    margins_um: [u32; 4],
    content_um: [u32; 4],
    tables: Vec<TableData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    text_defaults: Option<StylesData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    graphics: Option<Vec<crate::template_graphic_data::GraphicData>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TableData {
    id: String,
    bounds_um: [u32; 4],
    cells: Vec<CellData>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CellData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    picture: Option<crate::picture_data::PictureData>,
    id: String,
    bounds_um: [u32; 4],
    caption: String,
    contents: Vec<RunData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    text_styles: Option<StylesData>,
}
#[derive(Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub(crate) enum RunData {
    Literal(String),
    Field(String),
}
pub(crate) fn rect(value: [u32; 4]) -> FrameRect {
    FrameRect {
        x: value[0],
        y: value[1],
        width: value[2],
        height: value[3],
    }
}
pub(crate) fn coords(value: FrameRect) -> [u32; 4] {
    [value.x, value.y, value.width, value.height]
}

/// Encode one immutable variant with exact integer units, typed field keys and final LF.
/// Retains table/cell/run presentation order. No timestamps, sample field values, project
/// IDs, draft/history state, or machine-library paths enter the body. Does not publish it.
/// # Errors
/// Returns JSON encoding failure; domain constructors have already validated geometry.
pub fn encode_template_variant(value: &SavedTemplateVariant) -> Result<String, TemplateFileError> {
    Ok(serde_json::to_string_pretty(&VariantData::from(value))? + "\n")
}
impl From<&SavedTemplateVariant> for VariantData {
    fn from(value: &SavedTemplateVariant) -> Self {
        let paper = value.layout().paper();
        let pin = value.reference();
        let styled = value.layout().text_defaults() != &TemplateTextStyles::default()
            || value.layout().tables().iter().any(|t| {
                t.cells()
                    .iter()
                    .any(|c| c.text_styles != TemplateTextStyles::default())
            });
        let assets = crate::picture_data::asset_table(value.layout().pictures());
        Self {
            assets: (!assets.is_empty()).then_some(assets.clone()),
            text_defaults: styled.then(|| value.layout().text_defaults().into()),
            schema_version: if value.layout().text_defaults().layout
                != TemplateCellTextLayout::default()
                || value
                    .layout()
                    .tables()
                    .iter()
                    .flat_map(|t| t.cells())
                    .any(|c| c.text_styles.layout != TemplateCellTextLayout::default())
            {
                7
            } else if value
                .layout()
                .pictures()
                .any(|p| p.asset.format() != PictureFormat::Png)
            {
                6
            } else if !assets.is_empty() {
                5
            } else if styled {
                4
            } else if !value.layout().graphics().is_empty() {
                3
            } else if value.allows_new_assignments() {
                1
            } else {
                2
            },
            graphics: (!value.layout().graphics().is_empty())
                .then(|| value.layout().graphics().iter().map(Into::into).collect()),
            new_assignments: (!value.allows_new_assignments()).then_some(false),
            definition: pin.definition.to_string(),
            revision: pin.revision.to_string(),
            variant: pin.variant.to_string(),
            name: value.name().to_owned(),
            paper_um: [paper.width, paper.height],
            margins_um: [paper.left, paper.right, paper.top, paper.bottom],
            content_um: coords(value.layout().content()),
            tables: value
                .layout()
                .tables()
                .iter()
                .map(|table| TableData {
                    id: table.id().to_string(),
                    bounds_um: coords(table.bounds()),
                    cells: table
                        .cells()
                        .iter()
                        .map(|cell| CellData {
                            picture: cell.picture.as_ref().map(Into::into),
                            id: cell.id.to_string(),
                            bounds_um: coords(cell.bounds),
                            caption: cell.caption.clone(),
                            text_styles: (cell.text_styles != TemplateTextStyles::default())
                                .then(|| (&cell.text_styles).into()),
                            contents: cell
                                .contents
                                .iter()
                                .map(|run| match run {
                                    TemplateTextRun::Literal(text) => {
                                        RunData::Literal(text.clone())
                                    }
                                    TemplateTextRun::Field(field) => {
                                        RunData::Field(field.key().to_owned())
                                    }
                                })
                                .collect(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}
/// Decode an entire standalone body before exposing it. Unknown fields/field keys are
/// errors, preserving data rather than dropping unfamiliar content on a subsequent save.
/// # Errors
/// Rejects unsupported schemas, malformed identities, invalid partitions and layout bounds.
/// Reading a body has no publication, project migration or filesystem side effects.
pub fn decode_template_variant(text: &str) -> Result<SavedTemplateVariant, TemplateFileError> {
    let version: crate::schema::VersionEnvelope = serde_json::from_str(text)?;
    if ![1, 2, 3, 4, 5, 6, 7].contains(&version.schema_version) {
        return Err(TemplateFileError::Version(version.schema_version));
    }
    let data: VariantData = serde_json::from_str(text)?;
    data.try_into()
}
impl TryFrom<VariantData> for SavedTemplateVariant {
    type Error = TemplateFileError;
    fn try_from(data: VariantData) -> Result<Self, Self::Error> {
        if ![1, 2, 3, 4, 5, 6, 7].contains(&data.schema_version) {
            return Err(TemplateFileError::Version(data.schema_version));
        }
        if (data.schema_version == 1 && data.new_assignments.is_some())
            || (data.schema_version == 2 && data.new_assignments != Some(false))
        {
            return Err(TemplateFileError::Json(
                <serde_json::Error as serde::de::Error>::custom(
                    "Variant v2 requires explicit retained-legacy eligibility",
                ),
            ));
        }
        if (data.schema_version < 3 && data.graphics.is_some())
            || (data.schema_version == 3 && data.graphics.is_none())
        {
            return Err(TemplateFileError::Version(data.schema_version));
        }
        let has_styles = data.text_defaults.is_some()
            || data
                .tables
                .iter()
                .any(|t| t.cells.iter().any(|c| c.text_styles.is_some()));
        if (data.schema_version < 4 && has_styles)
            || (data.schema_version == 4 && data.text_defaults.is_none())
        {
            return Err(TemplateFileError::Version(data.schema_version));
        }
        if (data.schema_version < 5
            && (data.assets.is_some()
                || data
                    .tables
                    .iter()
                    .any(|t| t.cells.iter().any(|c| c.picture.is_some()))))
            || ([5, 6].contains(&data.schema_version) && data.assets.is_none())
        {
            return Err(TemplateFileError::Version(data.schema_version));
        }
        if data.schema_version < 7
            && (data.text_defaults.as_ref().is_some_and(|s| s.has_layout())
                || data
                    .tables
                    .iter()
                    .flat_map(|t| &t.cells)
                    .any(|c| c.text_styles.as_ref().is_some_and(|s| s.has_layout())))
        {
            return Err(TemplateFileError::Version(data.schema_version));
        }
        let assets = crate::picture_data::decode_assets(data.assets.unwrap_or_default())?;
        if data.schema_version < 6 && assets.values().any(|a| a.format() != PictureFormat::Png) {
            return Err(TemplateFileError::Version(data.schema_version));
        }
        let retained = data.new_assignments == Some(false);
        let paper = SheetPaper {
            width: data.paper_um[0],
            height: data.paper_um[1],
            left: data.margins_um[0],
            right: data.margins_um[1],
            top: data.margins_um[2],
            bottom: data.margins_um[3],
        };
        let tables = data
            .tables
            .into_iter()
            .map(|table| {
                let cells = table
                    .cells
                    .into_iter()
                    .map(|cell| {
                        let contents = cell
                            .contents
                            .into_iter()
                            .map(|run| match run {
                                RunData::Literal(text) => Ok(TemplateTextRun::Literal(text)),
                                RunData::Field(key) => FrameField::from_key(&key)
                                    .map(TemplateTextRun::Field)
                                    .ok_or(TemplateFileError::Field(key)),
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok::<_, TemplateFileError>(TemplateCell {
                            picture: cell.picture.map(|p| p.resolve(&assets)).transpose()?,
                            text_styles: cell.text_styles.map(Into::into).unwrap_or_default(),
                            id: cell.id.parse()?,
                            bounds: rect(cell.bounds_um),
                            caption: cell.caption,
                            contents,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok::<_, TemplateFileError>(TemplateTable::new(
                    table.id.parse()?,
                    rect(table.bounds_um),
                    cells,
                )?)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let layout = TemplateLayout::new(paper, rect(data.content_um), tables)?.with_graphics(
            data.graphics
                .unwrap_or_default()
                .into_iter()
                .map(|g| g.resolve(&assets))
                .collect::<Result<_, _>>()?,
        )?;
        if crate::picture_data::asset_table(layout.pictures())
            .keys()
            .ne(assets.keys())
        {
            return Err(TemplateError::Graphic.into());
        }
        let layout =
            layout.with_text_defaults(data.text_defaults.map(Into::into).unwrap_or_default())?;
        if retained && !layout.tables().is_empty() {
            return Err(TemplateFileError::Json(
                <serde_json::Error as serde::de::Error>::custom(
                    "Retained legacy eligibility requires an omitted title block",
                ),
            ));
        }
        let build = if retained {
            SavedTemplateVariant::preserve_legacy
        } else {
            SavedTemplateVariant::new
        };
        Ok(build(
            TemplateVariantRef {
                definition: data.definition.parse()?,
                revision: data.revision.parse()?,
                variant: data.variant.parse()?,
            },
            data.name,
            layout,
        )?)
    }
}

impl From<&TemplateTextRun> for RunData {
    fn from(run: &TemplateTextRun) -> Self {
        match run {
            TemplateTextRun::Literal(s) => Self::Literal(s.clone()),
            TemplateTextRun::Field(f) => Self::Field(f.key().into()),
        }
    }
}
impl TryFrom<RunData> for TemplateTextRun {
    type Error = TemplateFileError;
    fn try_from(run: RunData) -> Result<Self, Self::Error> {
        match run {
            RunData::Literal(s) => Ok(Self::Literal(s)),
            RunData::Field(k) => FrameField::from_key(&k)
                .map(Self::Field)
                .ok_or(TemplateFileError::Field(k)),
        }
    }
}
