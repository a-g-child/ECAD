//! Drawing and style failures independent of storage or rendering.

use crate::{LabelId, PolylineId, StyleId, ViewId};
use std::{error::Error, fmt};

/// A drawing proposal violates units, identity, references, or role applicability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawingError {
    /// A requested coordinate or offset exceeds the signed document range.
    CoordinateOverflow,
    /// Text sizes and stroke widths must be positive.
    ZeroLength,
    /// Required view/style/font/annotation text is blank.
    EmptyName,
    /// Paths need two points and no adjacent duplicates.
    InvalidPath,
    /// A style/appearance carries fields for the other graphical role.
    WrongRole,
    /// A referenced view is unavailable.
    MissingView(ViewId),
    /// A view already exists.
    DuplicateView(ViewId),
    /// A view still owns drawing objects.
    ViewInUse(ViewId),
    /// A referenced annotation is unavailable.
    MissingLabel(LabelId),
    /// An annotation identity already exists.
    DuplicateLabel(LabelId),
    /// A referenced path is unavailable.
    MissingPolyline(PolylineId),
    /// A path identity already exists.
    DuplicatePolyline(PolylineId),
    /// A referenced style is unavailable.
    MissingStyle(StyleId),
    /// A style identity already exists.
    DuplicateStyle(StyleId),
    /// A style remains referenced by objects or creation presets.
    StyleInUse(StyleId),
    /// Reusable electrical bodies cannot silently discard drawing data/settings.
    ReusableDrawingsUnsupported,
}
impl fmt::Display for DrawingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CoordinateOverflow => {
                f.write_str("drawing coordinate exceeds the signed document range")
            }
            Self::ZeroLength => f.write_str("drawing length must be positive"),
            Self::EmptyName => {
                f.write_str("drawing names, font requests, and annotation text must not be blank")
            }
            Self::InvalidPath => {
                f.write_str("graphical paths need at least two points with no adjacent duplicates")
            }
            Self::WrongRole => {
                f.write_str("drawing style properties or links do not match the graphical role")
            }
            Self::MissingView(id) => write!(f, "drawing view {id} is unavailable"),
            Self::DuplicateView(id) => write!(f, "drawing view {id} already exists"),
            Self::ViewInUse(id) => write!(f, "drawing view {id} still owns objects"),
            Self::MissingLabel(id) => write!(f, "annotation {id} is unavailable"),
            Self::DuplicateLabel(id) => write!(f, "annotation {id} already exists"),
            Self::MissingPolyline(id) => write!(f, "graphical path {id} is unavailable"),
            Self::DuplicatePolyline(id) => write!(f, "graphical path {id} already exists"),
            Self::MissingStyle(id) => write!(f, "drawing style {id} is unavailable"),
            Self::DuplicateStyle(id) => write!(f, "drawing style {id} already exists"),
            Self::StyleInUse(id) => write!(
                f,
                "drawing style {id} is referenced by objects or creation presets"
            ),
            Self::ReusableDrawingsUnsupported => f.write_str(
                "reusable electrical bodies do not yet support drawing content/settings",
            ),
        }
    }
}
impl Error for DrawingError {}
