# 0026: Project drawing defaults in the desktop editor

Status: implemented for project fallbacks. Named-style and creation-preset editors remain planned.

## Decision

Expose the existing drawing.styles.edit action from Drawing, Edit, command search and its existing Primary+K then S sequence. A bounded modal edits project font family, text height, graphical stroke width and output colour. Values stay local until Apply; Escape/Cancel leaves history untouched. Font uses the shared multiline text editor. Lengths are positive u32 integer micrometres; colour accepts #rrggbb. Font requests remain system-dependent rather than guaranteeing installed-font parity.

Keep bridge DTOs distinct from persistence/domain structs. The strict defaults object contains four strings. UpdateDefaults validates the captured revision, parses all four fields, clones current settings and replaces only the base values, then uses the existing registry/dispatcher with DrawingSettings parameters. This preserves every text/stroke/route creation preset and named-style reference. Project schema v6 remains unchanged. Shared monotonic input IDs also protect this new mutation path.

One complete settings command updates inherited properties without rewriting dependent objects. Object, view and named-style overrides retain precedence and exact intent, including values equal to inheritance. Apply is one undo operation; applying unchanged values does not clear redo or move the saved checkpoint. Validation and stale/replayed failures preserve the document. Invalid fields remain editable with their complete local draft; the error area is reserved and scrolls internally. The modal owns keyboard input and leaves shell pane geometry unchanged, including at 960 × 640.

## Evidence and limits

Three new bridge tests cover inherited scene changes, explicit overrides, exact save/reopen/history, strict and atomic invalid input, range limits, stale/replayed input, unchanged Apply and retained nondefault presets/styles. Two browser workflows exercise the real Rust engine, settings controls, rendering, exact saved records, undo/redo, shortcut/palette/ribbon entry points, cancel/text ownership, minimum window size and unchanged Apply with redo.

Local macOS/aarch64 checks: cargo xtask full (285 Rust behavior tests and two doctests), cargo xtask ui (25 Chromium workflows), cargo xtask desktop, embedded-asset debug build and native webview-to-core launch. A screenshot is written to ignored target/ui/drawing-defaults.png. Native dialog interaction automation, Windows/Linux execution and large-document rendering capacity remain unverified.
