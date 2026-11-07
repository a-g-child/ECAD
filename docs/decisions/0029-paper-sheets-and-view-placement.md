# 0029: Paper sheets and independent drawing-view placement

Status: implemented for one initial ordered drawing set and at most one view placement per sheet.

## Decision

Add stable SheetId and DrawingSheet independently of DrawingView and electrical containment. Sheets have unique nonblank drawing numbers, literal multiline titles, oriented positive paper width/height and nonnegative guide margins leaving a positive interior. Store actual dimensions rather than inconsistent preset/orientation flags. Presets and orientation controls edit those dimensions. The project's initial ordered sheet vector is one implicit drawing set; reordering never changes drawing numbers or identities.

SheetViewPlacement references an existing view, a paper origin, a source-view origin and positive integer scale numerator/denominator. Map paper = paper_origin + (source - view_origin) × numerator/denominator. Coordinates remain signed integer micrometres and ratio terms retain exact intent; scene floats are display-only. Paper edits never rewrite source geometry/styles/electrical records. A sheet may have no view; a view may appear on multiple sheets. Removing paper retains the source view. View deletion/aggregate replacement rejects dangling sheet references. Electrical reusable bodies explicitly reject sheets.

Use SheetCommand for localized insert/replace/remove/reorder inverses. SheetPreview validates a temporary candidate and captures the complete command plus original session revision. All entry points use drawing.sheets.edit. Invalid edits invalidate older previews; cancellation and project replacement discard them; stale or superseded proposals cannot commit. No-op replacement/reorder preserves history, redo and checkpoints. New sheets receive generated UUID-v4 IDs and an available initial drawing number; no sheet is inserted before reviewed Apply.

Strict project-v8 DTOs add an ordered sheets array without changing v7 metadata or electrical/drawing content. V2–v7 load with empty sheets; v1 migration stays explicit. Retain legacy fixtures and v8 canonical outputs. Editor strings stay distinct from persistence integers and domain structs. Rendering applies the validated placement and clips source artwork at paper bounds; dashed margins are guides, not measured printer limits.

The bounded sheet manager exposes all current paper/placement fields, read-only review, explicit reorder/remove and cancellable drafts. Sheet display switches between editable Model view and read-only paper. Paper uses the shared navigation input adapter for zoom/pan, F fits its bounds, source editing actions are disabled and right-click opens sheet properties. Source viewport and selection remain transient; missing displayed sheets fall back to Model view. Multiple named sets, multiple viewports, interactive paper placement and unified bulk sheet properties remain future work.

## Verification and limits

Two domain contracts cover paper/margin/scale/identity rules and view lifetime. Three adapter workflows cover exact preview/commit/save/reopen/history, unchanged source records, no-op redo, ordering/removal, rollback, strict v8 and v2–v7 upgrades, and reusable-body exclusion. Three bridge workflows cover exact string values beyond JS safe integers, seed identities/numbering, invalid/cancelled/stale proposals, blank scenes and shared ordered-set operations. Two real Chromium workflows check presets/orientation, placement mapping, minimum bounds, paper navigation/tool protection, blank/reordered/removed sheets and exact saved history.

Local macOS/aarch64 checks: cargo xtask full (305 behavior tests and two doctests), cargo xtask ui (31 workflows), cargo xtask desktop, embedded-asset debug build and native webview-to-core launch. Native interaction automation and Windows/Linux remain unverified. Title-block tables, bindings/templates, printer/PDF/export, output validation and font parity are separate future slices.
