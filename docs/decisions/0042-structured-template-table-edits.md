# 0042: Structured title-block table edits

Status: accepted and implemented for exact numeric editing.

## Partition and identity rules

Table edits return a complete validated `TemplateTable`; the source is immutable. The bridge then validates the owning layout and dispatches one closed variant replacement through the independent template history. Electrical/project geometry, saved sheet pins and published bodies remain unchanged. Undo restores exact IDs, order, geometry and unresolved contents rather than applying an inverse resize.

Split replaces one cell with a row-major grid. The upper-left cell retains the original identity, caption and content runs; every other cell is blank with a fresh OS-random identity, checked against the complete variant and other new IDs. Integer boundaries use floor ratios to distribute remainders without losing outer extent. Each interactive split is bounded to 2–4096 cells before allocation; this is a request bound, not a document capacity claim. Repeated per-row splits support unequal subdivisions. Missing targets, zero counts, collapsed geometry and identity collisions reject atomically.

Merge requires distinct existing cells whose union fills one rectangle. The user explicitly selects a surviving identity and either combines contents or keeps only that cell's contents. Combine is the default: nonempty captions and run sequences join with literal newlines in spatial reading order (Y, then X), independently of presentation order. Typed variable bindings stay typed. The survivor keeps its presentation slot; removed identities are restored on undo. Future custom sheet-entry adoption must define value mapping before these edits can adopt new revisions into existing drawings.

Whole-table position/size editing maps all local edge coordinates through exact integer floor ratios; identical edges map identically. Collapsed cells, overflowing geometry or paper-margin violations reject. Aligned-boundary editing moves every endpoint at one interior X/Y coordinate together; cells spanning the boundary remain intact. Distinct boundary coordinates cannot cross or collapse, including in other rows. This conservative rule is explicit; localized span-aware drags will need a separate proposal contract.

## Application and UI

Resize, Split, Merge and Move boundary have centrally registered unbound actions for ribbon/palette/keymap configuration. They open a captured review dialog for a stable variant/table. Numeric values and IDs cross the bridge as strings through strict DTOs distinct from persistence. Operation and registered action must agree. Apply creates one history entry; no-op edits retain redo. Invalid proposals retain dialog fields and history. Cancel releases transient pending state without changing the draft.

Pending inspector and table-dialog fields have distinct ownership. A pending table operation can commit only the matching closed table proposal, or be explicitly reverted/discarded. Pending fields participate in native dirty guards. Local controls remain transient and have no crash-recovery promise. The modal protects global shortcuts and freezes table selection once fields are edited; the common monotonic input stream and captured editor revision reject replay/stale delivery.

Blank cells can receive and remove ordered literal/known-variable runs through the multiline property editor. These remain local property proposals until Apply. The shared shell/ribbon bounds and scrolling controls are retained.

## Remaining work and verification

Drag handles, table insertion/removal, independent row/column insertion, standard paper/profile presets, anchoring, reusable blocks, shared formatting, field-value adoption and large-table rendering remain planned. The current complete-partition validator checks overlap quadratically; there is no new scale claim. Probe measured larger workloads before raising interactive split bounds or promising large-table performance.

Domain regressions exercise odd remainders, unequal rows, spans, reading-order content policies, invalid unions/IDs and extreme integer geometry. Real bridge tests cover exact history/publication, stale/mismatched/unknown proposals, pending ownership and unchanged project state. Browser workflows exercise dialogs, error retention, blank-cell text/bindings, cancellation/redo, saved publication and minimum-window bounds. Native build/lint checks do not constitute native interaction automation or Linux/Windows execution.
