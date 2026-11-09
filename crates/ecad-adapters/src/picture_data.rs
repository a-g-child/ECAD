//! Content-addressed picture references and embedded portable PNG asset tables.
use crate::{TemplateFileError, picture_base64, picture_from_base64, picture_key};
use ecad_domain::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PictureData {
    asset: String,
    fit: String,
}
impl From<&Picture> for PictureData {
    fn from(p: &Picture) -> Self {
        Self {
            asset: picture_key(&p.asset),
            fit: p.fit.key().into(),
        }
    }
}
impl PictureData {
    pub(crate) fn resolve(
        self,
        assets: &BTreeMap<String, PictureAsset>,
    ) -> Result<Picture, TemplateFileError> {
        Ok(Picture {
            asset: assets
                .get(&self.asset)
                .cloned()
                .ok_or(TemplateError::Graphic)?,
            fit: PictureFit::from_key(&self.fit).ok_or(TemplateError::Graphic)?,
        })
    }
}
pub(crate) fn asset_table<'a>(
    pictures: impl IntoIterator<Item = &'a Picture>,
) -> BTreeMap<String, String> {
    pictures
        .into_iter()
        .map(|p| (picture_key(&p.asset), picture_base64(&p.asset)))
        .collect()
}
pub(crate) fn decode_assets(
    data: BTreeMap<String, String>,
) -> Result<BTreeMap<String, PictureAsset>, TemplateFileError> {
    if data.len() > 64 || data.values().map(String::len).sum::<usize>() > 24 * 1024 * 1024 {
        return Err(TemplateError::Graphic.into());
    }
    data.into_iter()
        .map(|(key, text)| {
            let asset = picture_from_base64(&text).map_err(|_| TemplateError::Graphic)?;
            if picture_key(&asset) != key {
                return Err(TemplateError::Graphic.into());
            }
            Ok((key, asset))
        })
        .collect()
}

/// Reject duplicate content addresses rather than silently losing a stored body.
pub(crate) fn unique_assets<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, String>, D::Error> {
    struct Assets;
    impl<'de> serde::de::Visitor<'de> for Assets {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a table of unique picture assets")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut assets = BTreeMap::new();
            let mut bytes = 0;
            while let Some((key, value)) = map.next_entry::<String, String>()? {
                bytes += value.len();
                if assets.insert(key, value).is_some()
                    || assets.len() > 64
                    || bytes > 24 * 1024 * 1024
                {
                    return Err(serde::de::Error::custom(
                        "Duplicate or excessive picture assets",
                    ));
                }
            }
            Ok(assets)
        }
    }
    deserializer.deserialize_map(Assets)
}
pub(crate) fn optional_assets<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<BTreeMap<String, String>>, D::Error> {
    unique_assets(deserializer).map(Some)
}
