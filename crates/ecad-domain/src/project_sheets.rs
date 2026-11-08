//! Project-owned ordered sheets; view references never imply ownership of electrical objects.
use crate::*;
impl Project {
    /// Sheets flattened in document order and then owner-local sheet order.
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
    /// Compatibility insertion at a flattened ordinal. At a document boundary, use the
    /// preceding sheet's owner, or the first document at index zero. Prefer explicit-owner
    /// insertion for new workflows. Validation failure leaves the project unchanged.
    /// # Errors
    /// Rejects duplicate identity/number, invalid paper/scale/view and out-of-range order.
    pub fn insert_sheet(&mut self, value: DrawingSheet, index: usize) -> Result<(), SheetError> {
        if index > self.sheets.len() {
            return Err(SheetError::Order);
        }
        let document = if index == 0 {
            self.drawing_documents[0].id
        } else {
            self.sheet_document(self.sheets[index - 1].id)
                .expect("owned sheet")
        };
        let start = self.document_sheet_range(document)?.start;
        self.insert_sheet_in_document(value, document, index - start)
    }
    /// Insert at an explicit owner-local position. New UI/commands should use this rather than
    /// the compatibility flattened-index insertion method.
    /// # Errors
    /// Invalid/duplicate sheet data, unknown document or an out-of-range index is rejected.
    pub fn insert_sheet_in_document(
        &mut self,
        value: DrawingSheet,
        document: DrawingDocumentId,
        index: usize,
    ) -> Result<(), SheetError> {
        if self.sheet(value.id).is_some() {
            return Err(SheetError::Duplicate(value.id));
        }
        let range = self.document_sheet_range(document)?;
        if index > range.len() {
            return Err(SheetError::Order);
        }
        self.check_sheet(&value)?;
        self.sheet_template_layout(
            &value,
            self.drawing_document(document)
                .expect("checked owner")
                .default_template,
        )?;
        self.sheet_owners.insert(value.id, document);
        self.sheets.insert(range.start + index, value);
        Ok(())
    }
    /// Replace one sheet without changing its order or any view geometry.
    /// # Errors
    /// Missing/invalid sheets leave prior content unchanged.
    pub fn replace_sheet(&mut self, value: DrawingSheet) -> Result<DrawingSheet, SheetError> {
        self.check_sheet(&value)?;
        let owner = self
            .sheet_document(value.id)
            .ok_or(SheetError::Missing(value.id))?;
        self.sheet_template_layout(
            &value,
            self.drawing_document(owner)
                .expect("validated owner")
                .default_template,
        )?;
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
        self.sheet_owners.remove(&id);
        Ok((self.sheets.remove(index), index))
    }
    /// Compatibility move within the same owner, using a flattened final ordinal.
    /// Moving across a document boundary requires explicit move_sheet_to_document.
    /// # Errors
    /// Rejects unknown sheets or an out-of-range final position.
    pub fn move_sheet(&mut self, id: SheetId, index: usize) -> Result<usize, SheetError> {
        let owner = self.sheet_document(id).ok_or(SheetError::Missing(id))?;
        let range = self.document_sheet_range(owner)?;
        if !range.contains(&index) {
            return Err(SheetError::Order);
        }
        let old = self
            .sheets
            .iter()
            .position(|s| s.id == id)
            .expect("owned sheet");
        self.move_sheet_to_document(id, owner, index - range.start)?;
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
            let owner = self
                .sheet_document(sheet.id)
                .ok_or(SheetError::Missing(sheet.id))?;
            self.sheet_template_layout(
                sheet,
                self.drawing_document(owner)
                    .expect("validated owner")
                    .default_template,
            )?;
        }
        Ok(())
    }
}
