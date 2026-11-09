//! Versioned picture placements and a deduplicated portable asset table for project drawings.
use crate::{
    ProjectFileError,
    picture_data::{PictureData, asset_table, decode_assets},
};
use ecad_domain::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PicturesData {
    #[serde(deserialize_with = "crate::picture_data::unique_assets")]
    assets: BTreeMap<String, String>,
    placements: Vec<PlacementData>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlacementData {
    id: String,
    view: String,
    position_um: [i64; 2],
    size_um: [u32; 2],
    picture: PictureData,
}
impl From<&DrawingDocument> for PicturesData {
    fn from(d: &DrawingDocument) -> Self {
        Self {
            assets: asset_table(d.pictures().map(|p| &p.picture)),
            placements: d
                .pictures()
                .map(|p| PlacementData {
                    id: p.id.to_string(),
                    view: p.view.to_string(),
                    position_um: [p.position.x, p.position.y],
                    size_um: [p.width, p.height],
                    picture: (&p.picture).into(),
                })
                .collect(),
        }
    }
}
impl PicturesData {
    pub(crate) fn install(
        self,
        project: &mut Project,
        extended: bool,
    ) -> Result<(), ProjectFileError> {
        let assets = decode_assets(self.assets).map_err(data_error)?;
        if !extended && assets.values().any(|a| a.format() != PictureFormat::Png) {
            return Err(data_error("PNG-only schema contains another image format"));
        }
        for p in self.placements {
            project.insert_drawing_picture(DrawingPicture {
                id: p.id.parse()?,
                view: p.view.parse()?,
                position: DrawingPoint {
                    x: p.position_um[0],
                    y: p.position_um[1],
                },
                width: p.size_um[0],
                height: p.size_um[1],
                picture: p.picture.resolve(&assets).map_err(data_error)?,
            })?;
        }
        let used = asset_table(project.drawing().pictures().map(|p| &p.picture));
        if used.keys().ne(assets.keys()) {
            return Err(data_error("Unreferenced picture assets"));
        }
        Ok(())
    }
}

fn data_error(error: impl std::fmt::Display) -> ProjectFileError {
    ProjectFileError::Json(<serde_json::Error as serde::de::Error>::custom(error))
}
