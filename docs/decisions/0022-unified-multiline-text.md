# 0022: Every text value can contain multiple lines

Status: implemented in text editors, annotation/caption rendering, selection/Fit and report cells. Native platform interaction and export font/layout parity remain pending.

## Decision

Keep text as one existing string-valued property and one existing annotation type. Line count is content, never a single-line/multiline object mode. Users can add or remove line breaks during creation or Properties editing, retaining object identity, appearance intent, shared-property applicability and ordinary undo/redo.

Use the same textarea-based editor for annotation text and every editable text property. Enter and Shift+Enter insert line breaks. Ctrl/Cmd+Enter applies; property rows also expose an Apply button. Coordinate, colour and other non-text editors retain their existing Enter behaviour. Escape cancels a creation draft or reverts a property editor. Native text editing and IME composition own their input. Editors have fixed heights and internal scrolling, without content-driven resizing or pane changes. The annotation strip stays at its reserved height with a compact two-line editor; Properties provides three visible lines.

Render explicit SVG tspan baselines for both annotations and derived terminal captions. Treat LF, CRLF and CR as line boundaries for display. Preserve indentation and repeated spaces, Unicode and literal markup. Start the first baseline at the existing annotation position and advance subsequent baselines by 1.3 times the resolved text height. Preserve blank leading, interior and trailing lines through semantic text-block bounds unioned with actual glyph bounds. The original text is exposed as an accessible label. Annotation glyphs remain upright under the existing transform contract.

Window selection encloses the complete annotation block; crossing selection can touch its blank line boxes. Blank annotation regions remain part of the same movable/selectable object. Derived terminal captions remain noninteractive feedback. Fit uses rendered group bounds, including multiline text and captions, rather than document origins alone. Renderer/font measurements can vary slightly with zoom, so browser Fit checks allow one CSS pixel while exact model coordinates remain integer-based.

Report cells display stored line breaks with wrapping inside the existing scrollable report pane. Tree/menu/view-selector summaries stay compact with full text retained in their underlying value or tooltip. No editor or presentation path converts a multiline value to a different document object. The existing schema-v6 strings already preserve line breaks; no migration or new persistent flag is needed. Browser textareas use normal LF editing values; the bridge/persistence preserve literal CRLF/CR values when read or supplied directly.

## Evidence and revisit conditions

The full suite has 275 Rust behaviour tests and two doctests. A new bridge regression verifies mixed literal line endings, blank lines, inspection, same-object editing, undo, saving and reopening. Sixteen Chromium workflows include multiline creation, Enter protection, Ctrl/Cmd+Enter and explicit Apply, switching line counts on the same identity, blank-line selection, rendered Fit, shared designation edits, derived captions, report cells, fixed pane geometry and saved/history round trips.

Line advance is currently one shared renderer policy. Before making leading user-editable or exporting layout, promote it to the portable inherited-style/default contract and test font/layout parity. Rich text, automatic document word wrapping, rotated text, explicit text anchors and native font/export validation remain future work. Native Linux/Windows and physical keyboard validation are still required before release.
