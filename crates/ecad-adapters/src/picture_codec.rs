//! Bounded image import and content addressing, shared by persistence and desktop adapters.
use base64::{Engine, engine::general_purpose::STANDARD};
use ecad_domain::{MAX_PICTURE_BYTES, MAX_PICTURE_PIXELS, PictureAsset, PictureError};
use sha2::{Digest, Sha256};
use std::io::Cursor;
/// Fully decode a bounded static PNG before exposing portable content to the core.
/// Original bytes (including transparency) are retained; no source pathname is stored.
/// # Errors
/// Rejects corrupt/truncated/animated PNG, excessive dimensions and decoder allocation limits.
pub fn import_png(bytes: Vec<u8>) -> Result<PictureAsset, PictureError> {
    if bytes.len() > MAX_PICTURE_BYTES {
        return Err(PictureError);
    }
    let mut decoder = png::Decoder::new(Cursor::new(&bytes));
    decoder.set_limits(png::Limits {
        bytes: 32 * 1024 * 1024,
    });
    let mut reader = decoder.read_info().map_err(|_| PictureError)?;
    let info = reader.info();
    let (width, height) = (info.width, info.height);
    if info.animation_control.is_some() || u64::from(width) * u64::from(height) > MAX_PICTURE_PIXELS
    {
        return Err(PictureError);
    }
    let size = reader.output_buffer_size().ok_or(PictureError)?;
    if size > 32 * 1024 * 1024 {
        return Err(PictureError);
    }
    let mut output = vec![0; size];
    reader.next_frame(&mut output).map_err(|_| PictureError)?;
    reader.finish().map_err(|_| PictureError)?;
    drop(reader);
    PictureAsset::png(bytes, width, height)
}
/// Stable content address independent of filename, placement or import order.
pub fn picture_key(asset: &PictureAsset) -> String {
    format!("{:x}", Sha256::digest(asset.bytes()))
}
/// Encode the immutable image for an embedded asset table or a renderer transport.
pub fn picture_base64(asset: &PictureAsset) -> String {
    STANDARD.encode(asset.bytes())
}
/// Decode and validate an embedded image with a bound checked before allocation.
/// # Errors
/// Rejects malformed base64 and every invalid image rejected by [`import_picture`].
pub fn picture_from_base64(text: &str) -> Result<PictureAsset, PictureError> {
    if text.len() > MAX_PICTURE_BYTES.div_ceil(3) * 4 {
        return Err(PictureError);
    }
    import_picture(STANDARD.decode(text).map_err(|_| PictureError)?)
}

/// Validate an image by its contents and retain its original portable bytes.
/// SVG stays vector data; no filesystem or network resources are resolved.
/// # Errors
/// Rejects unsupported/invalid content, external SVG resources and excessive budgets.
pub fn import_picture(bytes: Vec<u8>) -> Result<PictureAsset, PictureError> {
    if bytes.len() > MAX_PICTURE_BYTES {
        return Err(PictureError);
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return import_png(bytes);
    }
    if bytes.starts_with(&[0xff, 0xd8]) {
        return crate::jpeg_picture::import(bytes);
    }
    let (width, height) = crate::svg_picture::validate(&bytes)?;
    PictureAsset::validated(bytes, width, height, ecad_domain::PictureFormat::Svg)
}
