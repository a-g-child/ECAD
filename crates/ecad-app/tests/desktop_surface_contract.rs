//! Host lifecycle/viewport requests use the same registry rather than bespoke UI shortcuts.
use ecad_app::*;
use ecad_domain::*;
#[test]
fn open_project_is_available_without_a_document_and_fit_requires_a_real_canvas_view() {
    let registry = ActionRegistry::builtins();
    let open = ActionId::new("document.open").unwrap();
    let fit = ActionId::new("drawing.view.fit").unwrap();
    assert!(
        registry
            .state(&open, &ActionContext::default(), None)
            .enabled()
    );
    let mut project = Project::new(ProjectId::new(1).unwrap());
    let view = ViewId::new(1).unwrap();
    project
        .insert_drawing_view(DrawingView {
            id: view,
            name: "Drawing".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    let session = Session::new(project);
    let mut context = ActionContext::for_session(&session);
    assert_eq!(
        registry.state(&fit, &context, Some(&session)).disabled,
        Some(ActionDisabled::NoDrawingView)
    );
    context.view = Some(view);
    assert!(registry.state(&fit, &context, Some(&session)).enabled());
    context.focus = ActionFocus::TextInput;
    assert_eq!(
        registry.state(&fit, &context, Some(&session)).disabled,
        Some(ActionDisabled::CanvasRequired)
    );
    assert!(matches!(
        registry.get(&open).unwrap().defaults,
        DefaultBindings::Suggested(_)
    ));
}
