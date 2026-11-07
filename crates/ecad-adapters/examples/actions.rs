//! Route T into a captured placement request, create, undo, redo, and guarded-save.

use ecad_adapters::{JsonFile, RandomIds};
use ecad_app::*;
use ecad_domain::*;
use std::num::NonZeroU64;

fn call(
    number: u64,
    action: &str,
    document: &Document<JsonFile>,
) -> Result<ActionInvocation, Box<dyn std::error::Error>> {
    Ok(ActionInvocation {
        input_id: InputId::new(number).ok_or("input ID is zero")?,
        action: ActionId::new(action)?,
        source: InvocationSource::Menu,
        repeat: false,
        context: ActionContext::for_session(document.session()),
        parameters: ActionParameters::None,
    })
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let store = JsonFile::new(directory.path().join("actions.json"));
    let mut project = Project::new(RandomIds::project()?);
    let view = RandomIds::view()?;
    project.insert_drawing_view(DrawingView {
        id: view,
        name: "Overview".into(),
        overrides: StylePatch::default(),
    })?;
    let mut document = Document::new(project, store.clone());
    document.save()?;
    let registry = ActionRegistry::builtins();
    let map = compile_keymap(
        &registry,
        &KeymapOverrides::default(),
        ShortcutPlatform::MacOS,
    )
    .map_err(|issues| format!("invalid defaults: {issues:?}"))?;
    println!(
        "Save binding: {}",
        map.shortcuts(&ActionId::new("document.save")?).join(" / ")
    );
    let mut router = ActionInputRouter::new(map, NonZeroU64::new(1_000).ok_or("zero timeout")?);
    let mut dispatcher = ActionDispatcher::default();
    let captured = ActionContext {
        view: Some(view),
        anchor: Some(DrawingPoint {
            x: 10_000,
            y: 5_000,
        }),
        ..ActionContext::for_session(document.session())
    };
    let key = ResolvedChord {
        key: ShortcutKey::character('t')?,
        modifiers: PhysicalModifiers::default(),
    };
    let event = ActionKeyEvent {
        id: InputId::new(1).ok_or("zero input")?,
        chord: key,
        repeat: false,
    };
    let KeyRoute::Invoke(invocation) = router.route(&registry, event, &captured, 0) else {
        return Err("T did not route".into());
    };
    let ActionOutcome::Request(request) =
        dispatcher.invoke(&registry, *invocation, Some(&mut document))?
    else {
        return Err("missing placement request".into());
    };
    if document.session().undo_len() != 0 || request.context.anchor != captured.anchor {
        return Err("placement request edited or retargeted the document".into());
    }
    let create = ActionInvocation {
        input_id: InputId::new(2).ok_or("zero input")?,
        action: request.action,
        source: InvocationSource::Palette,
        repeat: false,
        context: request.context,
        parameters: ActionParameters::Label {
            id: RandomIds::label()?,
            text: "Action-created annotation".into(),
            position: captured.anchor.ok_or("request needs placement")?,
        },
    };
    dispatcher.invoke(&registry, create.clone(), Some(&mut document))?;
    if !matches!(
        dispatcher.invoke(&registry, create, Some(&mut document)),
        Err(ActionDispatchError::ReplayedInput(_))
    ) {
        return Err("duplicate delivery was not rejected".into());
    }
    dispatcher.invoke(
        &registry,
        call(3, "document.undo", &document)?,
        Some(&mut document),
    )?;
    if document.session().is_dirty() {
        return Err("undo did not restore saved checkpoint".into());
    }
    dispatcher.invoke(
        &registry,
        call(4, "document.redo", &document)?,
        Some(&mut document),
    )?;
    dispatcher.invoke(
        &registry,
        call(5, "document.save", &document)?,
        Some(&mut document),
    )?;
    if Document::open(store)?.session().project() != document.session().project() {
        return Err("action save/reopen changed content".into());
    }
    println!(
        "One shared dispatch path: T requested placement, explicit parameters created one annotation, duplicate input was rejected, and undo/redo/save/reopen passed."
    );
    Ok(())
}
