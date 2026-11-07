# 0021: Annotation creation through the desktop bridge

Status: implemented with Rust bridge and Chromium integration tests. Native platform interaction automation and multiline text layout remain pending.

## Decision

Expose the existing T annotation action through the keyboard, Edit menu, toolbar and command search. The active drawing view is required. The reserved tool strip provides a literal single-line text editor, exact integer-micrometre X/Y fields, Apply and Cancel. Opening an empty draft does not create a document object. Enter text, then position it by pointer or exact coordinates. Apply/Enter or a canvas click creates one annotation and exits the tool. Escape/Cancel drops the draft and preserves undo/redo.

Generate the persistent label identity in the host with the existing OS-random UUID-v4 adapter. Return no editable identity field to the frontend. Each draft has a separate opaque lifetime and each valid proposal a token. An update clears the held proposal before parsing its strict inner DTO or validating text, coordinates and geometry. Replaced, invalid, stale or cancelled proposals cannot create an older text/position. Commit consumes only the held typed annotation input and goes through the normal registered T dispatcher on the shared host input-ID stream.

Preview a candidate inserted into a temporary project clone so normal model validation and style resolution apply. Copy the actual document text creation preset, and use the same label scene projection as committed annotations. The original document revision guards commit: the standard T command recreates the retained text/position/identity using that unchanged revision's creation preset. No snap query runs at commit. A style/default edit invalidates the proposal through the same revision guard as any other document change. Tests verify exact scene and saved appearance intent parity, including explicit preset overrides. A future generic creation-preview contract may replace this bridge-local candidate if multiple creation tools benefit from it.

Pointer placement uses the existing view-local terminal/endpoint/midpoint/grid query with a ten-pixel viewport tolerance and 5 mm grid. Tab cycles candidates and Alt temporarily suppresses snapping while the canvas owns input. Numeric input bypasses snapping. Snapping places text only; it creates no electrical attachments or live data binding.

Extend transient SVG rendering to include proposed new rows as well as replacements. Keep baseline hit-test rows authoritative and drop added preview rows on cancellation. Coalesce input requests, ignore old tool generations, and disable Apply until the current proposal is valid and acknowledged. Literal text uses textContent and input values, never HTML.

Select the new annotation after creation for ordinary Properties/move/transform workflows. Filter current selection against every incoming snapshot before inspection, since undo can delete a newly selected object. Native buttons/selects retain keyboard navigation and activation; application hotkeys cannot steal their Tab/Enter input. Text/IME protection and per-tool coordinate entry remain in place. All tool strips share the same reserved workspace height.

## Evidence and revisit conditions

Rust bridge tests cover fresh identities, cancelled/replaced/invalid/stale proposals, malformed placement DTOs, blank text, exact large coordinates, terminal snapping, replay, creation preset/style projection, one-step undo/redo, saved appearance and reopen identity. Thirteen browser workflows cover T/toolbar entry, literal Unicode/markup-like text, fixed bounds, new-object rendering, exact placement, property selection, save/undo/redo/deleted-selection cleanup, snap/cycle/suppression, native button navigation and redo-preserving cancellation.

The project schema remains v6; no runtime draft state is persisted. Full candidate cloning is a bounded prototype approach and needs measurement before large-document creation tools. Multiline text editing/layout, text orientation/font-export parity, live connection labels, repeated stamping, graphical path/wire/symbol creation, and new-view/project controls remain separate work. Native Windows/Linux and physical keyboard/trackpad behaviour require validation before release.
