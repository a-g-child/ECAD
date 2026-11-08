//! Independent draft history/publication uses real files and never rewrites a consuming project.
use ecad_adapters::*;
use ecad_app::*;
use ecad_domain::*;
use std::sync::Arc;
fn body(revision: u128) -> TemplateRevision {
    let pin = TemplateVariantRef {
        definition: TemplateDefinitionId::new(1).unwrap(),
        revision: TemplateRevisionId::new(revision).unwrap(),
        variant: TemplateVariantId::new(1).unwrap(),
    };
    TemplateRevision::new(
        pin.revision_ref(),
        vec![
            SavedTemplateVariant::new(
                pin,
                "Original\nTemplate".into(),
                SheetTemplate::StandardV1
                    .layout(&SheetPaper::default())
                    .unwrap(),
            )
            .unwrap(),
        ],
    )
    .unwrap()
}
#[test]
fn draft_publication_preserves_complete_revision_history_and_exact_saved_undo_checkpoint() {
    let dir = tempfile::tempdir().unwrap();
    let store = TemplateLibraryFile::new(dir.path().join("templates.json"));
    let mut publisher = TemplatePublisher::new(store.clone());
    let original = body(1);
    publisher
        .publish(
            Arc::new(original.clone()),
            Recommendation::RecommendPublished,
        )
        .unwrap();
    let mut editor = TemplateDocument::open(&original);
    let id = TemplateVariantId::new(1).unwrap();
    let layout = editor.variant(id).unwrap().layout().clone();
    assert!(!editor.is_dirty());
    let token = editor.revision();
    editor
        .replace_variant(token, id, "Edited\nName".into(), layout.clone())
        .unwrap();
    assert!(
        editor
            .replace_variant(token, id, "Stale".into(), layout.clone())
            .is_err()
    );
    let mut project = Project::new(ProjectId::new(1).unwrap());
    project
        .insert_template_revision(Arc::new(original.clone()))
        .unwrap();
    let before = encode_project(&project).unwrap();
    let saved = editor
        .save(
            editor.revision(),
            TemplateRevisionId::new(2).unwrap(),
            &mut publisher,
        )
        .unwrap();
    assert!(!editor.is_dirty());
    assert_eq!(publisher.catalog().revisions().len(), 2);
    assert_eq!(
        publisher.catalog().recommended(saved.definition),
        Some(original.reference())
    );
    assert_eq!(encode_project(&project).unwrap(), before);
    editor.undo(editor.revision()).unwrap();
    assert_eq!(editor.variant(id).unwrap().name(), "Original\nTemplate");
    assert!(editor.is_dirty());
    editor.redo(editor.revision()).unwrap();
    assert!(!editor.is_dirty());
    assert_eq!(editor.published(), Some(saved));
    let reopened = TemplatePublisher::open(store).unwrap();
    assert_eq!(
        reopened
            .catalog()
            .revision(saved)
            .unwrap()
            .variant(id)
            .unwrap()
            .name(),
        "Edited\nName"
    );
}
#[test]
fn failed_saves_reloads_noops_and_branching_keep_draft_history_and_publication_state() {
    let dir = tempfile::tempdir().unwrap();
    let store = TemplateLibraryFile::new(dir.path().join("templates.json"));
    let original = body(1);
    let mut first = TemplatePublisher::new(store.clone());
    first
        .publish(Arc::new(original.clone()), Recommendation::KeepCurrent)
        .unwrap();
    let mut stale = TemplatePublisher::open(store).unwrap();
    let mut editor = TemplateDocument::open(&original);
    let id = TemplateVariantId::new(1).unwrap();
    let layout = editor.variant(id).unwrap().layout().clone();
    editor
        .replace_variant(editor.revision(), id, "Draft".into(), layout.clone())
        .unwrap();
    first
        .publish(Arc::new(body(3)), Recommendation::KeepCurrent)
        .unwrap();
    let token = editor.revision();
    let history = editor.undo_len();
    assert!(
        editor
            .save(token, TemplateRevisionId::new(2).unwrap(), &mut stale)
            .is_err()
    );
    assert_eq!(editor.revision(), token);
    assert_eq!(editor.undo_len(), history);
    assert_eq!(editor.published(), Some(original.reference()));
    assert!(editor.is_dirty());
    editor.undo(editor.revision()).unwrap();
    let redo = editor.redo_len();
    let token = editor.revision();
    assert!(
        !editor
            .replace_variant(token, id, "Original\nTemplate".into(), layout.clone())
            .unwrap()
    );
    assert_eq!(editor.redo_len(), redo);
    assert_eq!(editor.revision(), token);
    editor
        .replace_variant(editor.revision(), id, "Branch".into(), layout)
        .unwrap();
    assert_eq!(editor.redo_len(), 0);
    stale.reload().unwrap();
    editor
        .save(
            editor.revision(),
            TemplateRevisionId::new(4).unwrap(),
            &mut stale,
        )
        .unwrap();
    assert!(!editor.is_dirty());
}
#[test]
fn new_template_and_router_dispatch_work_without_an_electrical_document_and_protect_foreign_tokens()
{
    let dir = tempfile::tempdir().unwrap();
    let mut publisher =
        TemplatePublisher::new(TemplateLibraryFile::new(dir.path().join("templates.json")));
    let original = body(1);
    let mut editor = TemplateDocument::open(&original);
    let other = TemplateDocument::open(&original);
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    let context = ActionContext {
        editor: Some(editor.revision()),
        focus: ActionFocus::Canvas,
        ..ActionContext::default()
    };
    let mut call = |input, action: &str, context: ActionContext, parameters| {
        dispatcher.invoke_template(
            &registry,
            ActionInvocation {
                input_id: InputId::new(input).unwrap(),
                action: ActionId::new(action).unwrap(),
                source: InvocationSource::Toolbar,
                repeat: false,
                context,
                parameters,
            },
            &mut editor,
            &mut publisher,
            Some(TemplateRevisionId::new(9).unwrap()),
        )
    };
    assert!(call(1, "document.undo", context.clone(), ActionParameters::None).is_err());
    assert!(call(1, "document.save", context.clone(), ActionParameters::None).is_err());
    let foreign = ActionContext {
        editor: Some(other.revision()),
        ..context.clone()
    };
    assert!(call(2, "document.save", foreign, ActionParameters::None).is_err());
    assert!(
        call(
            3,
            "drawing.label.create",
            context.clone(),
            ActionParameters::None
        )
        .is_err()
    );
    assert!(call(4, "document.save", context, ActionParameters::None).is_ok());
    let new = TemplateDocument::new(
        TemplateDefinitionId::new(10).unwrap(),
        TemplateVariantId::new(10).unwrap(),
        "New".into(),
        SheetTemplate::StandardV1
            .layout(&SheetPaper::default())
            .unwrap(),
    )
    .unwrap();
    assert!(new.is_dirty());
    assert_eq!(new.published(), None);
}
