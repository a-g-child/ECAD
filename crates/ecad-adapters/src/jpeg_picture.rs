//! Bounded JPEG validation, preserving original bytes and EXIF-oriented display dimensions.
use ecad_domain::{MAX_PICTURE_PIXELS, PictureAsset, PictureError, PictureFormat};
pub(crate) fn import(bytes: Vec<u8>) -> Result<PictureAsset, PictureError> {
    let mut decoder = jpeg_decoder::Decoder::new(std::io::Cursor::new(&bytes));
    decoder.set_max_decoding_buffer_size(32 * 1024 * 1024);
    decoder.read_info().map_err(|_| PictureError)?;
    let info = decoder.info().ok_or(PictureError)?;
    let (mut width, mut height) = (u32::from(info.width), u32::from(info.height));
    if u64::from(width) * u64::from(height) > MAX_PICTURE_PIXELS {
        return Err(PictureError);
    }
    decoder.decode().map_err(|_| PictureError)?;
    if decoder
        .exif_data()
        .and_then(orientation)
        .is_some_and(|v| (5..=8).contains(&v))
    {
        std::mem::swap(&mut width, &mut height);
    }
    PictureAsset::validated(bytes, width, height, PictureFormat::Jpeg)
}
// Only IFD0 Orientation is relevant. Invalid optional metadata never drives allocations or offsets.
fn orientation(data: &[u8]) -> Option<u16> {
    let little = match data.get(..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let word = |offset: usize| -> Option<u16> {
        let bytes = data.get(offset..offset.checked_add(2)?)?.try_into().ok()?;
        Some(if little {
            u16::from_le_bytes(bytes)
        } else {
            u16::from_be_bytes(bytes)
        })
    };
    let long = |offset: usize| -> Option<u32> {
        let bytes = data.get(offset..offset.checked_add(4)?)?.try_into().ok()?;
        Some(if little {
            u32::from_le_bytes(bytes)
        } else {
            u32::from_be_bytes(bytes)
        })
    };
    if word(2)? != 42 {
        return None;
    }
    let ifd = usize::try_from(long(4)?).ok()?;
    let count = usize::from(word(ifd)?);
    let start = ifd.checked_add(2)?;
    data.get(start..start.checked_add(count.checked_mul(12)?)?)?;
    for index in 0..count {
        let entry = start + index * 12;
        if word(entry)? == 0x112 && word(entry + 2)? == 3 && long(entry + 4)? == 1 {
            return word(entry + 8).filter(|v| (1..=8).contains(v));
        }
    }
    None
}
