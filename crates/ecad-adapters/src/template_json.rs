//! Standalone versioned template-variant bodies; no project-schema or filesystem side effects.
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
struct VariantData {
    schema_version: u32,
    definition: String,
    revision: String,
    variant: String,
    name: String,
    paper_um: [u32; 2],
    margins_um: [u32; 4],
    content_um: [u32; 4],
    tables: Vec<TableData>,
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
    id: String,
    bounds_um: [u32; 4],
    caption: String,
    contents: Vec<RunData>,
}
#[derive(Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
enum RunData {
    Literal(String),
    Field(String),
}
fn rect(value: [u32; 4]) -> FrameRect {
    FrameRect {
        x: value[0],
        y: value[1],
        width: value[2],
        height: value[3],
    }
}
fn coords(value: FrameRect) -> [u32; 4] {
    [value.x, value.y, value.width, value.height]
}

/// Encode one immutable variant with exact integer units, typed field keys and final LF.
/// Retains table/cell/run presentation order. No timestamps, sample field values, project
/// IDs, draft/history state, or machine-library paths enter the body. Does not publish it.
/// # Errors
/// Returns JSON encoding failure; domain constructors have already validated geometry.
pub fn encode_template_variant(value: &SavedTemplateVariant) -> Result<String, TemplateFileError> {
    let paper = value.layout().paper();
    let pin = value.reference();
    let data = VariantData {
        schema_version: 1,
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
                        id: cell.id.to_string(),
                        bounds_um: coords(cell.bounds),
                        caption: cell.caption.clone(),
                        contents: cell
                            .contents
                            .iter()
                            .map(|run| match run {
                                TemplateTextRun::Literal(text) => RunData::Literal(text.clone()),
                                TemplateTextRun::Field(field) => {
                                    RunData::Field(field.key().to_owned())
                                }
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    };
    Ok(serde_json::to_string_pretty(&data)? + "\n")
}
/// Decode an entire standalone body before exposing it. Unknown fields/field keys are
/// errors, preserving data rather than dropping unfamiliar content on a subsequent save.
/// # Errors
/// Rejects unsupported schemas, malformed identities, invalid partitions and layout bounds.
/// Reading a body has no publication, project migration or filesystem side effects.
pub fn decode_template_variant(text: &str) -> Result<SavedTemplateVariant, TemplateFileError> {
    let version: crate::schema::VersionEnvelope = serde_json::from_str(text)?;
    if version.schema_version != 1 {
        return Err(TemplateFileError::Version(version.schema_version));
    }
    let data: VariantData = serde_json::from_str(text)?;
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
    Ok(SavedTemplateVariant::new(
        TemplateVariantRef {
            definition: data.definition.parse()?,
            revision: data.revision.parse()?,
            variant: data.variant.parse()?,
        },
        data.name,
        TemplateLayout::new(paper, rect(data.content_um), tables)?,
    )?)
}
