//! Closed conversion from legacy frames to exact saved families, retaining inherited paper variants.
use crate::{
    ProjectFileError, TemplateFileError, TemplateLibraryFileError, capture_legacy_templates,
    encode_template_variant, legacy_template_capture::identity,
};
use ecad_app::{Command, Revision, Session};
use ecad_domain::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
/// A validated, revision-bound migration committed as one project transaction.
pub struct TemplateAssignmentMigration {
    revision: Revision,
    commands: Vec<Command>,
}
impl TemplateAssignmentMigration {
    /// Original project/session revision; stale migrations cannot commit.
    pub fn revision(&self) -> Revision {
        self.revision
    }
    /// Exact retained body imports and assignment edit; no re-querying on commit.
    pub fn commands(&self) -> Vec<Command> {
        self.commands.clone()
    }
}
fn bundle_error(error: impl Into<TemplateLibraryFileError>) -> ProjectFileError {
    ProjectFileError::TemplateBundle(Box::new(error.into()))
}
/// Build and validate a complete preservation migration without changing the session.
/// Legacy frames become complete saved families containing every required paper variant.
/// Explicit overrides remain explicit; inherited sheets retain stable variant IDs under
/// their owner's exact revision. Source geometry and all style intent remain unchanged.
/// # Errors
/// Invalid layouts, immutable-pin collisions or unresolved references reject the proposal.
pub fn preview_template_assignment_migration(
    session: &Session,
) -> Result<TemplateAssignmentMigration, ProjectFileError> {
    let project = session.project();
    let capture = capture_legacy_templates(session).map_err(bundle_error)?;
    let legacy = |t: SheetTemplate| !matches!(t, SheetTemplate::Saved(_));
    let mut needed = BTreeSet::new();
    for (id, pin) in capture.document_defaults() {
        if legacy(
            project
                .drawing_document(*id)
                .expect("captured owner")
                .default_template,
        ) {
            needed.insert(*pin);
        }
    }
    for (kind, pin) in capture.creation_defaults() {
        if legacy(project.template_defaults().for_kind(*kind)) {
            needed.insert(*pin);
        }
    }
    for item in capture.sheets() {
        let sheet = project.sheet(item.sheet).expect("captured sheet");
        let owner = project.sheet_document(item.sheet).expect("validated owner");
        if legacy(
            sheet.template.unwrap_or(
                project
                    .drawing_document(owner)
                    .expect("validated owner")
                    .default_template,
            ),
        ) {
            needed.insert(item.template);
        }
    }
    let mut groups: BTreeMap<
        TemplateDefinitionId,
        BTreeMap<TemplateVariantId, SavedTemplateVariant>,
    > = BTreeMap::new();
    for body in capture.bodies() {
        for variant in body.variants() {
            if needed.contains(&variant.reference()) {
                groups
                    .entry(variant.reference().definition)
                    .or_default()
                    .insert(variant.reference().variant, variant.clone());
            }
        }
    }
    let mut translated = BTreeMap::new();
    let mut commands = Vec::new();
    for (definition, variants) in groups {
        let mut canonical = Vec::new();
        for variant in variants.values() {
            let mut pin = variant.reference();
            pin.revision = TemplateRevisionId::new(1).expect("fixed nonzero ID");
            let placeholder = SavedTemplateVariant::preserve_legacy(
                pin,
                variant.name().to_owned(),
                variant.layout().clone(),
            )
            .map_err(TemplateFileError::Layout)
            .map_err(bundle_error)?;
            let text = encode_template_variant(&placeholder).map_err(bundle_error)?;
            canonical.extend_from_slice(&(text.len() as u64).to_be_bytes());
            canonical.extend_from_slice(text.as_bytes());
        }
        let revision = TemplateRevisionId::new(identity("assignment-family-v1", &canonical))
            .expect("nonzero deterministic ID");
        let mut bodies = Vec::new();
        for variant in variants.into_values() {
            let old = variant.reference();
            let pin = TemplateVariantRef {
                definition,
                revision,
                variant: old.variant,
            };
            translated.insert(old, pin);
            bodies.push(
                SavedTemplateVariant::preserve_legacy(
                    pin,
                    variant.name().to_owned(),
                    variant.layout().clone(),
                )
                .map_err(TemplateFileError::Layout)
                .map_err(bundle_error)?,
            );
        }
        let body = TemplateRevision::new(
            TemplateRevisionRef {
                definition,
                revision,
            },
            bodies,
        )
        .map_err(bundle_error)?;
        commands.push(Command::InsertTemplateRevision(Arc::new(body)));
    }
    let saved = |pin: TemplateVariantRef| {
        SheetTemplate::Saved(translated.get(&pin).copied().unwrap_or(pin))
    };
    let mut assignments = project.template_assignments();
    for (kind, pin) in capture.creation_defaults() {
        let choice = saved(*pin);
        match kind {
            SheetKind::Schematic => assignments.defaults.schematic = choice,
            SheetKind::Cover => assignments.defaults.cover = choice,
            SheetKind::DrawingList => assignments.defaults.drawing_list = choice,
        }
    }
    for (id, pin) in capture.document_defaults() {
        assignments
            .documents
            .iter_mut()
            .find(|(doc, _)| doc == id)
            .expect("captured owner")
            .1 = saved(*pin);
    }
    for item in capture.sheets() {
        let row = assignments
            .sheets
            .iter_mut()
            .find(|row| row.sheet == item.sheet)
            .expect("captured sheet");
        if row.template.is_some() {
            row.template = Some(saved(item.template));
            row.inherited_variant = None;
        } else {
            row.inherited_variant = Some(item.template.variant);
        }
    }
    commands.push(Command::TemplateAssignments(assignments));
    let mut candidate = Session::new(project.clone());
    candidate
        .execute_batch(candidate.revision(), commands.clone())
        .map_err(ProjectFileError::TemplateMigration)?;
    Ok(TemplateAssignmentMigration {
        revision: session.revision(),
        commands,
    })
}
/// Migrate a loaded project in memory without library IO or random identities. Used by
/// the desktop's explicit loading policy before establishing its clean checkpoint.
/// # Errors
/// Returns the complete validation error; the source project remains unchanged.
pub fn migrate_template_assignments(project: &Project) -> Result<Project, ProjectFileError> {
    let mut session = Session::new(project.clone());
    let proposal = preview_template_assignment_migration(&session)?;
    session
        .execute_batch(proposal.revision(), proposal.commands())
        .map_err(ProjectFileError::TemplateMigration)?;
    Ok(session.project().clone())
}
