# 0058 — Shared portable pictures

Status: accepted for the initial slice. [Decision 0059](0059-image-formats-and-exclusive-cells.md) extends formats and supersedes the initial text-over-picture cell rule.

## Decision

Use one immutable `PictureAsset` and `Picture` fit model for schematic placements,
template artwork and table-cell pictures. PNG decoding belongs in the adapter;
the headless core owns validated dimensions, immutable bytes and placements.
The browser and native desktop share browsing/property controls and SVG rendering.
Import is transient; Apply commits through the existing project or template
controller, with captured revisions and exact undo. Cell pictures follow cell
bounds, behind caption/content, without changing their text. Splitting retains
the picture only in the original cell. Default merge keeps the first cell's
picture; combining text rejects nonprimary pictures instead of discarding them.

Copy original static PNG bytes into the owning saved body, never retain a source
pathname. A SHA-256 asset table stores each distinct image once per drawing
picture document or template variant. Placements contain asset references and
fit. Template library publication and project bundling therefore carry their own
assets and remain independent of the importing workstation. Different template
revisions may contain their own copy; this is not repository-wide deduplication.

Initially embed base64 asset tables in existing atomic JSON saves. This deliberately
bounds the implementation without adding multi-file commit/recovery semantics.
It trades binary-friendly Git storage for single-file portability. Project v14
adds schematic pictures; variants containing pictures use nested template v5.
Projects without schematic pictures remain v13; variants without pictures retain
their previous canonical encoding. Old immutable library bodies are not rewritten.
Loading rejects invalid/missing/duplicate/unreferenced assets and unsupported fits.

Limit each image to 4 MiB compressed and 4,194,304 pixels, with a 32 MiB decoder
allocation budget. Limit unique assets per owning drawing document/template variant
to 64 and 16 MiB compressed. These are safety bounds, not measured performance
claims. Scene DTOs currently repeat encoded bytes across placements; the persisted
asset table deduplicates them. Decode/import and persistence tests run offline.

## Scope and review triggers

File browsing, exact editable placement dimensions, initial aspect-ratio sizing,
contain/cover/stretch fit, replacement, removal, translation and coherent history
are implemented in both editors. Table cells use their existing bounds. New free
pictures use a reviewed placement dialog followed by existing drag movement;
click-to-place insertion remains a follow-up. Rotation/mirroring of schematic
pictures is explicitly unavailable.

Revisit packaging when real projects make JSON/Git size or transport latency
material: introduce binary sidecars only with atomic recovery, dependency closure,
portable export and migration tests. Consider an asset-handle transport cache then,
without making that cache authoritative. JPEG/SVG, clipboard/drop, explicit crop
handles, rotation, opacity, stacking/locking, electrical-owner anchoring and
print/export parity require subsequent bounded slices. Native Linux/Windows
runtime behaviour requires validation on those platforms.
