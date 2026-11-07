//! Explicit defaults for implemented headless workflows and typed placement requests.

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
    /// Direct T/P request annotation/polyline placement; W requests a whole route
    /// for an existing conductor. Interactive placement and new-wire creation remain host work.
    pub fn builtins() -> Self {
        use ActionHandler as Handler;
        use BindingScope as Scope;
        let entries = [
            (
                "document.open",
                "Open project",
                "Document",
                "Request a host file chooser with protected document replacement",
                Handler::OpenProject,
                Scope::Application,
                ActionRepeat::Ignore,
                vec![binding(Scope::Application, vec![chord('o', true, false)])],
            ),
            (
                "drawing.view.fit",
                "Fit drawing",
                "View",
                "Fit the active drawing geometry to the available viewport",
                Handler::FitDrawing,
                Scope::Canvas,
                ActionRepeat::Ignore,
                vec![binding(Scope::Canvas, vec![chord('f', false, false)])],
            ),
            (
                "reports.connections",
                "Connection reports",
                "Reports",
                "Inspect actual terminal links and wire/cable memberships, including unconnected drafts",
                Handler::ConnectionReport,
                Scope::Document,
                ActionRepeat::Ignore,
                vec![],
            ),
            (
                "drawing.selection.rotate",
                "Rotate drawing selection",
                "Drawing",
                "Request an exact quarter-turn and document pivot; keep connected paths whole",
                Handler::RotateSelection,
                Scope::Canvas,
                ActionRepeat::Ignore,
                vec![binding(Scope::Canvas, vec![chord('r', false, false)])],
            ),
            (
                "drawing.selection.mirror",
                "Mirror drawing selection",
                "Drawing",
                "Reflect selected geometry across an explicit pivot axis; preserve electrical A/B order",
                Handler::MirrorSelection,
                Scope::Canvas,
                ActionRepeat::Ignore,
                vec![],
            ),
            (
                "drawing.selection.move",
                "Move drawing selection",
                "Drawing",
                "Translate selected drawing origins while preserving connected routes",
                Handler::MoveSelection,
                Scope::Canvas,
                ActionRepeat::Ignore,
                vec![binding(Scope::Canvas, vec![chord('m', false, false)])],
            ),
            (
                "drawing.selection.align",
                "Align drawing origins",
                "Drawing",
                "Align selected drawing origins to an explicit numeric coordinate",
                Handler::AlignSelection,
                Scope::Canvas,
                ActionRepeat::Ignore,
                vec![],
            ),
            (
                "drawing.selection.distribute",
                "Distribute drawing origins",
                "Drawing",
                "Distribute selected origins between fixed extrema with stable identity ties",
                Handler::DistributeSelection,
                Scope::Canvas,
                ActionRepeat::Ignore,
                vec![],
            ),
            (
                "drawing.route.create",
                "Route conductor",
                "Drawing",
                "Request anchors and a whole path for an existing wire conductor",
                Handler::CreateRoute,
                Scope::Canvas,
                ActionRepeat::Ignore,
                vec![binding(Scope::Canvas, vec![chord('w', false, false)])],
            ),
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
                    defaults: if defaults.is_empty() {
                        DefaultBindings::Unbound
                    } else {
                        DefaultBindings::Suggested(defaults)
                    },
                })
                .expect("unique valid builtins");
        }
        registry
    }
}
