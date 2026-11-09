# 0049 — Inline template text placement

Status: implemented for new template artwork text.

## Decision

The registered template text tool takes a paper anchor click, then opens a bounded editor beside that point inside the canvas. A click supplies a 60 × 20 mm text box, clipped to the remaining paper extent. Dragging instead supplies explicit box corners. Invalid or collapsed box geometry cancels before editing; no object is created. Existing text-box dimensions remain editable through contextual artwork properties after placement.

The floating editor reuses typed literal/variable run controls. Plain Enter adds newlines; Ctrl/Cmd+Enter or Place validates and commits; Escape or Cancel discards. Composition owns its keys and intermediate input. Multiple runs are edited without changing the authoritative template. The overlay scrolls within the existing canvas, repositions on canvas resize and never changes ribbon or pane dimensions.

Creation proposals add optional typed `contents` to the existing artwork preview DTO. It is accepted only by text creation with valid endpoints. Explicit empty/whitespace-only runs and unknown variables are rejected, invalidating the preceding preview token; a variable-only proposal is valid. Omitted contents retain compatibility with the existing two-point text primitive. Literal text is not parsed for executable markup or variable syntax; variables remain typed unresolved field identities.

The editor submits a final closed proposal and drains the shared preview queue before committing the retained Rust token. Invalid text remains editable. One successful creation adds the bounds and complete contents together as one template undo entry. Armed and editing phases retain the existing native lifecycle guards. No project/library body is updated until the normal explicit publication/import workflows.

## Integration and limits

Shared run controls now notify their owner after updating the run model; this prevents live preview from seeing the previous input value. Cell properties and artwork properties use the same controls and keep their existing transaction boundaries.

This is a canvas-local run editor, not direct editing of rendered glyphs. Appearance controls remain in the contextual inspector; shared font/style controls, auto-sizing, successive-vertex template paths, circles, borders and portable image insertion remain separate work. Text clips/marks overflow using the existing rendering policy; newlines do not silently expand the box or scale text.

No persistence schema version changes: typed text runs already exist in retained artwork and immutable template revisions. The new field affects transient bridge proposals only. Revisit the default box size when creation presets and template artwork style inheritance are integrated.

## Verification

Bridge regression covers literal/multiline/variable proposals, strict malformed/empty contents, stale-token rejection, lifecycle guards, one-step history and cancellation preserving redo. Browser regression covers click placement, live previews, protected multiline/IME input, delayed completion, invalid-text recovery, variable-only drafts, cancellation, bounded viewport resizing, undo/redo identity and library reopen. Existing text-box drag placement and cell/artwork run editing remain in the full local UI gate.
