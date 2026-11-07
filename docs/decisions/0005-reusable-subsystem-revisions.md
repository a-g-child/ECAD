# 0005: Immutable reusable revisions and independent occurrences

Status: accepted for the headless reuse slice.

## Decision

Keep definition lineage (`DefinitionId`), immutable revision (`DefinitionRevisionId`), and placed occurrence (`SubsystemId`) distinct. A revision contains one validated, self-contained local hierarchy. Nested reusable dependencies are rejected until dependency resolution and expansion budgets are designed.

Bundle source bodies in the project for offline reopening. Importing identical content is a no-op; changing content under an existing lineage/revision key is an error. Importing a newer revision never updates existing occurrences.

Placement takes a complete typed mapping from definition-local IDs to fresh occurrence IDs, including terminals, ports, conductors, and cables. Materialize independent electrical objects and bind their root to the source. Use one compound application transaction, with exact rollback and undo. Share immutable revision bodies through `Arc`; do not share mutable electrical state between occurrences.

Pinned internals are read-only. Root name and parent are occurrence parameters; external wiring must use root-exposed terminals. Maintain an ownership index for mutation guards. Every new electrical mutator must honor these guards.

Make-local preserves the original source and ID correspondence as historical ancestry while unlocking objects. Deleted or changed internals do not erase that baseline. Forgetting ancestry is separate and undoable. A source cannot be removed while either a pinned instance or local origin references it. Reversible state transitions distinguish pinned, traced-local, and unlinked content: undoing placement must not leave ancestry behind, while undoing make-local restores the original pin.

## Persistence and limits

Schema v4 stores each bundled body once, plus occurrence mappings and local origins. Bodies use the strict v3 electrical DTO, preventing recursive catalogs. Mapping pairs are explicit arrays so duplicate local keys are detected rather than overwritten by map decoding. Validate sources, mappings, electrical content, and ownership before accepting a loaded project. V2/v3 inputs preserve electrical IDs and upgrade on explicit save; retain legacy fixtures.

Count terminals toward a provisional 100,000-object placement guard; callers may request a lower limit. This is a per-occurrence limit, not a project memory or history budget. Placement and complete validation currently perform broad scans and compare materialized content to its source. Benchmark 1/10/100 repeated instances alongside existing hierarchy probes before changing indexing or representation. These measurements do not establish release capacity.

## Deferred work and revisit conditions

External publication, recommended/default pointers, known-use discovery, arbitrary-subtree capture, compatible revision adoption, and nested reusable dependencies remain separate slices. Importing a source is not publication or adoption. Design those operations around explicit review, external-interface conflicts, local ancestry, atomic undo, and independently pinned consuming projects.

Revisit materialization and broad validation when representative workloads show unacceptable memory or placement/validation cost. Add affected-region indexes or lazy projections without weakening identity independence, offline resolution, or complete validation. Revisit limits with measured memory, cancellation, and project-wide resource budgets.

## Verification

Application contracts cover independent occurrences, exact undo/redo, failed placement rollback, ownership guards, immutable conflicts, local ancestry, and explicit forgetting. Adapter contracts cover canonical v4 persistence, malformed mappings/catalogs, offline reopening, and two independently pinned projects. The reusable example exercises placement, save/reopen, make-local, and undo; the scale example records repeated-instance correctness and timing.
