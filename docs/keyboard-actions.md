# Keyboard and action model

Status: a headless action registry, shared invocation dispatcher, logical-key router, and canonical keymap schema v1 are implemented. The first desktop prototype routes supported keyboard/menu/tool/palette actions through this core and protects typing. Comprehensive native accelerators, shortcut editor, and the broader interaction requirements below remain planned. See [decision 0011](decisions/0011-action-registry-and-keymaps.md).

Ribbon tabs use ArrowLeft/Right and Home/End with native control focus. File/Drawing ribbon commands, menus and palette share dispatch; explicit drawing actions target the canvas with captured revision/view/selection retained. Tab changes are transient workspace presentation and do not enter project history. See [fixed ribbons](decisions/0024-fixed-tool-ribbons.md).

## Implemented first slice

| Action | Shipped shortcut | Scope |
| --- | --- | --- |
| Open project request | Primary+O | Application |
| Fit active drawing | F | Canvas |
| Undo | Primary+Z | Document |
| Redo | Primary+Shift+Z | Document |
| Guarded save | Primary+S | Document |
| Electrical connection reports | Unbound; remappable | Document |
| Properties | E | Canvas/project tree |
| Annotation input/creation | T | Canvas |
| Graphical polyline input/creation | P | Canvas |
| Existing conductor route input/creation | W | Canvas |
| Numeric selection movement | M | Canvas |
| Quarter-turn rotation request | R | Canvas |
| Mirror request | Unbound; remappable | Canvas |
| Origin alignment/distribution | Unbound; remappable | Canvas |
| Drawing fallbacks/creation presets | Primary+K, S | Document |
| Palette request | Primary+Shift+P | Application |
| Cancel placement request | Escape | Active canvas tool |

Primary resolves to Control for Linux/Windows profiles and Meta/Command for macOS. These are explicit initial choices, not native-platform validation. W requests a whole route for an existing electrical conductor; interactive wire creation remains planned. T/P/W/M/R request typed input/placement; they do not create geometry without concrete parameters. Escape cancels a pending key sequence first, then may request tool cancellation on a subsequent press. T now has desktop annotation text/numeric/pointer controls with preview, Apply and Escape cancellation. In all text editors, Enter inserts a line break and Ctrl/Cmd+Enter applies; non-text numeric editors retain Enter to apply. P now places graphical paths: canvas clicks or numeric-field Enter add vertices, canvas Enter/Finish completes accepted vertices, Backspace removes the last draft point, and Escape cancels. W remains a host request awaiting an electrical route tool.

All layout handlers accept generic exact preview/commit parameters, checked against the handler operation, captured selection/view, and original preview revision. Rotation requests require an explicit pivot and quarter-turn; mirror requests require a pivot and axis. Neither infers a selection centre or changes electrical endpoints. The desktop now opens explicit pivot/angle controls with R and exposes mirror through menu/toolbar/palette; Apply/Enter commits the reviewed proposal and Escape cancels. See [decision 0020](decisions/0020-desktop-orthogonal-transforms.md).

The M handler also accepts exact/pointer preview parameters and a closed preview for commit. Its preview result exposes candidate targets and exact geometry without changing history. Commit checks original revision, selection, and view; it never substitutes a fresh revision for the preview. The desktop M tool now provides pointer/numeric previews, reference selection, Tab candidate cycling, Alt suppression, Apply/Enter, and Escape cancellation; see [decision 0018](decisions/0018-desktop-move-proposals.md).

Layout requests retain the complete original selection. Supported origins belong to annotations, graphical polylines, and terminal anchors in the active view. Routes move through anchors; electrical objects have no inferred drawing origin. Unsupported/mixed-view selections disable the actions, and distribution requires three distinct objects. Layout parameters are validated in the core, including overflow and connected-route geometry; the desktop now provides reviewed origin alignment/distribution through Drawing, Edit and command search. Both actions remain unbound by default and remappable. Visual-bounds alignment and edge-gap controls remain planned.

The connection-report action is read-only and requires a current document context, with no selection or drawing view. It returns the actual terminal/wire/cable projection at that revision. It performs no save or export file IO; native report controls and file dialogs remain planned.

The registry exposes stable IDs, descriptions/categories/search terms, requirements, availability reasons, optional checked state, repeat policy, and suggested bindings or an explicit unbound policy. Current built-ins are non-toggle actions. Every registered action can be remapped or explicitly unbound; underlying model functions are not automatically user actions. Preserve one action implementation for menu/toolbar/palette/context-menu/keyboard/native accelerator invocation. Typed requests retain the original revision/selection/view/optional anchor; replies get new input IDs and revalidate before commands execute.

The initial router uses logical characters, ASCII-letter normalization with explicit Shift, exact non-ASCII scalars, supported named keys, and F1–F24. It rejects overlapping exact/prefix bindings, including across application/document/tool scopes; mutually exclusive canvas/tree/inspector scopes may reuse keys. There is no implicit shadowing precedence or delayed direct-key fallback. User overrides replace an action's full binding list, including all contexts; an empty list unbinds. Unknown action IDs survive import/export and appear as compile diagnostics. Compilation failure never yields a partially installable effective map.

All application shortcuts are suppressed during text/numeric editing, modal UI, composition, or control-owned input, including Save/palette shortcuts in this conservative first policy. Hosts should leave protected input to the control's normal behavior. Document undo/redo are unavailable while protected input owns undo even for explicit menu/toolbar invocation. Sequences preserve their first anchor, cancel on target/focus/revision/tool changes, and do not reinterpret a mismatched continuation as a fresh drawing shortcut. Caller-supplied monotonic time drives timeout/deadline feedback without blocking; keymap replacement clears pending sequences.

Hosts must share one monotonic input-ID stream across native/web paths, reuse an ID for duplicate delivery, and flag repeats on native invocations too. Both router and dispatcher reject replay/out-of-order IDs; attempted failures consume IDs. Native accelerator callbacks must honor the same router decision for that input, including pending/failed sequences, rather than bypass it as an explicit menu click. This is a tested core protocol, not proof that a future native host normalizes/deduplicates its events correctly. Palette and cancellation requests can be handled without a document; malformed parameters are rejected before document access. No input ID is persisted into project/keymap data.

Preference-file lifecycle/editor, physical-key bindings, OS reserved-key diagnostics, native focus/accelerator tests, renamed-action migrations, live UI synchronization, and actual popup/tool controls remain planned. Keymap encoding is a standalone import/export contract; keymap schema v1 and library schema v1 remain independent of project schema v6.

## Intent and coverage

Make common CAD work fast through direct, sensible keys, while keeping all eligible user operations discoverable and assignable. Plan input dispatch as infrastructure before individual features accumulate their own shortcut handling.

An action is a user operation such as starting a wire, moving a selection, aligning objects, opening Properties, or saving a document. It is not every internal Rust function or every possible assignment of a property value. Detailed property editing remains keyboard-accessible through ordinary controls rather than requiring an action for every field/value combination.

Every eligible action has a registry entry and can be bound. Common operations receive mnemonic defaults where practical; less frequent actions can use modifiers, sequences, or command-palette access. An unbound action is not inaccessible or impossible to map. Record the default binding policy explicitly rather than letting shortcut support depend on which feature author remembered it.

## Central action registry

Each action definition includes:

- A stable, non-localized ID, such as `drawing.wire.start` or `selection.align.left`.
- A user-facing name, category, description, and search terms.
- Context/applicability, enabled state, and a reason when unavailable.
- Invocation requirements, such as selection, cursor anchor, or a parameter dialog.
- Default shortcut candidates per platform and context, or an explicit unbound policy.
- Repeat behavior and any checked/toggle state.

Menus, native menu adapters, toolbars, context menus, shortcut help, and the command palette reference action identities. They must not duplicate their own action implementation. Shortcut displays come from the effective keymap, so a remapped key is reflected throughout the interface.

Distinguish UI actions from document commands. Starting a tool, changing zoom, or opening a dialog may not change the document. A completed wire placement or bulk property edit dispatches a validated application command and creates the appropriate undo record. The registry must not bypass that boundary or become a giant module containing every feature's business logic.

## Bindings and growth

Support single keys, modifier combinations, and multi-key sequences through one routing mechanism. Sequence bindings extend the available space without requiring a unique modifier combination for every future action. They are a supported mechanism, not a requirement to replace direct keys for frequent tools.

Use a declarative, versioned keymap with stable action references and no arbitrary action-count limit. Layer shipped defaults with user overrides and optional imported presets. Show conflicts and unknown action references during import; allow users to resolve them rather than silently dropping entries. Define migration behavior when an action is renamed or retired.

Provide a shortcut editor with search, key capture, assignment, removal, conflict inspection, reset, and import/export. Do not store arbitrary executable code in a keymap. Keep keymaps outside the shared electrical project by default.

## Context, focus, and targets

Contexts may include the document canvas, an active drawing tool, a project tree, an inspector, a text editor, and a modal dialog. Define a deterministic precedence and consumption model. The same key may intentionally mean different things in mutually exclusive contexts; ambiguity in overlapping contexts is a conflict.

Do not fire bare-letter drawing actions while editing a text or numeric field, composing characters through an input method, or interacting with a control that owns the key. Preserve navigation, copy/paste, selection, and native text undo. Application document undo should not steal an active text editor's local undo operation.

Make action targeting explicit. Selection-based operations use the current selection. Cursor-based tools use a well-defined canvas anchor or highlighted candidate. If hover fallback is supported, display the target and use documented precedence rather than unpredictably switching between hovered and selected objects.

Palette or menu invocation must not accidentally change the cursor anchor to the menu's location. Capture the applicable document context before opening transient UI, then revalidate it before execution. If an action requires a location and none is available, enter placement mode or request input rather than inventing a position.

Define Escape consistently for cancelling a preview, ending a tool, and dismissing transient UI. Active-tool shortcuts and temporary modifiers need visible feedback. Exiting a tool restores the appropriate drawing context.

## Conflict handling and event behavior

Distinguish simultaneous chords from ordered sequences. Define sequence timeout/cancellation, show pending sequence feedback, and detect prefix conflicts such as assigning both `W` and `W, A` in the same context. Avoid delaying frequently used single-key tools to accommodate ambiguous sequences; require explicit resolution.

Classify key repeat per action. Holding a pan/nudge key may repeat; holding a placement or delete shortcut must not accidentally generate uncontrolled document edits. Define grouping into undo steps for intentional repeated edits.

Normalize platform modifiers and handle keyboard layouts deliberately. Decide whether character-based mnemonics or physical-key positions apply to each binding category; do not accidentally assume a US keyboard. Make reserved or unavailable OS shortcuts visible and offer alternatives.

Native menu accelerators and webview keyboard handlers must invoke an action once. Centralize event routing, prevent duplicate dispatch, and avoid scattered global listeners. Dialog shortcuts and focus restoration require native-host verification.

## Discoverability and accessibility

Provide a keyboard-accessible command palette and shortcut reference generated from the registry. Search includes actions that are currently disabled, with an explanation of what is required. Actions needing parameters may open an input control rather than having a separate binding for each parameter value.

Keep visible focus, keyboard navigation, and non-pointer access to essential operations. Users should be able to disable or remap bare-character shortcuts when needed and use modifiers or sequences instead. Tooltips, menus, and help reflect effective bindings and platform naming.

## Proposed first workflow

1. With the canvas focused, press `W` to enter wire placement; show the tool and endpoint candidates.
2. Place bends, complete the route as one document command, then cancel/end the tool predictably.
3. Invoke Properties through its assigned action and type a label containing `w`; verify no drawing tool starts.
4. Select several devices and invoke alignment from a key, menu, or command palette; all entry points use the same target and operation.
5. Remap wire placement and verify shortcut help and menus update. Save and reload the keymap.
6. Assign a key sequence to a less frequent action and resolve an intentional conflict without losing other bindings.

`W` is a proposed default; the complete initial keymap remains a design task.

## Verification requirements

- Unit-test context resolution, repeat policy, keymap merging, sequence/prefix conflict detection, and unknown action handling.
- Integration-test that keyboard, menu, and toolbar dispatch reach the same action and document command where applicable.
- Regression-test shortcuts during typing/composition, dialog focus, cursor-anchor retention, duplicate dispatch, and stale selection/context.
- Check registry coverage so eligible menu/tool operations are not accidentally unregistered or impossible to bind. Check default binding conflicts automatically.
- Verify native shortcuts and relevant non-US layouts on Linux, Windows, and macOS. Simulated browser key events alone do not establish native behavior.

## Open decisions

Choose the first keymap, sequence leader and feedback, hover-selection policy, scope precedence, keymap schema, and platform alternatives using representative drawing workflows. Keep interfaces extensible before selecting a shortcut library.

The desktop drawing-defaults editor is available through `drawing.styles.edit` from Drawing, Edit and command search, or Primary+K then S. The modal owns text and Escape input; Apply submits one guarded settings command and returns focus to the canvas. Named-style and creation-preset editing remain future UI work.

Project information uses `document.metadata.edit`, unbound by default and available from File, its menu and command search. The modal owns text input; Enter adds a line, Ctrl/Cmd+Enter applies, and Escape cancels. This edits project-owned values, with no implicit personal-profile lookup.
