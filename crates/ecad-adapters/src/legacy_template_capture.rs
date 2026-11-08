//! Deterministic, read-only legacy layout capture before saved-template assignment migration.
use crate::{TemplateFileError, TemplateLibraryFileError, encode_template_variant};
use ecad_app::{Command, Revision, Session};
use ecad_domain::*;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::Arc};

/// A sheet's exact resolved legacy layout, retaining explicit override versus inheritance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacySheetTemplate {
    /// Stable existing sheet identity.
    pub sheet: SheetId,
    /// True when the current sheet inherits its owner's enum template.
    pub inherited: bool,
    /// Exact captured variant; source geometry/placement stays on the original sheet.
    pub template: TemplateVariantRef,
}
/// Closed preservation proposal at its original project-session revision.
/// Bodies contain paper/tables/bindings only. The report is not a new sheet assignment;
/// current enum/default/inheritance fields remain authoritative until that migration lands.
pub struct LegacyTemplateCapture {
    revision: Revision,
    bodies: BTreeMap<TemplateRevisionRef, Arc<TemplateRevision>>,
    sheets: Vec<LegacySheetTemplate>,
    documents: Vec<(DrawingDocumentId, TemplateVariantRef)>,
    defaults: Vec<(SheetKind, TemplateVariantRef)>,
}
impl LegacyTemplateCapture {
    /// Session/revision authorizing this exact closed proposal.
    pub fn revision(&self) -> Revision {
        self.revision
    }
    /// Immutable complete captured revisions, in canonical identity order.
    pub fn bodies(&self) -> impl ExactSizeIterator<Item = &TemplateRevision> {
        self.bodies.values().map(Arc::as_ref)
    }
    /// Sheet mappings in existing owner-local drawing order, without renumbering.
    pub fn sheets(&self) -> &[LegacySheetTemplate] {
        &self.sheets
    }
    /// Default-size variants for existing document enum defaults. Actual inherited sheets
    /// retain their own exact paper-specific captures in sheets(), not this default paper.
    pub fn document_defaults(&self) -> &[(DrawingDocumentId, TemplateVariantRef)] {
        &self.documents
    }
    /// Default-size variants capturing project creation-template choices by page role.
    pub fn creation_defaults(&self) -> &[(SheetKind, TemplateVariantRef)] {
        &self.defaults
    }
    /// Ordinary import commands for one execute_batch at revision(). Exact inverses retain
    /// earlier bundled state; repeated capture/import is a no-op. No shared library is written.
    pub fn commands(&self) -> Vec<Command> {
        self.bodies
            .values()
            .map(|body| Command::InsertTemplateRevision(Arc::clone(body)))
            .collect()
    }
}
/// Capture all current enum/paper combinations plus unused document/creation defaults.
/// Existing body pins are reused only for exactly equal complete bodies; collisions fail.
/// Generated identities derive from versioned canonical inputs, never paths, project IDs,
/// random IDs, clocks or resolved metadata. The same layout in two projects has the same pin.
/// This is a preservation stage, not automatic adoption or a standards-compliance conversion.
/// # Errors
/// Rejects invalid legacy geometry or immutable-pin conflicts. Performs no mutation or IO.
pub fn capture_legacy_templates(
    session: &Session,
) -> Result<LegacyTemplateCapture, TemplateLibraryFileError> {
    let project = session.project();
    let mut capture = LegacyTemplateCapture {
        revision: session.revision(),
        bodies: BTreeMap::new(),
        sheets: Vec::new(),
        documents: Vec::new(),
        defaults: Vec::new(),
    };
    for document in project.drawing_documents() {
        let pin = capture.add(project, document.default_template, &SheetPaper::default())?;
        capture.documents.push((document.id, pin));
        for sheet in project
            .sheets_in_document(document.id)
            .expect("validated document owner")
        {
            let pin = capture.add(
                project,
                project
                    .sheet_template_pin(sheet, document.default_template)
                    .map(SheetTemplate::Saved)
                    .unwrap_or(sheet.template.unwrap_or(document.default_template)),
                &sheet.paper,
            )?;
            capture.sheets.push(LegacySheetTemplate {
                sheet: sheet.id,
                inherited: sheet.template.is_none(),
                template: pin,
            });
        }
    }
    for kind in [
        SheetKind::Schematic,
        SheetKind::Cover,
        SheetKind::DrawingList,
    ] {
        let pin = capture.add(
            project,
            project.template_defaults().for_kind(kind),
            &SheetPaper::default(),
        )?;
        capture.defaults.push((kind, pin));
    }
    Ok(capture)
}
impl LegacyTemplateCapture {
    fn add(
        &mut self,
        project: &Project,
        template: SheetTemplate,
        paper: &SheetPaper,
    ) -> Result<TemplateVariantRef, TemplateLibraryFileError> {
        if let SheetTemplate::Saved(pin) = template {
            let revision = project
                .template_revision(pin.revision_ref())
                .ok_or(TemplateLibraryError::MissingRevision(pin.revision_ref()))?;
            self.bodies
                .insert(pin.revision_ref(), Arc::new(revision.clone()));
            return Ok(pin);
        }
        let layout = template
            .layout(paper)
            .map_err(|error| TemplateFileError::Layout(TemplateError::Paper(error)))?;
        let definition =
            TemplateDefinitionId::new(identity("definition", template.key().as_bytes()))
                .expect("nonzero deterministic identity");
        let dimensions: Vec<_> = [
            paper.width,
            paper.height,
            paper.left,
            paper.right,
            paper.top,
            paper.bottom,
        ]
        .into_iter()
        .flat_map(u32::to_be_bytes)
        .collect();
        let variant = TemplateVariantId::new(identity("variant", &dimensions))
            .expect("nonzero deterministic identity");
        let name = format!(
            "Legacy {} / {} × {} µm",
            template.label(),
            paper.width,
            paper.height
        );
        // Hash a canonical body with a fixed placeholder revision, then store the real pin.
        let placeholder = SavedTemplateVariant::preserve_legacy(
            TemplateVariantRef {
                definition,
                revision: TemplateRevisionId::new(1).expect("fixed nonzero ID"),
                variant,
            },
            name.clone(),
            layout.clone(),
        )
        .map_err(TemplateFileError::Layout)?;
        let body = encode_template_variant(&placeholder)?;
        let revision = TemplateRevisionId::new(identity("revision", body.as_bytes()))
            .expect("nonzero deterministic identity");
        let pin = TemplateVariantRef {
            definition,
            revision,
            variant,
        };
        let body = SavedTemplateVariant::preserve_legacy(pin, name, layout)
            .map_err(TemplateFileError::Layout)?;
        let revision = Arc::new(TemplateRevision::new(pin.revision_ref(), vec![body])?);
        let existing = project
            .template_revision(pin.revision_ref())
            .or_else(|| self.bodies.get(&pin.revision_ref()).map(Arc::as_ref));
        if existing.is_some_and(|previous| previous != revision.as_ref()) {
            return Err(TemplateLibraryError::RevisionConflict(pin.revision_ref()).into());
        }
        self.bodies.insert(pin.revision_ref(), revision);
        Ok(pin)
    }
}
pub(crate) fn identity(kind: &str, bytes: &[u8]) -> u128 {
    let mut hash = Sha256::new();
    hash.update(b"ecad-legacy-template-capture-v1\0");
    hash.update(kind.as_bytes());
    hash.update([0]);
    hash.update(bytes);
    let hash = hash.finalize();
    u128::from_be_bytes(hash[..16].try_into().expect("fixed SHA-256 prefix")).max(1)
}
