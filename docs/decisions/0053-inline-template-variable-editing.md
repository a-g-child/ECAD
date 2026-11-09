# 0053 — Inline template variable editing

Status: implemented for template cells, free artwork text and new canvas text placement.

## Decision

Replace stacked literal/variable run controls with one multiline textarea. Display typed variables as `{{project.title}}` placeholders alongside ordinary text. **Insert variable** and right-click on the text field open the same grouped disclosure tree: company, engineer, project, document and sheet. Selecting a field replaces the captured text selection, or inserts at the caret. Cancelling restores the selection and focus without changing the text or cancelling an enclosing drawing tool. Existing keys continue to own multiline editing, composition and field completion.

The UI notation is not a new persisted format. A small codec maps to existing typed literal/field runs. Complete unknown tokens remain typed fields and fail core validation; they never silently resolve or turn into literal text. Incomplete tokens remain literal text. Escape a literal opening delimiter as `\{{`; backslashes are displayed escaped as `\\`. Consecutive literal runs are combined before encoding so a delimiter split across old run boundaries cannot accidentally become a variable. Merely opening the editor, or returning to its original text, preserves exact original run segmentation and empty runs.

Each editor retains its existing transaction owner: contextual template fields finish via captured-target proposals; new canvas text remains transient until one creation commit. The picker is part of the field focus group. Moving into it does not prematurely commit pending text. Picker keyboard events do not reach field-completion or placement cancellation handlers. All displayed content uses text/value assignment, not HTML parsing.

Use the browser's undoable `insertText` editing operation for variable insertion into the native textarea. This deliberately isolates the legacy `execCommand` API: `setRangeText` does not provide the same native undo history. If insertion is unavailable, show the exact token the user can type and preserve existing text. Revisit this adapter when supported webviews provide a replacement with equivalent native history/composition behavior. Tests verify local undo/redo on Chromium; native webview interaction remains a platform validation item.

## Boundaries

The picker exposes the existing typed field catalog; no new field scopes, arbitrary formulas, project metadata writes or persistence migrations are introduced. Placeholders remain editable text rather than rich-text chips. Plain-text clipboard syntax carries symbolic tokens, not resolved project values. Unknown complete variables require correction or explicit literal escaping. Schematic annotations remain literal until their domain supports typed bindings.

## Verification

Real browser workflows cover button/right-click opening, caret and selection insertion, picker cancellation, local undo/redo, IME protection, exact proposal runs, unknown-binding rejection, coherent creation history, template table edits and save/reopen. Codec checks cover literal placeholder lookalikes, backslashes, adjacent literal boundaries, partial tokens and preservation of unchanged legacy runs. Existing core binding validation and persistence regressions remain unchanged.
