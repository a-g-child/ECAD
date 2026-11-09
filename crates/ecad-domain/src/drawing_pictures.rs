//! View-local pictures are graphical placements with no electrical connectivity.
use crate::*;
/// One picture in document coordinates, separate from its reusable immutable asset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingPicture {
    /// Stable placement identity.
    pub id: PictureId,
    /// Existing drawing scope.
    pub view: ViewId,
    /// Upper-left position in integer micrometres.
    pub position: DrawingPoint,
    /// Positive bounding width in micrometres.
    pub width: u32,
    /// Positive bounding height in micrometres.
    pub height: u32,
    /// Portable content and non-destructive fit.
    pub picture: Picture,
}
impl DrawingPicture {
    pub(crate) fn validate(&self) -> Result<(), DrawingError> {
        if self.width == 0 || self.height == 0 {
            return Err(DrawingError::ZeroLength);
        }
        self.position
            .x
            .checked_add(i64::from(self.width))
            .ok_or(DrawingError::CoordinateOverflow)?;
        self.position
            .y
            .checked_add(i64::from(self.height))
            .ok_or(DrawingError::CoordinateOverflow)?;
        Ok(())
    }
}
impl DrawingDocument {
    /// Pictures in stable identity order.
    pub fn pictures(&self) -> impl ExactSizeIterator<Item = &DrawingPicture> {
        self.pictures.values()
    }
    /// Resolve one existing picture placement.
    pub fn picture(&self, id: PictureId) -> Option<&DrawingPicture> {
        self.pictures.get(&id)
    }
    /// Insert an independent picture with validated view/geometry and bounded assets.
    /// # Errors
    /// Rejects duplicate identity, missing view, invalid geometry or excessive picture assets.
    pub fn insert_picture(&mut self, picture: DrawingPicture) -> Result<(), DrawingError> {
        if self.pictures.contains_key(&picture.id) {
            return Err(DrawingError::DuplicatePicture(picture.id));
        }
        self.check_picture(&picture)?;
        self.pictures.insert(picture.id, picture);
        Ok(())
    }
    fn check_picture(&self, picture: &DrawingPicture) -> Result<(), DrawingError> {
        picture.validate()?;
        if self.view(picture.view).is_none() {
            return Err(DrawingError::MissingView(picture.view));
        }
        let assets: std::collections::BTreeSet<_> = self
            .pictures
            .values()
            .filter(|p| p.id != picture.id)
            .map(|p| p.picture.asset.bytes())
            .chain(std::iter::once(picture.picture.asset.bytes()))
            .collect();
        if assets.len() > 64 || assets.iter().map(|b| b.len()).sum::<usize>() > 16 * 1024 * 1024 {
            return Err(DrawingError::PictureBudget);
        }
        Ok(())
    }
    /// Replace one picture, retaining the exact prior state for undo.
    /// # Errors
    /// Rejects missing identity or invalid geometry/view/assets without mutation.
    pub fn replace_picture(
        &mut self,
        picture: DrawingPicture,
    ) -> Result<DrawingPicture, DrawingError> {
        if !self.pictures.contains_key(&picture.id) {
            return Err(DrawingError::MissingPicture(picture.id));
        }
        self.check_picture(&picture)?;
        Ok(self
            .pictures
            .insert(picture.id, picture)
            .expect("checked picture"))
    }
    /// Remove one placement; other uses retain their shared image bytes.
    /// # Errors
    /// Rejects a missing identity without mutation.
    pub fn remove_picture(&mut self, id: PictureId) -> Result<DrawingPicture, DrawingError> {
        self.pictures
            .remove(&id)
            .ok_or(DrawingError::MissingPicture(id))
    }
}
