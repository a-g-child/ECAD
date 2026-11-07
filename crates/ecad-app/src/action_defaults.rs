//! Explicit defaults for working headless operations; no unimplemented wire-tool action.

use crate::*;

fn chord(key: char, primary: bool, shift: bool) -> ShortcutChord {
    ShortcutChord {
        key: ShortcutKey::character(key).expect("printable constant"),
        modifiers: ShortcutModifiers {
            primary,
            shift,
            ..ShortcutModifiers::default()
        },
    }
}
fn binding(scope: BindingScope, chords: Vec<ShortcutChord>) -> ScopedShortcut {
    ScopedShortcut {
        scope,
        sequence: KeySequence::new(chords).expect("nonempty constant"),
    }
}
impl ActionRegistry {
    /// Register shipped working handlers and their explicit default/unbound policies.
    /// Direct T/P request annotation/polyline placement; W is reserved for a future
    /// electrical route tool rather than advertising unsupported wire behavior.
    pub fn builtins() -> Self {
        use ActionHandler as Handler;
        use BindingScope as Scope;
        let entries = [
            (
                "document.undo",
                "Undo",
                "Document",
                "Undo the last document transaction",
                Handler::Undo,
                Scope::Document,
                ActionRepeat::Allow,
                vec![binding(Scope::Document, vec![chord('z', true, false)])],
            ),
            (
                "document.redo",
                "Redo",
                "Document",
                "Redo an undone document transaction",
                Handler::Redo,
                Scope::Document,
                ActionRepeat::Allow,
                vec![binding(Scope::Document, vec![chord('z', true, true)])],
            ),
            (
                "document.save",
                "Save",
                "Document",
                "Save using observed-version conflict checks",
                Handler::Save,
                Scope::Document,
                ActionRepeat::Ignore,
                vec![binding(Scope::Document, vec![chord('s', true, false)])],
            ),
            (
                "selection.properties",
                "Properties",
                "Inspection",
                "Inspect all applicable fields of selected objects",
                Handler::Properties,
                Scope::Document,
                ActionRepeat::Ignore,
                vec![
                    binding(Scope::Canvas, vec![chord('e', false, false)]),
                    binding(Scope::ProjectTree, vec![chord('e', false, false)]),
                ],
            ),
            (
                "drawing.label.create",
                "Create annotation",
                "Drawing",
                "Request text and placement, then create an annotation",
                Handler::CreateLabel,
                Scope::Canvas,
                ActionRepeat::Ignore,
                vec![binding(Scope::Canvas, vec![chord('t', false, false)])],
            ),
            (
                "drawing.polyline.create",
                "Create graphical polyline",
                "Drawing",
                "Request vertices, then create a whole graphical path",
                Handler::CreatePolyline,
                Scope::Canvas,
                ActionRepeat::Ignore,
                vec![binding(Scope::Canvas, vec![chord('p', false, false)])],
            ),
            (
                "drawing.styles.edit",
                "Drawing defaults and styles",
                "Drawing",
                "Inspect or edit portable fallbacks and creation presets",
                Handler::DrawingSettings,
                Scope::Document,
                ActionRepeat::Ignore,
                vec![binding(
                    Scope::Document,
                    vec![chord('k', true, false), chord('s', false, false)],
                )],
            ),
            (
                "workspace.palette",
                "Command palette",
                "Workspace",
                "Search registered actions including unavailable operations",
                Handler::Palette,
                Scope::Application,
                ActionRepeat::Ignore,
                vec![binding(Scope::Application, vec![chord('p', true, true)])],
            ),
            (
                "placement.cancel",
                "Cancel placement",
                "Drawing",
                "Cancel host-owned transient placement without document edits",
                Handler::CancelPlacement,
                Scope::Tool,
                ActionRepeat::Ignore,
                vec![binding(
                    Scope::Tool,
                    vec![ShortcutChord {
                        key: ShortcutKey::named(NamedKey::Escape),
                        modifiers: ShortcutModifiers::default(),
                    }],
                )],
            ),
        ];
        let mut registry = Self::default();
        for (id, label, category, description, handler, scope, repeat, defaults) in entries {
            registry
                .register(ActionDefinition {
                    id: ActionId::new(id).expect("stable constant"),
                    label: label.into(),
                    category: category.into(),
                    description: description.into(),
                    search_terms: vec![],
                    scope,
                    handler,
                    repeat,
                    defaults: DefaultBindings::Suggested(defaults),
                })
                .expect("unique valid builtins");
        }
        registry
    }
}
