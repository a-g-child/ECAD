//! Registry, captured selection, and real document layout invocation.
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
            view: Some(view()),
            selection: (1..=4).map(|id| ObjectRef::Label(label(id))).collect(),
            ..ActionContext::for_session(document.session())
        },
        parameters,
    }
}

#[test]
fn all_layout_actions_request_typed_input_and_commit_through_one_shared_transaction() {
    let directory = tempfile::tempdir().unwrap();
    let before = setup();
    let mut document = Document::new(
        before.clone(),
        JsonFile::new(directory.path().join("layout.json")),
    );
    document.save().unwrap();
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    for (index, (action, kind, operation)) in [
        (
            "drawing.selection.move",
            ActionRequestKind::MoveSelection,
            LayoutOperation::Translate(point(7, 11)),
        ),
        (
            "drawing.selection.align",
            ActionRequestKind::AlignSelection,
            LayoutOperation::Align {
                axis: LayoutAxis::Y,
                coordinate_um: 37,
            },
        ),
        (
            "drawing.selection.distribute",
            ActionRequestKind::DistributeSelection,
            LayoutOperation::Distribute(LayoutAxis::X),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let input = u64::try_from(index).unwrap() * 2 + 1;
        let original = invocation(&document, input, action, ActionParameters::None);
        let captured = original.context.clone();
        assert_eq!(
            registry.get(&original.action).unwrap().input_requirement(),
            ActionInputRequirement::Layout
        );
        let ActionOutcome::Request(request) = dispatcher
            .invoke(&registry, original, Some(&mut document))
            .unwrap()
        else {
            panic!("missing input request");
        };
        assert_eq!(request.kind, kind);
        assert_eq!(request.context, captured);
        assert_eq!(document.session().project(), &before);
        let mut reply = invocation(
            &document,
            input + 1,
            action,
            ActionParameters::Layout(operation),
        );
        reply.context = request.context;
        assert!(matches!(
            dispatcher.invoke(&registry, reply, Some(&mut document)),
            Ok(ActionOutcome::Edited(true))
        ));
        assert_eq!(document.session().undo_len(), 1);
        document.undo(document.session().revision()).unwrap();
        assert_eq!(document.session().project(), &before);
        assert!(!document.session().is_dirty());
    }
    let move_action = registry
        .get(&ActionId::new("drawing.selection.move").unwrap())
        .unwrap();
    let DefaultBindings::Suggested(bindings) = &move_action.defaults else {
        panic!("missing move binding");
    };
    assert_eq!(
        bindings[0].sequence.chords()[0].key,
        ShortcutKey::character('m').unwrap()
    );
    for name in ["drawing.selection.align", "drawing.selection.distribute"] {
        assert_eq!(
            registry
                .get(&ActionId::new(name).unwrap())
                .unwrap()
                .defaults,
            DefaultBindings::Unbound
        );
    }
}

#[test]
fn layout_actions_reject_stale_replies_wrong_parameter_kinds_and_invalid_active_views() {
    let directory = tempfile::tempdir().unwrap();
    let mut document = Document::new(setup(), JsonFile::new(directory.path().join("layout.json")));
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    let stale = invocation(
        &document,
        2,
        "drawing.selection.move",
        ActionParameters::Layout(LayoutOperation::Translate(point(1, 1))),
    );
    let op = invocation(
        &document,
        1,
        "drawing.selection.align",
        ActionParameters::Layout(LayoutOperation::Align {
            axis: LayoutAxis::Y,
            coordinate_um: 42,
        }),
    );
    dispatcher
        .invoke(&registry, op, Some(&mut document))
        .unwrap();
    let before = document.session().project().clone();
    assert!(matches!(
        dispatcher.invoke(&registry, stale, Some(&mut document)),
        Err(ActionDispatchError::Disabled(ActionDisabled::StaleContext))
    ));
    let wrong = invocation(
        &document,
        3,
        "drawing.selection.move",
        ActionParameters::Layout(LayoutOperation::Distribute(LayoutAxis::X)),
    );
    assert!(matches!(
        dispatcher.invoke(&registry, wrong, Some(&mut document)),
        Err(ActionDispatchError::WrongParameters)
    ));
    let mut invalid = invocation(
        &document,
        4,
        "drawing.selection.move",
        ActionParameters::None,
    );
    invalid.context.selection.insert(ObjectRef::Route(route(1)));
    assert!(matches!(
        dispatcher.invoke(&registry, invalid, Some(&mut document)),
        Err(ActionDispatchError::Disabled(ActionDisabled::Layout(
            LayoutError::UnsupportedTarget(_)
        )))
    ));
    let mut invalid = invocation(
        &document,
        5,
        "drawing.selection.move",
        ActionParameters::None,
    );
    invalid.context.view = None;
    assert!(matches!(
        dispatcher.invoke(&registry, invalid, Some(&mut document)),
        Err(ActionDispatchError::Disabled(ActionDisabled::NoDrawingView))
    ));
    let mut protected = invocation(
        &document,
        6,
        "drawing.selection.move",
        ActionParameters::None,
    );
    protected.source = InvocationSource::Keyboard;
    protected.context.composing = true;
    assert!(matches!(
        dispatcher.invoke(&registry, protected, Some(&mut document)),
        Err(ActionDispatchError::ProtectedInput)
    ));
    assert_eq!(document.session().project(), &before);
}
