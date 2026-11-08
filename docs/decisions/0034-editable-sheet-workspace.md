# 0034: Edit project drawings directly on template-backed sheets

Status: implemented in the bounded single-project desktop prototype. Multi-project host integration, page explorers and tabs remain separate work.

The desktop's normal drawing surface is now a sheet. Remove the Model view/read-only paper switch and the free source-view selector. Select sheets by stable identity; blank pages display their frame and disable source-geometry tools until an existing drawing view is attached. Electrical inspection remains available independently. Library/definition editors are still future, distinct workspace kinds; this change does not turn them into project sheets.

## Coordinates and navigation

Keep source drawing coordinates authoritative. SVG source objects, hit regions, marquee geometry, tool previews and snap queries continue using their existing source-unit contract. Render only the paper/frame through the inverse placement transform:

source = view_origin + (paper - paper_origin) × denominator / numerator.

Validate the display transform before installing it. Blank sheets use identity paper coordinates. Fit uses transformed paper bounds with a physical paper-space pad, including blank sheets, rather than fitting source glyphs. Keep editable source content outside paper visible for recovery; this is not output clipping/preflight. The separate numeric sheet-review preview remains clipped to paper. Display uses floating-point millimetres; exact numeric editor values, IDs and revisions remain strings and committed coordinates remain integer micrometres. Unsafe pointer coordinates continue to be rejected by the input adapter rather than being rounded into supposedly exact edits.

Wheel/trackpad navigation, directional marquee, direct group drag, snapping/cycling/suppression, numeric placement and existing annotation/path/layout tools all use the same editable SVG. Frame cells carry no electrical/drawing object identity and cannot be mistaken for schematic objects. Paper/template edits change presentation, never source geometry or connectivity.

Retain per-sheet cameras during the open project session, keyed by stable sheet ID and placement/paper-layout signature. Rename, metadata and drawing-style updates do not fit the viewport. Changed placements/paper refit; removed-sheet cameras are discarded. Selection clears on sheet switches and source-view changes cannot retain editable hidden-view selection. Per-tab retained selections and persisted workspace cameras are future work.

## Captured targets and legacy projects

ActionContext carries optional captured sheet identity alongside source view and revision. Registry availability rejects missing sheets or mismatched sheet/view pairs. The bridge validates the same relationship for inspection and property requests. Key sequences include sheet identity in their captured target, even when two sheets share a source view. View-only contexts remain supported for headless/definition-editor callers. Blank-sheet Fit uses a valid captured sheet through the ordinary registry/router/dispatcher.

Block sheet switching during a tool, an in-flight action or an active pointer gesture. Proposals keep their original revision and closed source geometry. The sheet's source is never changed by choosing a different tab/menu target while a tool is running. These are bounded single-project input guards, not a multi-project routing implementation.

Legacy unplaced views remain loaded and clean. With no sheets, display an empty state and the registered Sheets and paper action. Users select an existing view, review its sheet/placement/frame and explicitly apply one normal undoable SheetCommand. Cancelling changes nothing; remove-sheet undo retains the source. No automatic fresh IDs, renumbering, geometry copies or dirty migrations occur on load. Bulk wrapping/new source-view creation are future UI features. The built-in demonstration is constructed with its own initial DEMO sheet before creating the session, so it begins clean with no artificial undo entry.

Several sheets may intentionally display one source view. Show a visible shared-drawing warning and keep edits reflected on all placements. No materialization/copy policy is inferred; source electrical/report identities remain singular.

## Verification and remaining boundaries

Core/bridge contracts cover startup, unchanged legacy loading and reviewed attachment/undo, captured-sheet mismatch rejection, blank-sheet Fit availability and sequence cancellation between sheets sharing a view. Browser workflows exercise normal tools on sheets, scaled pointer edits with independent paper/screen mapping, snapping in source units, exact numeric input, connectivity/whole-route undo, paper-only edits, shared views, blank-page Fit and reviewed attachment/cancel/undo/save/reload.

Local macOS/aarch64 verification: cargo xtask full (329 behavior tests and two doctests), cargo xtask ui (34 browser workflows), cargo xtask desktop and embedded-asset native debug build/launch. Browser workflows are not native interaction automation. Native Windows/Linux execution, precision at extreme display magnitudes, performance with many open sheets, output/font parity, automatic wrapping and project-scoped tabs/choosers remain future work.
