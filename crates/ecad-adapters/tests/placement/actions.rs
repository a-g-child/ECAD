//! M preview/commit follows captured action context and one monotonic input stream.
use super::*;
fn invocation(
    document: &Document<JsonFile>,
    input: u64,
    parameters: ActionParameters,
) -> ActionInvocation {
    ActionInvocation {
        input_id: InputId::new(input).unwrap(),
        action: ActionId::new("drawing.selection.move").unwrap(),
        source: InvocationSource::Keyboard,
        repeat: false,
        context: ActionContext {
            view: Some(view()),
            selection: group(),
            ..ActionContext::for_session(document.session())
        },
        parameters,
    }
}
#[test]
fn shared_move_action_previews_then_commits_exact_geometry_and_rejects_duplicate_delivery() {
    let directory = tempfile::tempdir().unwrap();
    let mut document = Document::new(setup(), JsonFile::new(directory.path().join("action.json")));
    document.save().unwrap();
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    let before = document.session().project().clone();
    let request = invocation(
        &document,
        1,
        ActionParameters::PreviewMove {
            reference: ObjectRef::Anchor(anchor(1)),
            placement: pointer(point(39900, 4990), SnapChoice::Best),
        },
    );
    let context = request.context.clone();
    let ActionOutcome::MovePreview(preview) = dispatcher
        .invoke(&registry, request, Some(&mut document))
        .unwrap()
    else {
        panic!("missing preview");
    };
    assert_eq!(document.session().project(), &before);
    assert!(!document.session().is_dirty());
    let projection = preview.layout().clone();
    let mut commit = invocation(&document, 2, ActionParameters::CommitMove(*preview));
    commit.context = context;
    let duplicate = commit.clone();
    assert!(matches!(
        dispatcher.invoke(&registry, commit, Some(&mut document)),
        Ok(ActionOutcome::Edited(true))
    ));
    assert_projection(document.session().project(), &projection);
    assert!(matches!(
        dispatcher.invoke(&registry, duplicate, Some(&mut document)),
        Err(ActionDispatchError::ReplayedInput(_))
    ));
    document.undo(document.session().revision()).unwrap();
    assert_eq!(document.session().project(), &before);
}
#[test]
fn preview_commit_checks_original_revision_and_selection_even_if_host_supplies_fresh_context() {
    let directory = tempfile::tempdir().unwrap();
    let mut document = Document::new(setup(), JsonFile::new(directory.path().join("action.json")));
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    let preview = preview_move(
        document.session(),
        group(),
        ObjectRef::Anchor(anchor(1)),
        MovePlacement::Exact(point(10, 20)),
    )
    .unwrap();
    let mut wrong = invocation(&document, 1, ActionParameters::CommitMove(preview.clone()));
    wrong.context.selection.remove(&ObjectRef::Label(label()));
    assert!(matches!(
        dispatcher.invoke(&registry, wrong, Some(&mut document)),
        Err(ActionDispatchError::WrongParameters)
    ));
    let other = preview_layout(
        document.session(),
        [ObjectRef::Label(label())],
        LayoutOperation::Translate(point(1, 1)),
    )
    .unwrap();
    document.apply_layout_preview(other).unwrap();
    let before = document.session().project().clone();
    let stale = invocation(&document, 2, ActionParameters::CommitMove(preview));
    assert!(matches!(
        dispatcher.invoke(&registry, stale, Some(&mut document)),
        Err(ActionDispatchError::Edit(EditError::StaleRevision { .. }))
    ));
    let bad = invocation(
        &document,
        3,
        ActionParameters::PreviewMove {
            reference: ObjectRef::Label(label()),
            placement: MovePlacement::Pointer {
                position: point(0, 0),
                settings: SnapSettings::default(),
                choice: SnapChoice::Candidate(0),
            },
        },
    );
    assert!(matches!(
        dispatcher.invoke(&registry, bad, Some(&mut document)),
        Err(ActionDispatchError::MovePreview(
            MovePreviewError::CandidateOutOfRange { .. }
        ))
    ));
    assert_eq!(document.session().project(), &before);
}
