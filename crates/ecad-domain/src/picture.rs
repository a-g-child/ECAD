//! Portable immutable image content and reusable placement appearance, independent of editors.
use std::{error::Error, fmt, sync::Arc};
/// Maximum compressed bytes in one imported picture (4 MiB).
pub const MAX_PICTURE_BYTES: usize = 4 * 1024 * 1024;
/// Maximum decoded pixels in one picture, bounding ordinary RGBA decoding to 16 MiB.
pub const MAX_PICTURE_PIXELS: u64 = 4 * 1024 * 1024;
/// Immutable image asset. Clone shares bytes; placements never contain machine-local paths.
/// Import/persistence adapters fully decode PNG before construction; the domain checks
/// the format envelope, dimensions and budget without depending on a codec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PictureAsset {
    format: PictureFormat,
    bytes: Arc<[u8]>,
    width: u32,
    height: u32,
}
impl PictureAsset {
    /// Retain a decoded/validated PNG's original bytes and dimensions.
    /// # Errors
    /// Rejects a mismatched PNG header, empty/oversized bytes or excessive pixel dimensions.
    /// This structural check does not replace full codec validation at import/load boundaries.
    pub fn png(bytes: Vec<u8>, width: u32, height: u32) -> Result<Self, PictureError> {
        if bytes.len() < 33
            || bytes.len() > MAX_PICTURE_BYTES
            || width == 0
            || height == 0
            || u64::from(width) * u64::from(height) > MAX_PICTURE_PIXELS
            || &bytes[..8] != b"\x89PNG\r\n\x1a\n"
            || &bytes[12..16] != b"IHDR"
            || bytes[16..20] != width.to_be_bytes()
            || bytes[20..24] != height.to_be_bytes()
        {
            return Err(PictureError);
        }
        Ok(Self {
            format: PictureFormat::Png,
            bytes: bytes.into(),
            width,
            height,
        })
    }
    /// Retain validated JPEG or self-contained SVG bytes from an import adapter.
    /// # Errors
    /// Rejects format envelopes and dimensions outside the shared image budget.
    /// Adapters must fully validate/decode the content before calling this constructor.
    pub fn validated(
        bytes: Vec<u8>,
        width: u32,
        height: u32,
        format: PictureFormat,
    ) -> Result<Self, PictureError> {
        if format == PictureFormat::Png {
            return Self::png(bytes, width, height);
        }
        if bytes.is_empty()
            || bytes.len() > MAX_PICTURE_BYTES
            || width == 0
            || height == 0
            || u64::from(width) * u64::from(height) > MAX_PICTURE_PIXELS
            || (format == PictureFormat::Jpeg && !bytes.starts_with(&[0xff, 0xd8]))
            || (format == PictureFormat::Svg && std::str::from_utf8(&bytes).is_err())
        {
            return Err(PictureError);
        }
        Ok(Self {
            format,
            bytes: bytes.into(),
            width,
            height,
        })
    }
    /// Validated media format; filenames and supplied MIME types are not authoritative.
    pub fn format(&self) -> PictureFormat {
        self.format
    }
    /// Validated original image bytes, independent of placements.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Intrinsic raster width in pixels (not a paper measurement).
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Intrinsic raster height in pixels (not a paper measurement).
    pub fn height(&self) -> u32 {
        self.height
    }
}
/// Mapping of pixels into a placement box; fitting never changes stored image bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PictureFit {
    /// Preserve aspect ratio and show the entire picture, centred in the box.
    #[default]
    Contain,
    /// Preserve aspect ratio and crop centrally to fill the box.
    Cover,
    /// Stretch to the exact box dimensions.
    Stretch,
}
/// Shared picture content used by free artwork and cell-bound placements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picture {
    /// Immutable portable image content; repeated placements may share this asset.
    pub asset: PictureAsset,
    /// Non-destructive mapping into the owner's bounds.
    pub fit: PictureFit,
}
/// Unsupported, malformed or over-budget picture data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PictureError;
impl fmt::Display for PictureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "Use PNG, JPEG or supported static SVG up to 4 MiB and 4 megapixels. SVG scripts, animation, embedded images and external resources are unsupported",
        )
    }
}
impl Error for PictureError {}

impl PictureFit {
    /// Stable interchange spelling, independent of translated UI labels.
    pub fn key(self) -> &'static str {
        match self {
            Self::Contain => "contain",
            Self::Cover => "cover",
            Self::Stretch => "stretch",
        }
    }
    /// Recognize a supported fit without silently substituting a default.
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "contain" => Some(Self::Contain),
            "cover" => Some(Self::Cover),
            "stretch" => Some(Self::Stretch),
            _ => None,
        }
    }
}

/// Supported portable image encodings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PictureFormat {
    /// Portable Network Graphics, including transparency.
    Png,
    /// JPEG raster.
    Jpeg,
    /// Self-contained static SVG, retained as vector data.
    Svg,
}
impl PictureFormat {
    /// MIME type for image-only rendering.
    pub fn mime(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Svg => "image/svg+xml",
        }
    }
}
