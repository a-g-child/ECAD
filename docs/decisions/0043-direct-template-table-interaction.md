# 0043: Direct template table interaction and closed gesture previews

Status: accepted and implemented for the bounded SVG editor.

## Direct controls

Dragging through cell interiors selects an Excel-style rectangular range within one table. Touching a merged span expands the rectangle until it contains whole cells, with deterministic closure over the retained partition. Selection stays transient. The merge review dialog starts with those stable cell IDs checked; the inspector explicitly shows the first cell rather than implying a mixed property edit.

Selected tables expose a move grip, interior boundary hit areas, outer sizing edges and contextual row/column plus buttons. Controls remain screen-sized through zoom. Pointer positions are mapped through the actual SVG transform and quantized once to integer micrometres. Each drag retains its original table, edge and pointer offset; re-rendering cannot retarget it. The camera stays fixed during a gesture. Existing numeric dialogs and centrally registered actions remain available.

Interior edge drags use the existing aligned-boundary rule: every endpoint at the original X/Y coordinate moves together, spanning cells remain intact and distinct coordinates cannot cross. Outer edge drags resize the coherent table; a start-edge drag holds the opposite edge fixed. The move grip translates the entire table without scaling cells. Source-view or electrical geometry never moves.

## Row/column insertion

`TemplateTable::insert_band` inserts a positive band at an existing local boundary. Existing cells after the boundary shift; spanning cells extend; blank cells fill the remaining cross-axis intervals using adjacent divisions. New IDs are allocated outside the domain, checked across the complete variant and retained in the closed preview. Existing IDs, captions, typed runs and presentation order are preserved; blank cells are appended in spatial order. Outer boundaries use the neighbouring strip's divisions.

Contextual plus buttons insert after the selected range using the adjacent selected cell's band size. They keep the table's bottom/right edge fixed and grow toward its top/left, as labelled in their tooltips. Numeric insertion actions also offer start-edge anchoring. Overflow, insufficient anchor space and paper-margin violations reject atomically. Content regions remain explicit; insertion does not resize them or move schematic contents. Table creation/removal and localized span-aware resizing remain future work.

## Closed previews and cancellation

The bridge retains one transient table gesture separately from the document. Every proposal carries its original editor revision and registered action. A target signature includes the variant/table and fixed cell/selection/boundary/insertion identities; later proposals cannot change those targets. Geometry or insertion size may change within that gesture.

A validated proposal returns a preview token and table projection, without editing history. Each newer attempt invalidates the prior proposal, including malformed/invalid input. Commit consumes the exact successful retained body; it does not rerun geometry or allocate new IDs. Superseded, stale, invalid and replayed commits fail without mutation. Gesture state participates in native dirty guards and blocks unrelated document/property actions until it commits or cancels.

The frontend coalesces pointer proposals while one request is in flight. Release waits for the final pointer proposal and commits its token; it never accepts an earlier successful position after a later invalid attempt. Escape/pointer cancellation waits for in-flight work and dispatches the shared placement-cancel action, preserving document checkpoint and redo. Text, composition and modal controls retain their input protections. No new global shortcut listener is installed.

## Verification and bounds

Domain tests cover inside/outside insertions, spans, anchor choices, fresh-ID validation and integer overflow. Real bridge tests verify exact preview IDs on commit/undo/redo, invalidation, target retention, cancellation, stale/replayed requests and unchanged project state. Chromium tests exercise range/merged-span closure, edge/outer sizing, movement, contextual insertion, zoom-sized controls, keyboard activation and delayed final/cancelled previews.

Rendering and hit-testing are bounded prototype behavior, with no large-table capacity or native interaction automation claim. Current quadratic partition validation and complete scene projection remain measured-scale follow-up work. No persistence schema change or automatic adoption into consuming drawings is introduced.
