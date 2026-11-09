# 0048 — Shared preview queue and click placement

Status: implemented as the first shared drawing-tool integration slice.

## Decision

Use a small frontend `PreviewQueue` for replaceable pointer previews in template gestures and schematic annotation placement. It serializes requests, coalesces unsent proposals, suppresses obsolete replies/errors, invalidates old sessions on reset, and lets a finish operation await the latest proposal. Transport input IDs are allocated when a template request is actually sent, preserving the common monotonic stream even when previews are replaced.

This module owns scheduling only. Template adapters retain captured variant/target/revision and closed Rust preview tokens. Schematic annotations retain source-view coordinates, snap options, inherited appearance and their Rust session/token. The queue does not calculate geometry, resolve snapping, allocate document identities, or commit commands. Polyline vertex intents remain ordered and are not replaceable previews; do not route Add/Back/Finish intents through this latest-only queue.

Template lines, rectangles and ellipses accept a start click followed by a finish click, showing a fixed-screen-size anchor marker and hovering preview. Existing press-drag-release placement remains available: a first release displaced at least four screen pixels selects the drag workflow; a shorter first release establishes the click anchor. Once anchored, the second release supplies the final coordinates even when older previews are still loading. Whole-artwork movement and template text-box placement retain their existing drag workflows. Escape and pointer capture loss cancel without an undo entry; an invalid final point cannot fall back to an earlier valid preview.

Schematic annotation finish now waits for pending previews rather than ignoring a click or Ctrl/Cmd+Enter while a preview is loading. Input freezes while finishing; invalid final input restores editing and creates no document object. Cancellation before finish invalidates delayed frontend responses while the existing Rust cancellation action releases the draft.

## Boundaries and revisit conditions

No document schema, bridge contract or Rust geometry changes. Template/project histories, pin adoption and library publication are unchanged. This is not yet a general tool framework: inline template text/variable editing, successive-vertex template polylines, circle tools, shared appearance controls, images and borders remain separate slices. Extract shared point-collection state only when another working adapter needs the same semantics; retain ordered vertex intents for graphical paths.

## Verification

Real Rust-backed browser regressions cover reverse start/finish coordinates, every supported click shape, delayed intermediate previews, invalid finishes, exact undo/redo identity, anchor cleanup and cancellation retaining redo. Schematic tests cover delayed numeric/text completion, delayed canvas-click completion, invalid text and cancellation. Existing drag, table preview, snapping and multiline tests remain in the full local UI gate. Native host checks compile/lint; Windows/Linux runtime and native interaction testing remain separate gates.
