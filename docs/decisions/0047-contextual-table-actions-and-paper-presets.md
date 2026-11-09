# 0047 — Contextual table actions and exact paper presets

Status: accepted and implemented for the bounded template editor.

## Decision

Carry a typed template selection in captured action context, separate from electrical selection and from document persistence. The headless action registry computes Split/Merge availability against the captured template revision. A read-only bridge query supplies that same eligibility to ribbon and palette controls; stale replies cannot enable a newer selection. Dispatch revalidates eligibility, and structural proposals must match their captured variant/table/cells. Pending table fields retain their original selection. Keyboard sequences include template selection in target equivalence.

Split accepts one cell or a contiguous vertical stack with matching horizontal extents. Each source cell is divided by the supplied row/column counts, retaining its own original upper-left identity and contents. Fresh cells are blank; IDs are allocated top-to-bottom, then row-major. The complete replacement validates before one template-history edit. Interactive requests are bounded to 4096 resulting subdivisions across the selection.

Merge requires a rectangular cell union. The survivor is the first selected cell in spatial reading order, independent of click order. The dialog has no table/cell/survivor selectors. It asks about content handling only when multiple cells contain captions or text runs; otherwise Combine preserves the existing content automatically. The final Apply remains a review step, including when no content decision is needed. Existing domain partition checks and exact history remain authoritative.

Paper presets are exact dimension shortcuts, not new persisted paper identities or claims of standards-compliant margins. The inspector offers ISO A0–A5 and US Letter/Legal/Tabloid/ANSI C–E, orientation and general margin. Advanced exposes exact custom dimensions, individual margins, content bounds and the existing fit-within-margins option. Size/orientation changes can translate tables to fit and clip the content region through the existing validated command; they never scale cells or artwork. An impossible fit or out-of-paper artwork is rejected with the staged values retained. Custom dimensions survive undo, publication and reopen unchanged.

## Boundaries and revisit conditions

No project/template persistence version changes. Publication is still explicit, and project pins do not adopt new revisions automatically. Numeric resize/boundary/insertion dialogs and the artwork catalog remain transitional. Revisit the query projection when more contextual actions need eligibility, preserving a single Rust policy rather than duplicating geometry rules in TypeScript. Shared schematic tools, portable images, independent borders and verified margin profiles remain separate work.

The template editor coordinator is just over the 500-line review threshold. Field completion, eligibility requests, paper controls and structural dialogs remain separate cohesive modules; extract selection coordination if another interaction owner is added. No empty framework is introduced.

## Verification

Domain regression covers order-independent column splitting, gaps, horizontal mismatches, duplicates and failure in a later cell. Bridge regressions cover eligibility, mismatched/stale targets, bounded edits and one-step undo/redo. Browser workflows cover selection-driven dialogs, ribbon/palette agreement, column history, standard/custom paper dimensions, unchanged cell sizes, and saved-library reopen. Routine full/UI/native-host checks remain local; native Windows/Linux execution is a separate gate.
