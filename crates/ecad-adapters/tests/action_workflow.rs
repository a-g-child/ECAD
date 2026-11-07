//! Real creation/history/save dispatch and captured transient-UI target regressions.

#[path = "actions/support.rs"]
mod support;
use ecad_adapters::{JsonFile, ProjectFileError};
use ecad_app::*;
use ecad_domain::*;
use std::fs;
use support::*;

#[test]
fn real_dispatch_creates_undoes_redoes_and_saves_with_native_web_duplicate_protection() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let mut document = document(&path);
    let before = document.session().project().clone();
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    let mut create = invocation(1, "drawing.label.create", context(&document), label(1));
    create.source = InvocationSource::Keyboard;
    assert!(matches!(
        dispatcher.invoke(&registry, create.clone(), Some(&mut document)),
        Ok(ActionOutcome::Edited(true))
    ));
    assert_eq!(document.session().undo_len(), 1);
    assert!(document.session().is_dirty());
    create.source = InvocationSource::NativeAccelerator;
    assert!(matches!(
        dispatcher.invoke(&registry, create, Some(&mut document)),
        Err(ActionDispatchError::ReplayedInput(_))
    ));
    assert_eq!(document.session().project().drawing().labels().len(), 1);
    assert!(matches!(
        dispatcher.invoke(
            &registry,
            invocation(
                2,
                "document.undo",
                context(&document),
                ActionParameters::None
            ),
            Some(&mut document)
        ),
        Ok(ActionOutcome::Edited(true))
    ));
    assert_eq!(document.session().project(), &before);
    assert!(!document.session().is_dirty());
    dispatcher
        .invoke(
            &registry,
            invocation(
                3,
                "document.redo",
                context(&document),
                ActionParameters::None,
            ),
            Some(&mut document),
        )
        .unwrap();
    dispatcher
        .invoke(
            &registry,
            invocation(
                4,
                "document.save",
                context(&document),
                ActionParameters::None,
            ),
            Some(&mut document),
        )
        .unwrap();
    assert!(!document.session().is_dirty());
    assert_eq!(
        Document::open(JsonFile::new(path))
            .unwrap()
            .session()
            .project(),
        document.session().project()
    );
}

#[test]
fn placement_requests_keep_captured_targets_and_do_not_invent_a_missing_anchor() {
    let directory = tempfile::tempdir().unwrap();
    let mut document = document(&directory.path().join("project.json"));
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    let original = context(&document);
    let ActionOutcome::Request(request) = dispatcher
        .invoke(
            &registry,
            invocation(
                1,
                "drawing.label.create",
                original.clone(),
                ActionParameters::None,
            ),
            Some(&mut document),
        )
        .unwrap()
    else {
        panic!("no input request");
    };
    assert_eq!(request.kind, ActionRequestKind::LabelPlacement);
    assert_eq!(request.context.anchor, original.anchor);
    assert_eq!(document.session().undo_len(), 0);
    // Menu/palette pointer positions are not input fields and never replace this anchor.
    let parameters = ActionParameters::Label {
        id: LabelId::new(1).unwrap(),
        text: "Captured".into(),
        position: request.context.anchor.unwrap(),
    };
    dispatcher
        .invoke(
            &registry,
            invocation(2, "drawing.label.create", request.context, parameters),
            Some(&mut document),
        )
        .unwrap();
    assert_eq!(
        document
            .session()
            .project()
            .drawing()
            .label(LabelId::new(1).unwrap())
            .unwrap()
            .position,
        original.anchor.unwrap()
    );
    let mut no_anchor = context(&document);
    no_anchor.anchor = None;
    let ActionOutcome::Request(request) = dispatcher
        .invoke(
            &registry,
            invocation(
                3,
                "drawing.polyline.create",
                no_anchor,
                ActionParameters::None,
            ),
            Some(&mut document),
        )
        .unwrap()
    else {
        panic!("no placement request");
    };
    assert_eq!(request.context.anchor, None);
    assert_eq!(document.session().project().drawing().polylines().len(), 0);
}

#[test]
fn delayed_parameter_replies_and_unavailable_views_are_revalidated_without_partial_creation() {
    let directory = tempfile::tempdir().unwrap();
    let mut document = document(&directory.path().join("project.json"));
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    let captured = context(&document);
    document
        .execute(
            document.session().revision(),
            Command::Drawing(DrawingCommand::ReplaceView(DrawingView {
                id: ViewId::new(1).unwrap(),
                name: "Changed".into(),
                overrides: StylePatch::default(),
            })),
        )
        .unwrap();
    let before = document.session().project().clone();
    assert!(matches!(
        dispatcher.invoke(
            &registry,
            invocation(1, "drawing.label.create", captured, label(1)),
            Some(&mut document)
        ),
        Err(ActionDispatchError::Disabled(ActionDisabled::StaleContext))
    ));
    assert_eq!(document.session().project(), &before);
    document
        .execute(
            document.session().revision(),
            Command::Drawing(DrawingCommand::RemoveView(ViewId::new(1).unwrap())),
        )
        .unwrap();
    assert!(matches!(
        dispatcher.invoke(
            &registry,
            invocation(2, "drawing.label.create", context(&document), label(1)),
            Some(&mut document)
        ),
        Err(ActionDispatchError::Disabled(ActionDisabled::NoDrawingView))
    ));
    assert_eq!(document.session().project().drawing().labels().len(), 0);
}

#[test]
fn all_surfaces_share_properties_and_undo_never_steals_text_or_composition_history() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let mut document = document(&path);
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    dispatcher
        .invoke(
            &registry,
            invocation(1, "drawing.label.create", context(&document), label(1)),
            Some(&mut document),
        )
        .unwrap();
    let mut selected = context(&document);
    selected
        .selection
        .insert(ObjectRef::Label(LabelId::new(1).unwrap()));
    let expected = inspect_properties(
        document.session().project(),
        selected.selection.iter().copied(),
    )
    .unwrap();
    let bytes = fs::read(&path).unwrap();
    let before = document.session().project().clone();
    for (index, source) in [
        InvocationSource::Keyboard,
        InvocationSource::NativeAccelerator,
        InvocationSource::Menu,
        InvocationSource::Toolbar,
        InvocationSource::Palette,
        InvocationSource::ContextMenu,
    ]
    .into_iter()
    .enumerate()
    {
        let mut call = invocation(
            u64::try_from(index).unwrap() + 2,
            "selection.properties",
            selected.clone(),
            ActionParameters::None,
        );
        call.source = source;
        let ActionOutcome::Properties(properties) = dispatcher
            .invoke(&registry, call, Some(&mut document))
            .unwrap()
        else {
            panic!("missing inspector");
        };
        assert_eq!(properties, expected);
    }
    for (index, source) in [
        InvocationSource::Keyboard,
        InvocationSource::NativeAccelerator,
        InvocationSource::Menu,
    ]
    .into_iter()
    .enumerate()
    {
        let mut editing = context(&document);
        editing.focus = ActionFocus::TextInput;
        let mut call = invocation(
            u64::try_from(index).unwrap() + 8,
            "document.undo",
            editing,
            ActionParameters::None,
        );
        call.source = source;
        assert!(
            dispatcher
                .invoke(&registry, call, Some(&mut document))
                .is_err()
        );
    }
    let mut composing = context(&document);
    composing.composing = true;
    assert!(
        dispatcher
            .invoke(
                &registry,
                invocation(11, "document.undo", composing, ActionParameters::None),
                Some(&mut document)
            )
            .is_err()
    );
    assert_eq!(document.session().project(), &before);
    assert_eq!(fs::read(path).unwrap(), bytes);
}

#[test]
fn save_conflicts_keep_external_bytes_and_original_typed_error_without_marking_clean() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("project.json");
    let mut document = document(&path);
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    dispatcher
        .invoke(
            &registry,
            invocation(1, "drawing.label.create", context(&document), label(1)),
            Some(&mut document),
        )
        .unwrap();
    fs::write(&path, "external change").unwrap();
    let before = document.session().project().clone();
    let history = document.session().undo_len();
    let error = dispatcher
        .invoke(
            &registry,
            invocation(
                2,
                "document.save",
                context(&document),
                ActionParameters::None,
            ),
            Some(&mut document),
        )
        .err()
        .unwrap();
    assert!(matches!(
        error,
        ActionDispatchError::Store(ProjectFileError::ExternalChange(_))
    ));
    assert!(std::error::Error::source(&error).is_some());
    assert_eq!(document.session().project(), &before);
    assert_eq!(document.session().undo_len(), history);
    assert!(document.session().is_dirty());
    assert_eq!(fs::read_to_string(path).unwrap(), "external change");
}

#[test]
fn failed_attempts_consume_input_ids_and_model_parameter_errors_leave_history_intact() {
    let directory = tempfile::tempdir().unwrap();
    let mut document = document(&directory.path().join("project.json"));
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    let before = document.session().project().clone();
    assert!(matches!(
        dispatcher.invoke(
            &registry,
            invocation(1, "document.save", context(&document), label(1)),
            Some(&mut document)
        ),
        Err(ActionDispatchError::WrongParameters)
    ));
    assert!(matches!(
        dispatcher.invoke(
            &registry,
            invocation(
                1,
                "document.save",
                context(&document),
                ActionParameters::None
            ),
            Some(&mut document)
        ),
        Err(ActionDispatchError::ReplayedInput(_))
    ));
    let invalid = ActionParameters::Label {
        id: LabelId::new(1).unwrap(),
        text: "  ".into(),
        position: DrawingPoint::default(),
    };
    assert!(matches!(
        dispatcher.invoke(
            &registry,
            invocation(2, "drawing.label.create", context(&document), invalid),
            Some(&mut document)
        ),
        Err(ActionDispatchError::Edit(_))
    ));
    let mut repeated = invocation(3, "drawing.label.create", context(&document), label(1));
    repeated.source = InvocationSource::NativeAccelerator;
    repeated.repeat = true;
    assert!(matches!(
        dispatcher.invoke(&registry, repeated, Some(&mut document)),
        Err(ActionDispatchError::RepeatedInput)
    ));
    assert_eq!(document.session().project(), &before);
    assert_eq!(document.session().undo_len(), 0);
    assert!(!document.session().is_dirty());
}

#[test]
fn application_requests_work_without_a_document_and_settings_replies_use_normal_commands() {
    let registry = ActionRegistry::builtins();
    let mut dispatcher = ActionDispatcher::default();
    let result = dispatcher
        .invoke(
            &registry,
            invocation(
                1,
                "workspace.palette",
                ActionContext::default(),
                ActionParameters::None,
            ),
            None::<&mut Document<JsonFile>>,
        )
        .unwrap();
    assert!(matches!(
        result,
        ActionOutcome::Request(request) if request.kind == ActionRequestKind::Palette
    ));
    let directory = tempfile::tempdir().unwrap();
    let mut document = document(&directory.path().join("project.json"));
    let ActionOutcome::Request(request) = dispatcher
        .invoke(
            &registry,
            invocation(
                2,
                "drawing.styles.edit",
                context(&document),
                ActionParameters::None,
            ),
            Some(&mut document),
        )
        .unwrap()
    else {
        panic!("no settings request");
    };
    let mut settings = request.settings.unwrap();
    settings.base.text_height = DrawingLength::new(5_000).unwrap();
    let before = document.session().project().clone();
    dispatcher
        .invoke(
            &registry,
            invocation(
                3,
                "drawing.styles.edit",
                request.context,
                ActionParameters::DrawingSettings(settings),
            ),
            Some(&mut document),
        )
        .unwrap();
    assert_eq!(document.session().undo_len(), 1);
    dispatcher
        .invoke(
            &registry,
            invocation(
                4,
                "document.undo",
                context(&document),
                ActionParameters::None,
            ),
            Some(&mut document),
        )
        .unwrap();
    assert_eq!(document.session().project(), &before);
    assert!(matches!(
        dispatcher.invoke(
            &registry,
            invocation(5, "workspace.palette", ActionContext::default(), label(1)),
            Some(&mut document)
        ),
        Err(ActionDispatchError::WrongParameters)
    ));
    let cancelling = ActionContext {
        tool_active: true,
        ..ActionContext::default()
    };
    assert!(matches!(
        dispatcher.invoke(
            &registry,
            invocation(6, "placement.cancel", cancelling, label(1)),
            Some(&mut document)
        ),
        Err(ActionDispatchError::WrongParameters)
    ));
}
