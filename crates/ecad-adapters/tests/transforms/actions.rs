//! Shared R/mirror action requests, exact preview/commit, and closed-proposal guards.
use super::*;
fn invocation(
    document: &Document<JsonFile>,
    input: u64,
    action: &str,
    parameters: ActionParameters,
) -> ActionInvocation {
    ActionInvocation {
        input_id: InputId::new(input).unwrap(),
        action: ActionId::new(action).unwrap(),
        source: InvocationSource::ContextMenu,
        repeat: false,
        context: ActionContext {
            selection: selection(),
            view: Some(view()),
            ..ActionContext::for_session(document.session())
        },
        parameters,
    }
}
#[test]
fn rotation_and_mirror_actions_request_parameters_preview_then_commit_one_exact_operation() {
    let directory = tempfile::tempdir().unwrap();
    let before = setup();
    let mut document = Document::new(
        before.clone(),
        JsonFile::new(directory.path().join("actions.json")),
    );
    document.save().unwrap();
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    for (index, (action, kind, transform)) in [
        (
            "drawing.selection.rotate",
            ActionRequestKind::RotateSelection,
            rotate(),
        ),
        (
            "drawing.selection.mirror",
            ActionRequestKind::MirrorSelection,
            DrawingTransform::Mirror {
                pivot: point(10, 20),
                axis: MirrorAxis::Horizontal,
            },
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let id = u64::try_from(index).unwrap() * 3 + 1;
        let request = invocation(&document, id, action, ActionParameters::None);
        assert_eq!(
            registry.get(&request.action).unwrap().input_requirement(),
            ActionInputRequirement::Layout
        );
        let ActionOutcome::Request(request) = dispatcher
            .invoke(&registry, request, Some(&mut document))
            .unwrap()
        else {
            panic!("no parameter request");
        };
        assert_eq!(request.kind, kind);
        let preview = invocation(
            &document,
            id + 1,
            action,
            ActionParameters::PreviewLayout(LayoutOperation::Transform(transform)),
        );
        let ActionOutcome::LayoutPreview(preview) = dispatcher
            .invoke(&registry, preview, Some(&mut document))
            .unwrap()
        else {
            panic!("no layout preview");
        };
        assert_eq!(document.session().project(), &before);
        assert!(!document.session().is_dirty());
        let commit = invocation(
            &document,
            id + 2,
            action,
            ActionParameters::CommitLayout(*preview.clone()),
        );
        assert!(matches!(
            dispatcher.invoke(&registry, commit, Some(&mut document)),
            Ok(ActionOutcome::Edited(true))
        ));
        assert_projection(document.session().project(), &preview);
        assert_eq!(document.session().undo_len(), 1);
        document.undo(document.session().revision()).unwrap();
        assert_eq!(document.session().project(), &before);
        assert!(!document.session().is_dirty());
    }
    let action = registry
        .get(&ActionId::new("drawing.selection.rotate").unwrap())
        .unwrap();
    let DefaultBindings::Suggested(bindings) = &action.defaults else {
        panic!("missing rotation key");
    };
    assert_eq!(
        bindings[0].sequence.chords()[0].key,
        ShortcutKey::character('r').unwrap()
    );
    assert_eq!(
        registry
            .get(&ActionId::new("drawing.selection.mirror").unwrap())
            .unwrap()
            .defaults,
        DefaultBindings::Unbound
    );
}
#[test]
fn actions_reject_incompatible_closed_previews_changed_selection_and_fresh_context_for_stale_preview()
 {
    let directory = tempfile::tempdir().unwrap();
    let mut document = Document::new(
        setup(),
        JsonFile::new(directory.path().join("actions.json")),
    );
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    let mirror = DrawingTransform::Mirror {
        pivot: point(0, 0),
        axis: MirrorAxis::Vertical,
    };
    let preview = preview_layout(
        document.session(),
        selection(),
        LayoutOperation::Transform(mirror),
    )
    .unwrap();
    let wrong = invocation(
        &document,
        1,
        "drawing.selection.rotate",
        ActionParameters::CommitLayout(preview.clone()),
    );
    assert!(matches!(
        dispatcher.invoke(&registry, wrong, Some(&mut document)),
        Err(ActionDispatchError::WrongParameters)
    ));
    let mut wrong = invocation(
        &document,
        2,
        "drawing.selection.mirror",
        ActionParameters::CommitLayout(preview.clone()),
    );
    wrong.context.selection.remove(&ObjectRef::Label(label()));
    assert!(matches!(
        dispatcher.invoke(&registry, wrong, Some(&mut document)),
        Err(ActionDispatchError::WrongParameters)
    ));
    let wrong = invocation(
        &document,
        3,
        "drawing.selection.rotate",
        ActionParameters::Layout(LayoutOperation::Transform(mirror)),
    );
    assert!(matches!(
        dispatcher.invoke(&registry, wrong, Some(&mut document)),
        Err(ActionDispatchError::WrongParameters)
    ));
    let other = preview_layout(
        document.session(),
        selection(),
        LayoutOperation::Transform(rotate()),
    )
    .unwrap();
    document.apply_layout_preview(other).unwrap();
    let before = document.session().project().clone();
    let stale = invocation(
        &document,
        4,
        "drawing.selection.mirror",
        ActionParameters::CommitLayout(preview),
    );
    assert!(matches!(
        dispatcher.invoke(&registry, stale, Some(&mut document)),
        Err(ActionDispatchError::Edit(EditError::StaleRevision { .. }))
    ));
    assert_eq!(document.session().project(), &before);
}
