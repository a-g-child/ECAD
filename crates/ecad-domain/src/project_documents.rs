//! Validated document metadata and canonical ordered sheet ownership; no drawing geometry changes.
use crate::*;
use std::{collections::BTreeSet, ops::Range};
impl Project {
    /// Logical drawing packages in explicit order (not application session controllers).
    pub fn drawing_documents(&self) -> &[ProjectDrawingDocument] {
        &self.drawing_documents
    }
    /// Resolve stable drawing-package identity.
    pub fn drawing_document(&self, id: DrawingDocumentId) -> Option<&ProjectDrawingDocument> {
        self.drawing_documents.iter().find(|d| d.id == id)
    }
    /// Owner of a project sheet, independent of drawing number or electrical hierarchy.
    pub fn sheet_document(&self, id: SheetId) -> Option<DrawingDocumentId> {
        self.sheet_owners.get(&id).copied()
    }
    pub(crate) fn document_sheet_range(
        &self,
        id: DrawingDocumentId,
    ) -> Result<Range<usize>, DrawingDocumentError> {
        let index = self
            .drawing_documents
            .iter()
            .position(|d| d.id == id)
            .ok_or(DrawingDocumentError::Missing(id))?;
        let count = |id| {
            self.sheet_owners
                .values()
                .filter(|owner| **owner == id)
                .count()
        };
        let start = self.drawing_documents[..index]
            .iter()
            .map(|d| count(d.id))
            .sum();
        Ok(start..start + count(id))
    }
    /// Borrow one document's sheets in its explicit order.
    /// # Errors
    /// Rejects an unknown drawing-document identity.
    pub fn sheets_in_document(
        &self,
        id: DrawingDocumentId,
    ) -> Result<&[DrawingSheet], DrawingDocumentError> {
        Ok(&self.sheets[self.document_sheet_range(id)?])
    }
    /// Inspect exact ownership and ordering without duplicating sheet records.
    pub fn drawing_document_layout(&self) -> Vec<DrawingDocumentLayout> {
        self.drawing_documents
            .iter()
            .map(|d| DrawingDocumentLayout {
                document: d.clone(),
                sheets: self
                    .sheets
                    .iter()
                    .filter(|s| self.sheet_document(s.id) == Some(d.id))
                    .map(|s| s.id)
                    .collect(),
            })
            .collect()
    }
    fn check_document_metadata(
        &self,
        value: &ProjectDrawingDocument,
    ) -> Result<(), DrawingDocumentError> {
        value.validate()?;
        self.check_template_default(value.default_template)?;
        for sheet in self
            .sheets
            .iter()
            .filter(|s| self.sheet_document(s.id) == Some(value.id))
        {
            self.sheet_template_layout(sheet, value.default_template)
                .map_err(|e| match e {
                    SheetError::Template(e) => *e,
                    _ => TemplateAssignmentError::PaperMismatch,
                })?;
        }
        if self
            .drawing_documents
            .iter()
            .any(|d| d.id != value.id && d.number == value.number)
        {
            return Err(DrawingDocumentError::NumberInUse(value.number.clone()));
        }
        Ok(())
    }
    /// Insert an empty drawing package without altering sheets or source objects.
    /// # Errors
    /// Rejects duplicate identity/number, invalid metadata and out-of-range order.
    pub fn insert_drawing_document(
        &mut self,
        value: ProjectDrawingDocument,
        index: usize,
    ) -> Result<(), DrawingDocumentError> {
        if self.drawing_document(value.id).is_some() {
            return Err(DrawingDocumentError::Duplicate(value.id));
        }
        if index > self.drawing_documents.len() {
            return Err(DrawingDocumentError::Order);
        }
        self.check_document_metadata(&value)?;
        self.drawing_documents.insert(index, value);
        Ok(())
    }
    /// Replace package metadata with exact previous values for undo.
    /// # Errors
    /// Unknown documents or invalid/duplicate numbers leave metadata unchanged.
    pub fn replace_drawing_document(
        &mut self,
        value: ProjectDrawingDocument,
    ) -> Result<ProjectDrawingDocument, DrawingDocumentError> {
        self.check_document_metadata(&value)?;
        let old = self
            .drawing_documents
            .iter_mut()
            .find(|d| d.id == value.id)
            .ok_or(DrawingDocumentError::Missing(value.id))?;
        Ok(std::mem::replace(old, value))
    }
    /// Remove only an empty document, returning its metadata/order for undo.
    /// # Errors
    /// Rejects unknown/nonempty packages and removal of the last package.
    pub fn remove_drawing_document(
        &mut self,
        id: DrawingDocumentId,
    ) -> Result<(ProjectDrawingDocument, usize), DrawingDocumentError> {
        let index = self
            .drawing_documents
            .iter()
            .position(|d| d.id == id)
            .ok_or(DrawingDocumentError::Missing(id))?;
        if self.sheet_owners.values().any(|owner| *owner == id) {
            return Err(DrawingDocumentError::InUse(id));
        }
        if self.drawing_documents.len() == 1 {
            return Err(DrawingDocumentError::LastDocument);
        }
        Ok((self.drawing_documents.remove(index), index))
    }
    fn reorder_document_sheets(&mut self) {
        let order: std::collections::BTreeMap<_, _> = self
            .drawing_documents
            .iter()
            .enumerate()
            .map(|(index, d)| (d.id, index))
            .collect();
        self.sheets
            .sort_by_key(|s| order[&self.sheet_owners[&s.id]]);
    }
    /// Move a package together with its sheets, without changing any number or identity.
    /// # Errors
    /// Unknown packages/out-of-range ordinals leave the project unchanged.
    pub fn move_drawing_document(
        &mut self,
        id: DrawingDocumentId,
        index: usize,
    ) -> Result<usize, DrawingDocumentError> {
        if index >= self.drawing_documents.len() {
            return Err(DrawingDocumentError::Order);
        }
        let old = self
            .drawing_documents
            .iter()
            .position(|d| d.id == id)
            .ok_or(DrawingDocumentError::Missing(id))?;
        let value = self.drawing_documents.remove(old);
        self.drawing_documents.insert(index, value);
        self.reorder_document_sheets();
        Ok(old)
    }
    /// Inspect a sheet's owner and zero-based position in that owner.
    /// # Errors
    /// Rejects an unavailable sheet.
    pub fn document_sheet_position(
        &self,
        id: SheetId,
    ) -> Result<(DrawingDocumentId, usize), DrawingDocumentError> {
        let owner = self
            .sheet_document(id)
            .ok_or(DrawingDocumentError::MissingSheet(id))?;
        let index = self
            .sheets_in_document(owner)?
            .iter()
            .position(|s| s.id == id)
            .ok_or(DrawingDocumentError::MissingSheet(id))?;
        Ok((owner, index))
    }
    /// Transfer/reorder one sheet at an explicit final owner-local index; retain all source data.
    /// # Errors
    /// Rejects unknown sheets/documents and invalid final ordinals before mutation.
    pub fn move_sheet_to_document(
        &mut self,
        id: SheetId,
        target: DrawingDocumentId,
        index: usize,
    ) -> Result<(DrawingDocumentId, usize), DrawingDocumentError> {
        let old = self.document_sheet_position(id)?;
        let range = self.document_sheet_range(target)?;
        let length = range.len();
        if index > length || (old.0 == target && index == length) {
            return Err(DrawingDocumentError::Order);
        }
        let target_default = self
            .drawing_document(target)
            .expect("checked target")
            .default_template;
        self.sheet_template_layout(self.sheet(id).expect("checked sheet"), target_default)
            .map_err(|e| match e {
                SheetError::Template(e) => *e,
                _ => TemplateAssignmentError::PaperMismatch,
            })?;
        let global = self
            .sheets
            .iter()
            .position(|s| s.id == id)
            .expect("known sheet");
        let sheet = self.sheets.remove(global);
        self.sheet_owners.remove(&id);
        let start = self.document_sheet_range(target)?.start;
        self.sheets.insert(start + index, sheet);
        self.sheet_owners.insert(id, target);
        Ok(old)
    }
    fn check_document_layout(
        &self,
        layout: &[DrawingDocumentLayout],
    ) -> Result<(), DrawingDocumentError> {
        if layout.is_empty() {
            return Err(DrawingDocumentError::LastDocument);
        }
        let mut ids = BTreeSet::new();
        let mut numbers = BTreeSet::new();
        let mut sheets = BTreeSet::new();
        for d in layout {
            d.document.validate()?;
            self.check_template_default(d.document.default_template)?;
            if !ids.insert(d.document.id) {
                return Err(DrawingDocumentError::Duplicate(d.document.id));
            }
            if !numbers.insert(&d.document.number) {
                return Err(DrawingDocumentError::NumberInUse(d.document.number.clone()));
            }
            for id in &d.sheets {
                if self.sheet(*id).is_none() {
                    return Err(DrawingDocumentError::MissingSheet(*id));
                }
                self.sheet_template_layout(
                    self.sheet(*id).expect("checked sheet"),
                    d.document.default_template,
                )
                .map_err(|e| match e {
                    SheetError::Template(e) => *e,
                    _ => TemplateAssignmentError::PaperMismatch,
                })?;
                if !sheets.insert(*id) {
                    return Err(DrawingDocumentError::Membership(*id));
                }
            }
        }
        for sheet in &self.sheets {
            if !sheets.contains(&sheet.id) {
                return Err(DrawingDocumentError::Membership(sheet.id));
            }
        }
        Ok(())
    }
    /// Atomically install a complete metadata/membership layout during import or reviewed editing.
    /// The result contains exact prior metadata/IDs/order, not duplicate geometry records.
    /// # Errors
    /// Rejects missing/duplicate sheets, invalid metadata or an empty document set.
    pub fn replace_drawing_document_layout(
        &mut self,
        layout: Vec<DrawingDocumentLayout>,
    ) -> Result<Vec<DrawingDocumentLayout>, DrawingDocumentError> {
        self.check_document_layout(&layout)?;
        let old = self.drawing_document_layout();
        let mut sheets = vec![];
        let mut owners = std::collections::BTreeMap::new();
        for d in &layout {
            for id in &d.sheets {
                sheets.push(self.sheet(*id).expect("checked sheet").clone());
                owners.insert(*id, d.document.id);
            }
        }
        self.drawing_documents = layout.into_iter().map(|d| d.document).collect();
        self.sheets = sheets;
        self.sheet_owners = owners;
        Ok(old)
    }
    pub(crate) fn validate_drawing_documents(&self) -> Result<(), DrawingDocumentError> {
        for id in self.sheet_owners.keys() {
            if self.sheet(*id).is_none() {
                return Err(DrawingDocumentError::MissingSheet(*id));
            }
        }
        let layout = self.drawing_document_layout();
        self.check_document_layout(&layout)?;
        if layout
            .iter()
            .flat_map(|d| d.sheets.iter())
            .copied()
            .ne(self.sheets.iter().map(|s| s.id))
        {
            return Err(DrawingDocumentError::Order);
        }
        Ok(())
    }
    pub(crate) fn has_custom_drawing_documents(&self) -> bool {
        self.drawing_documents != [ProjectDrawingDocument::initial(self.id())]
    }
}
