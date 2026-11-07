# 0024: Fixed File, Drawing and Library ribbons

Status: implemented as a desktop workspace surface. File/drawing commands use existing actions; library/Git management controls remain explicitly planned and disabled.

## Decision

Replace the flat tool row with three compact ribbons: File, Drawing and Library. Drawing is selected initially. Group File commands into Project, History, Outputs, Commands and a planned Version control area. Group Drawing into Selection, Create, Transform, Arrange and View/outputs. Reserve Library groups for Definitions (symbols, wires/cables, parts) and Reuse (shared blocks, revisions).

Keep Open/Save/Undo/Redo in a small always-visible quick-access area beside the application brand. Existing menus, command search and keyboard workflows remain available. Visible ribbon commands reference the same action IDs and share availability, input identity and dispatcher paths; no ribbon-specific editing implementation is introduced. A command may appear in quick access, a menu and its ribbon while retaining one action identity.

An explicit drawing action from a menu/ribbon/palette targets the canvas while retaining its captured selection, view and revision. This matters when the invocation originates from a ribbon tab or command search with workspace focus. Keyboard routing keeps its normal focus/IME/control guards; tab navigation does not cause drawing shortcuts. Existing backend availability and stale-context checks remain authoritative.

Reserve one 96-pixel ribbon area using a shared layout token. Tabs and every panel keep the same height; groups do not wrap into extra rows. Local horizontal overflow stays inside the panel. Switching tabs preserves canvas bounds, viewport, selection, inspector values, active placement fields and ghost geometry. It neither cancels nor commits a draft and never changes document history or project files. Ribbon selection is transient per-window state, initially Drawing; saved workspace preferences remain future work.

Use tablist/tab/tabpanel relationships and roving focus. ArrowLeft/Right wrap through the three tabs; Home/End select the first/last. Buttons retain their normal Tab/Enter behaviour. Captions and group labels make commands readable alongside the existing SVG icons, with common button/focus styling.

Library and Git placeholders are disabled presentation entries with planned labels/tooltips, without fabricated action IDs or click-through workflows. They express the requested ribbon organisation while their actual management UI is still pending. Drawing wire/alignment/distribution actions remain disabled through existing action metadata until their UI is integrated. Register future eligible commands when their working workflows are added.

## Evidence and revisit conditions

The full local suite remains 278 Rust behaviour tests and two doctests. Twenty-one Chromium workflows include fixed bounds/viewport/selection across tabs, keyboard navigation, planned library controls, 960-pixel resizing, unchanged project bytes, real File save/undo/redo, active draft preservation, drawing targeting and palette invocation originating from File. All prior creation/property/geometry/report workflows remain in the same suite.

The native macOS host is built/launched separately. Native accessibility, physical keyboard behaviour and Windows/Linux validation remain pending. Ribbon customisation/collapse, persistent workspace profiles, comprehensive native menu integration and operational library/Git editors remain future scope. Revisit grouping and density as working commands are added; retain fixed pane bounds and shared registry/dispatcher semantics.
