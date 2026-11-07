# 0020: Desktop rotation and mirror previews

Status: implemented in the Rust bridge and desktop controls; local browser workflows exercise real document history and files. Native platform interaction automation remains pending.

## Decision

Expose the existing rotation and mirror actions through Edit, toolbar and command search. R opens rotation controls. Rotation offers 90° clockwise, 180°, and 90° anticlockwise in the current downward-y drawing viewport. Mirror offers a vertical or horizontal line through an explicit pivot. These labels correspond to the engine's exact quarter-turn and mirror-axis conventions.

Seed the visible pivot fields from the exact origin of the first selected object, supplied by Rust. The user can then edit X/Y as signed integer micrometre strings. Never infer a selection-centre pivot from approximate scene bounds. Show the pivot marker and complete proposed geometry before Apply/Enter or a canvas click. Escape/Cancel drops the proposal. Annotation glyphs remain upright; only their position transforms.

Retain a closed `LayoutPreview` at the original revision, selection, view and action. A transform lifetime has an opaque identity; each update invalidates its predecessor before parsing, operation-kind checks, overflow or routing validation. Commit consumes the matching held proposal and invokes `CommitLayout` through the original registered rotation/mirror handler and shared input-ID stream. It cannot substitute client geometry, change handlers, or recalculate a new proposal. Starting another tool or opening a document clears the previous lifetime. Normal command/history/save rules remain authoritative.

Move and transform tools share presentation of exact preview records and captured appearance for the selected/touching-route union. The tools retain their different typed application proposals. Split transient request orchestration into a private child of the document controller so controller fields remain private and neither UI dependencies nor generic command deserialization enter the core.

Place both controls inside one permanently reserved 40-pixel shell tool area. Only the active tool's controls are shown, without changing canvas bounds. Lock selection, view choice and other editing operations while reviewing a proposal. Coalesce input updates, ignore replies from old generations, and disable Apply while a proposal is pending or invalid. Keep pivot text editing protected from application shortcuts.

Both-end selected routes transform every retained vertex, preserving bends and electrical A/B identities. Boundary routes adjust only their moved end using existing engine rules. Routes are transformed through selected anchors; arbitrary selected electrical objects and whole routes are not assigned inferred origins. Defaults/style intent and electrical topology remain unchanged.

## Evidence and revisit conditions

Bridge regressions cover complete-route rotation, both reflection axes, exact preview/commit agreement, one-step undo/redo, save/reopen, invalid/overflow pivots, wrong handler operations, stale/replaced/cancelled proposals, replay and pivot values beyond JavaScript's safe integer range. Eleven browser workflows include R/menu/toolbar entry, exact pivot controls, protected invalid input, cancellation, fixed workspace bounds, real saved geometry and undo/redo. Core drawing/persistence schemas remain unchanged.

Native build/launch is checked separately on macOS. Windows/Linux execution, accessibility and physical keyboard/pointer validation remain pending. Pointer-picked/reference-choice pivots, angle cycling shortcuts, arbitrary angles, visual-bounds alignment and font/export parity remain future scope. Shared preview rendering still rebuilds SVG and has no measured large-view latency claim.
