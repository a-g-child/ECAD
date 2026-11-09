# 0060 — Cell text layout and justification

Status: accepted. Extends 0055/0056 with optional caption/value block layout.

## Behaviour

A text cell can place its caption above the content or inline with it. Inline flow
uses an exact editable separator, defaulting to `: `. Insert the separator only
when both caption and resolved content are nonempty. Empty separators are explicit
values, distinct from Default. Preserve spaces, literal/variable runs, caption and
content fonts/colours and explicit multiline text. Separators are single-line
literals of at most 64 Unicode characters; they do not introduce variable bindings.

Expose left/centre/right and top/middle/bottom justification for the whole block.
Above flow places the caption above the content with a physical gap. Inline flow
shares a baseline between the last caption line and first content line, measuring
the actual caption and separator advances. Subsequent content lines hang from the
content start. Keep independent caption/content fonts, sizes, weight, slant and
colour; font changes remeasure the block. Empty caption/content does not leave a
separator or a reserved empty caption strip.

The shared frame renderer positions the measured block within a 1 mm cell inset,
including semantic extents for explicit blank lines. It clips overflow and retains
the existing diagnostic/full-value tooltip; it never shrinks or rewrites content.
Automatic wrapping, full paragraph justification by expanded word spacing and
print/native font parity remain future work.

## Properties, inheritance and history

`TemplateCellTextLayout` contains optional caption position, separator, horizontal
and vertical values. Resolve each cell property over template defaults; factories
are above, `: `, left and top. Existing content-channel alignment supplies the
legacy fallback when no whole-block override is set. A new Cell text layout section
works for single text cells, ranges, whole tables and template defaults. Mixed
values stay mixed; each change affects only that property. Default clears its
explicit override. Picture cells remain inapplicable to these text controls.

Existing above-mode channel horizontal alignment can align paragraphs inside the
block. Font-palette vertical controls are hidden when the block layout governs
vertical placement; inline mode also hides channel horizontal controls, because
its caption and content form one flow. Stored font/channel values remain intact.
The same capability-aware palette continues to serve other text owners.

Use the existing complete template property command, captured targets/revisions
and independent template undo. Split copies text layout with appearance, merge
keeps the primary cell's layout and row/column edits preserve retained styles.
Justification and separators never change electrical objects or variable bindings.

## Persistence and compatibility

Nested template v7 stores optional layout defaults/overrides. Old versions reject
layout fields. Bodies without layout intent retain their prior canonical encoding,
including PNG/JPEG/SVG asset tables; immutable published bodies are not rewritten.
The outer project schema is unchanged because template revisions already travel
in the project bundle. Domain, persistence and bridge layout types remain separate.

When no resolved block-layout property is set, render through the existing caption
strip/channel alignment path. This preserves historical layouts until an explicit
block-layout choice is made. Both real sheet projections and template placeholder
previews resolve the new properties and use the same rendering implementation.
Regression coverage includes inheritance/explicit empty separators, invalid and
stale proposals, coherent range undo, old/new file round trips, actual sheet
resolution, mixed-font geometry, blank lines, overflow and template publication.


## Inspector presentation follow-up

Use the shared `PropertySections` UI control for compact collapsible Text, Layout
and Style groups. Show picture fields only on picture cells; conversion from text
uses the existing Picture ribbon action. Expansion and scroll are transient
inspector state, retained through same-target property completion without entering
undo or persistence. Sections start expanded and remain explicitly collapsible.
Numeric/text validation, pending drafts, keyboard completion and exact proposals
remain owned by the existing template editor.
