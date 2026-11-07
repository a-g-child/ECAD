# 0006: Explicit revision adoption with stable occurrence identities

Status: accepted for exact pinned occurrences.

## Decision

Provide a deterministic `RevisionChanges` report over source bodies, with typed added/removed/modified local IDs for blocks, devices, terminals, ports, conductors, and cables. Device changes include their terminal lists; terminal changes also include owning device identity. Root source-name changes are reported, although the placed name remains an occurrence parameter. The report can be computed independently of adoption validation, including for incompatible targets.

`adoption_plan` takes a selected pinned root, a bundled target revision, complete target ID mappings, and an object limit. Preserve placed IDs for surviving definition-local IDs automatically; use caller-supplied fresh IDs for additions. Match by stable ID, never designation, terminal number, name, or array position. Reject recycling a removed placed ID for another local object. Keep the placed root, its name, and its parent.

Initially require the same definition lineage, the same definition-local root, and unchanged device ownership of surviving terminals. Changing a terminal's display number is allowed; moving its identity to a different device requires a future explicit migration policy. Local variants are rejected until a deliberate merge workflow exists. Importing or publishing a revision does not adopt it in any project.

## Atomic validation and history

Validate a temporary candidate using the same core path for preview and commit. Remove only objects owned by the previous pin, insert the target materialization through existing invariant checks, bind the new revision, then validate the whole project. Preserve external conductors, cable assemblies, and outer interfaces exactly. Missing anchors, lost exposures, endpoint incompatibilities, and collisions return existing typed model errors and discard the candidate. Users must explicitly resolve external wiring and request a new plan; adoption never silently disconnects or remaps it.

Commit checks the expected binding and resulting connections again, so external changes after preview cannot bypass validation. Submit plans against the session revision used to observe the project. A plan is neither authorization to ignore a newer session revision nor an automatic update subscription.

One `AdoptInstance` command stores previous/target bindings; undo reconstructs the old immutable source with its prior mapping. No full-project snapshot is retained in history. Multiple selected occurrences can be adopted as one existing compound transaction, with exact rollback if a later command fails. Source revisions stay bundled while history may need them through normal inverse ordering. Same-binding adoption is a no-op and preserves redo/checkpoints.

## Cost and boundaries

Candidate validation clones the current materialized project temporarily and validates all pins. This deliberately favors correctness for an infrequent structural edit; preview and commit each pay that cost. A multi-occurrence batch currently repeats candidate work per occurrence. Ownership guards for ordinary wire edits are unchanged. Measure one preview/commit/undo at 1/10/100 instances in the release probe; single samples are exploratory, not performance gates. Revisit with representative memory and large-batch measurements before introducing incremental candidate updates.

No persistence version change is required: v4 already stores immutable bodies and pin mappings. External publication, recommended/default revision pointers, known-use discovery, interactive conflict resolution, nested reusable dependencies, cross-lineage migration, and local-variant merging remain future work.

## Verification

Local integration/regression contracts cover source reports, additions/removals, stable connections and root parameters, independent projects and occurrences, offline save/reopen, compound adoption, exact undo/redo, saved checkpoints, no-ops, stale plans, commit revalidation, failure rollback, limits, collisions, and incompatible identity changes. A runnable adoption example demonstrates review, selective adoption, unchanged external wiring, offline reopening, and exact undo.
