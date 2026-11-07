//! Project-owned ordered sheets; view references never imply ownership of electrical objects.
use crate::*;
impl Project {
    /// Sheets in the initial drawing set's explicit order.
    pub fn sheets(&self) -> &[DrawingSheet] {
        &self.sheets
    }
    /// Find a sheet by stable identity, independent of number and position.
    pub fn sheet(&self, id: SheetId) -> Option<&DrawingSheet> {
        self.sheets.iter().find(|s| s.id == id)
    }
    fn check_sheet(&self, value: &DrawingSheet) -> Result<(), SheetError> {
        value.validate()?;
        if self
            .sheets
            .iter()
            .any(|s| s.id != value.id && s.number == value.number)
        {
            return Err(SheetError::NumberInUse(value.number.clone()));
        }
        if let Some(p) = &value.placement
            && self.drawing().view(p.view).is_none()
        {
            return Err(SheetError::MissingView(p.view));
        }
        Ok(())
    }
    /// Insert a sheet at an explicit ordinal; validation failure leaves the set unchanged.
    /// # Errors
    /// Rejects duplicate identity/number, invalid paper/scale/view and out-of-range order.
    pub fn insert_sheet(&mut self, value: DrawingSheet, index: usize) -> Result<(), SheetError> {
        if self.sheet(value.id).is_some() {
            return Err(SheetError::Duplicate(value.id));
        }
        if index > self.sheets.len() {
            return Err(SheetError::Order);
        }
        self.check_sheet(&value)?;
        self.sheets.insert(index, value);
        Ok(())
    }
    /// Replace one sheet without changing its order or any view geometry.
    /// # Errors
    /// Missing/invalid sheets leave prior content unchanged.
    pub fn replace_sheet(&mut self, value: DrawingSheet) -> Result<DrawingSheet, SheetError> {
        self.check_sheet(&value)?;
        let sheet = self
            .sheets
            .iter_mut()
            .find(|s| s.id == value.id)
            .ok_or(SheetError::Missing(value.id))?;
        Ok(std::mem::replace(sheet, value))
    }
    /// Remove a sheet, returning exact content and order for undo. Views remain intact.
    /// # Errors
    /// Rejects an unknown sheet identity.
    pub fn remove_sheet(&mut self, id: SheetId) -> Result<(DrawingSheet, usize), SheetError> {
        let index = self
            .sheets
            .iter()
            .position(|s| s.id == id)
            .ok_or(SheetError::Missing(id))?;
        Ok((self.sheets.remove(index), index))
    }
    /// Move a sheet within the initial drawing set without renumbering it.
    /// # Errors
    /// Rejects unknown sheets or an out-of-range final position.
    pub fn move_sheet(&mut self, id: SheetId, index: usize) -> Result<usize, SheetError> {
        if index >= self.sheets.len() {
            return Err(SheetError::Order);
        }
        let old = self
            .sheets
            .iter()
            .position(|s| s.id == id)
            .ok_or(SheetError::Missing(id))?;
        let value = self.sheets.remove(old);
        self.sheets.insert(index, value);
        Ok(old)
    }
    pub(crate) fn check_sheet_views(&self, drawing: &DrawingDocument) -> Result<(), DrawingError> {
        for sheet in &self.sheets {
            if let Some(p) = &sheet.placement
                && drawing.view(p.view).is_none()
            {
                return Err(DrawingError::ViewInUse(p.view));
            }
        }
        Ok(())
    }
    pub(crate) fn validate_sheets(&self) -> Result<(), SheetError> {
        for sheet in &self.sheets {
            self.check_sheet(sheet)?;
        }
        Ok(())
    }
}
