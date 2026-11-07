//! Shared read-only report action honors normal availability and protected input.
use super::*;
#[test]
fn report_action_is_remappable_and_returns_current_values_without_any_document_or_file_mutation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("report.json");
    let mut document = Document::new(project(), JsonFile::new(&path));
    let registry = ActionRegistry::builtins();
    let action = ActionId::new("reports.connections").unwrap();
    let definition = registry.get(&action).unwrap();
    assert_eq!(definition.defaults, DefaultBindings::Unbound);
    assert_eq!(definition.input_requirement(), ActionInputRequirement::None);
    let invocation = ActionInvocation {
        input_id: InputId::new(1).unwrap(),
        action: action.clone(),
        source: InvocationSource::Menu,
        repeat: false,
        context: ActionContext::for_session(document.session()),
        parameters: ActionParameters::None,
    };
    let before = document.session().project().clone();
    let revision = document.session().revision();
    let dirty = document.session().is_dirty();
    let mut dispatcher = ActionDispatcher::default();
    let ActionOutcome::ConnectionReport(report) = dispatcher
        .invoke(&registry, invocation.clone(), Some(&mut document))
        .unwrap()
    else {
        panic!("no connection report");
    };
    assert!(report.is_current(document.session()));
    assert_eq!(report.terminals().len(), 3);
    assert_eq!(document.session().project(), &before);
    assert_eq!(document.session().revision(), revision);
    assert_eq!(document.session().is_dirty(), dirty);
    assert_eq!(document.session().undo_len(), 0);
    assert!(!path.exists());
    assert!(matches!(
        dispatcher.invoke(&registry, invocation.clone(), Some(&mut document)),
        Err(ActionDispatchError::ReplayedInput(_))
    ));
    let mut protected = invocation.clone();
    protected.input_id = InputId::new(2).unwrap();
    protected.source = InvocationSource::Keyboard;
    protected.context.composing = true;
    assert!(matches!(
        dispatcher.invoke(&registry, protected, Some(&mut document)),
        Err(ActionDispatchError::ProtectedInput)
    ));
    let mut absent = invocation;
    absent.input_id = InputId::new(3).unwrap();
    assert!(matches!(
        dispatcher.invoke::<JsonFile>(&registry, absent, None),
        Err(ActionDispatchError::Disabled(ActionDisabled::NoDocument))
    ));
}
