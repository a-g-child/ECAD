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

#[test]
fn blank_sheet_fit_uses_captured_identity_and_rejects_missing_or_mismatched_sheets() {
    let registry = ActionRegistry::builtins();
    let fit = ActionId::new("drawing.view.fit").unwrap();
    let mut p = Project::new(ProjectId::new(1).unwrap());
    let id = SheetId::new(1).unwrap();
    p.insert_sheet(
        DrawingSheet {
            inherited_variant: None,
            kind: ecad_domain::SheetKind::Schematic,
            template: None,
            id,
            number: "COVER".into(),
            title: "Cover".into(),
            paper: SheetPaper::default(),
            placement: None,
        },
        0,
    )
    .unwrap();
    let session = Session::new(p);
    let mut context = ActionContext::for_session(&session);
    context.sheet = Some(id);
    assert!(registry.state(&fit, &context, Some(&session)).enabled());
    context.view = Some(ViewId::new(99).unwrap());
    assert_eq!(
        registry.state(&fit, &context, Some(&session)).disabled,
        Some(ActionDisabled::InvalidSheetContext(id))
    );
    context.view = None;
    context.sheet = Some(SheetId::new(99).unwrap());
    assert_eq!(
        registry.state(&fit, &context, Some(&session)).disabled,
        Some(ActionDisabled::InvalidSheetContext(
            SheetId::new(99).unwrap()
        ))
    );
    context.sheet = Some(id);
    context.focus = ActionFocus::TextInput;
    assert_eq!(
        registry.state(&fit, &context, Some(&session)).disabled,
        Some(ActionDisabled::CanvasRequired)
    );
}

#[test]
fn explorer_modes_are_discoverable_unbound_document_requests() {
    let registry = ActionRegistry::builtins();
    let session = Session::new(Project::new(ProjectId::new(1).unwrap()));
    let context = ActionContext::for_session(&session);
    for name in ["workspace.explorer.pages", "workspace.explorer.electrical"] {
        let id = ActionId::new(name).unwrap();
        assert!(registry.state(&id, &context, Some(&session)).enabled());
        assert_eq!(
            registry.state(&id, &context, None).disabled,
            Some(ActionDisabled::NoDocument)
        );
        assert!(matches!(
            registry.get(&id).unwrap().defaults,
            DefaultBindings::Unbound
        ));
    }
}
