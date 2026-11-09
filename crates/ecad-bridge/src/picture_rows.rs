//! Shared picture transport; PNG bytes are validated at every write boundary, never URLs/paths.
use crate::dto::BridgeError;
use ecad_domain::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PictureRow {
    pub png: String,
    pub fit: String,
}
impl From<&Picture> for PictureRow {
    fn from(p: &Picture) -> Self {
        Self {
            png: ecad_adapters::picture_base64(&p.asset),
            fit: p.fit.key().into(),
        }
    }
}
impl TryFrom<PictureRow> for Picture {
    type Error = BridgeError;
    fn try_from(p: PictureRow) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: ecad_adapters::picture_from_base64(&p.png)
                .map_err(|e| BridgeError::new("picture", e))?,
            fit: PictureFit::from_key(&p.fit)
                .ok_or_else(|| BridgeError::new("picture", "Choose a supported picture fit"))?,
        })
    }
}
/// Exact editable placement values, distinct from renderer scene points.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PlacementRow {
    pub bounds: [String; 4],
    pub picture: PictureRow,
}
impl From<&DrawingPicture> for PlacementRow {
    fn from(p: &DrawingPicture) -> Self {
        Self {
            bounds: [
                p.position.x.to_string(),
                p.position.y.to_string(),
                p.width.to_string(),
                p.height.to_string(),
            ],
            picture: (&p.picture).into(),
        }
    }
}
impl PlacementRow {
    pub(crate) fn drawing(
        self,
        id: PictureId,
        view: ViewId,
    ) -> Result<DrawingPicture, BridgeError> {
        let error = || {
            BridgeError::new(
                "picture",
                "Use exact coordinates and positive picture dimensions",
            )
        };
        Ok(DrawingPicture {
            id,
            view,
            position: DrawingPoint {
                x: self.bounds[0].parse().map_err(|_| error())?,
                y: self.bounds[1].parse().map_err(|_| error())?,
            },
            width: self.bounds[2].parse().map_err(|_| error())?,
            height: self.bounds[3].parse().map_err(|_| error())?,
            picture: self.picture.try_into()?,
        })
    }
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ImportedPictureRow {
    pub picture: PictureRow,
    pub pixels: [String; 2],
}
