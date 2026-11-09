# 0059 — JPEG/SVG and exclusive cell content

Status: accepted. Extends 0058's formats and supersedes its text-over-picture rule.

## Formats and ownership

Keep one shared import/property/rendering workflow for schematics, template artwork
and cells. Detect PNG/JPEG/SVG from bytes, not the selected filename or claimed MIME.
Validate again at persistence and bridge write boundaries. Keep original bytes in
the existing content-addressed copied asset table; SVG remains vector content,
not a rasterized substitute. JPEG display dimensions account for EXIF orientation.

Pure-Rust `jpeg-decoder` (without Rayon), `roxmltree`, and `cssparser` belong only in
the adapter. XML/CSS parsing validates passive SVG, including CSS escapes and local
fragment references, without a network or filesystem resolver. Supported SVG
includes paths, shapes, text, groups, internal references, gradients and filters.
Scripts/events, animation, foreign content, embedded images, external images/fonts/
stylesheets, DTDs and processing instructions are rejected rather than stripped.
This is a supported static subset, not a general browser-document importer.
SVG text uses available renderer fonts; cross-platform font/output parity is not
established. Future broader SVG import should use explicit packaging/sanitization
rules, not silently add external dependencies.

Keep the existing per-image 4 MiB/4,194,304 pixel or intrinsic SVG-size budget and
32 MiB raster decoder allocation bound. SVG also limits XML nodes to 10,000,
ancestry to 65 and nested CSS blocks to 32. These are validation bounds, not a
claim that arbitrary SVG rendering has constant cost.

Project v15 allows JPEG/SVG drawing assets; PNG-only projects remain v14, and
projects without drawing pictures remain v13. Template v6 allows JPEG/SVG; existing
PNG-only v5 and earlier bodies keep their canonical encoding. Older schema versions
reject extended formats. The bridge now carries base64 `data` plus validated `mime`;
legacy `png` request fields still accept PNG. Templates in project bundles use their
own variant version independently of the outer project version.

## Cells and compatibility

New cell content is either caption/value text (including variables) or one picture.
Choosing a picture clears caption and value runs in the same reviewed undoable edit.
Picture cells expose picture controls and **Use text instead**; text fields and
font controls are inapplicable until returning to text. Bulk font controls skip
picture cells. Cell geometry and saved text style preferences survive type changes.

The template application controller rejects new mixed content, including direct
bridge proposals. Default merge keeps the first cell's complete content; Combine
cannot produce picture-plus-text or discard another picture. Split keeps the picture
only in the original cell. Undo restores exact prior content and asset references.

Do not rewrite immutable published bodies or discard user text while loading files
created by 0058. Historical mixed cells remain representable and render unchanged;
the inspector identifies them and offers **Keep picture only** or **Use text instead**.
Unchanged historical content may survive geometry/style edits. Content changes must
resolve the conflict. New documents reject mixed content. This compatibility rule is
explicit in the controller; the retained domain representation is not a tagged union
because that would lose historical mixed state and break exact undo/canonical bodies.

Regression coverage includes all formats in both editors, original-file independence,
version gates, malformed/external SVG input, MIME spoofing, EXIF sizing, coherent
cell switching and legacy content restoration. Native Windows/Linux testing and
advanced transforms/clipboard/drop remain follow-ups.
