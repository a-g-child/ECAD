//! Fixed document units and portable style requests, independent of screen/theme values.

use crate::DrawingError;

/// Positive document distance in integer micrometres, never pixels or physical wire area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawingLength(u32);
impl DrawingLength {
    /// Construct a positive distance; 2,500 micrometres is 2.5 mm.
    /// # Errors
    /// Zero is invalid for text height and visible stroke width.
    pub fn new(micrometres: u32) -> Result<Self, DrawingError> {
        if micrometres == 0 {
            return Err(DrawingError::ZeroLength);
        }
        Ok(Self(micrometres))
    }
    /// Return the exact document distance in micrometres.
    pub fn micrometres(self) -> u32 {
        self.0
    }
}
/// Signed document coordinates in integer micrometres, independent of zoom.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawingPoint {
    /// Horizontal coordinate in micrometres.
    pub x: i64,
    /// Vertical coordinate in micrometres.
    pub y: i64,
}
/// Output RGB colour; application theme/selection colours never enter this value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawingColour {
    /// Red channel, 0–255.
    pub red: u8,
    /// Green channel, 0–255.
    pub green: u8,
    /// Blue channel, 0–255.
    pub blue: u8,
}
/// A requested font family. Availability/substitution is a future renderer responsibility.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFamily(String);
impl FontFamily {
    /// Preserve a nonblank family request without consulting personal machine settings.
    /// # Errors
    /// Rejects blank requests.
    pub fn new(name: impl Into<String>) -> Result<Self, DrawingError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DrawingError::EmptyName);
        }
        Ok(Self(name))
    }
    /// Inspect the requested family without resolving installed font files.
    pub fn name(&self) -> &str {
        &self.0
    }
}
