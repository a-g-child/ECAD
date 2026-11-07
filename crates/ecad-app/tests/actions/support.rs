//! Stable logical-key fixtures, with no host keyboard or GUI assumptions.

use ecad_app::*;
use ecad_domain::*;

pub fn id(value: &str) -> ActionId {
    ActionId::new(value).unwrap()
}
pub fn chord(key: char, primary: bool, shift: bool) -> ShortcutChord {
    ShortcutChord {
        key: ShortcutKey::character(key).unwrap(),
        modifiers: ShortcutModifiers {
            primary,
            shift,
            ..ShortcutModifiers::default()
        },
    }
}
pub fn binding(scope: BindingScope, sequence: Vec<ShortcutChord>) -> ScopedShortcut {
    ScopedShortcut {
        scope,
        sequence: KeySequence::new(sequence).unwrap(),
    }
}
pub fn session() -> Session {
    let mut project = Project::new(ProjectId::new(1).unwrap());
    project
        .insert_drawing_view(DrawingView {
            id: ViewId::new(1).unwrap(),
            name: "Overview".into(),
            overrides: StylePatch::default(),
        })
        .unwrap();
    Session::new(project)
}
pub fn context(session: &Session) -> ActionContext {
    ActionContext {
        view: Some(ViewId::new(1).unwrap()),
        anchor: Some(DrawingPoint { x: 100, y: 200 }),
        ..ActionContext::for_session(session)
    }
}
pub fn router(registry: &ActionRegistry) -> ActionInputRouter {
    ActionInputRouter::new(
        compile_keymap(
            registry,
            &KeymapOverrides::default(),
            ShortcutPlatform::Linux,
        )
        .unwrap(),
        std::num::NonZeroU64::new(1_000).unwrap(),
    )
}
pub fn event(number: u64, key: ShortcutChord, repeat: bool) -> ActionKeyEvent {
    ActionKeyEvent {
        id: InputId::new(number).unwrap(),
        chord: key.resolve(ShortcutPlatform::Linux),
        repeat,
    }
}
