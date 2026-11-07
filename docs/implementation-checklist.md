# Implementation checklist

Status: headless lifecycle plus project-local hierarchy, interfaces, and cable grouping implemented. Immutable reusable definitions, independent instances, and make-local ancestry are implemented. Explicit pinned-instance adoption and local shared-library publication/recommendations are implemented; explicit registered-file known-use discovery is implemented; durable registries and local-variant merging remain planned. Checked items describe executable behavior or completed checks, not the full intended CAD product.

## 1. Working foundation

- [x] Cargo workspace with separate domain, application, and adapter crates.
- [x] Portable `cargo xtask` runner with offline/locked quick, full, and scale profiles.
- [x] Enforce domain/application dependency boundaries through Cargo metadata.
- [x] Formatting, Clippy, required public rustdoc, doctests, and warning-free documentation checks.
- [x] Typed, caller-supplied device, terminal, and conductor identities; deterministic ordering.
- [x] Generic devices with terminals and individual conductors with optional end attachments.
- [x] Reject duplicate identities, blank required labels, dangling attachments, and deletion of connected devices without partial mutation.
- [x] Revision-guarded commands with localized inverses, undo/redo, redo branching, and configurable history count limits.
- [x] Keep failed edits and no-ops from corrupting history or advancing revisions.
- [x] Separate disk DTOs from domain types; experimental versioned, deterministic JSON with final LF.
- [x] Set repository LF attributes for source and golden JSON fixtures across checkout platforms.
- [x] Reject unsupported versions, unknown fields, invalid references, and malformed documents.
- [x] Save through same-directory staged replacement; save/reopen and failed-replacement integration tests.
- [x] Golden fixture and real temporary Git workflow verify unchanged saves, focused diffs, and branch content restoration.
- [x] Runnable two-device/one-conductor headless example.
- [x] First bounded headless scale probe with correctness checks and JSON results.

## 2. Complete the document lifecycle

- [ ] Select and pin an exact Rust toolchain using a supported rustup channel/version; keep dependency setup reproducible on all target OSes.
- [x] Define fallible OS-random UUID-v4 generation and persistent project lineage identity, distinct from process-local session epochs and file paths.
- [x] Add process-local session epochs so equal revision sequences in separate/reopened sessions cannot accept each other's commands.
- [x] Track saved checkpoints/dirty state through successful/failed saves, undo/redo, divergent branches, no-ops, and history eviction.
- [x] Introduce ordered compound transactions for multi-object edits with rollback on failure and one undo operation.
- [x] Integrate saved-state handling with external changes through a versioned document controller, guarded saves/save-as, and validated explicit reload policies.
- [ ] Define memory/byte budgets for history and test long editing sessions.
- [x] Add content-based external-change detection and explicit document-reload/history invalidation, including a real Git checkout regression.
- [ ] Define cooperative locking/concurrent-writer guarantees and path-alias handling beyond the current optimistic check/replace window.
- [ ] Specify and test recovery/durability, metadata preservation, and multi-file save behavior before adopting a multi-file project format.
- [ ] Set input/resource limits and add malformed-input/failure-injection cases as the format grows.
- [x] Define explicit v1 migration assigning a chosen project ID; retain v1/v2 fixtures and test lossless v2-to-v3 upgrade with electrical identities preserved.

## 3. Hierarchy and cable vertical slice

- [x] Model definition lineage, immutable revision, and placed occurrence as distinct identities.
- [x] Introduce project-local occurrences with dedicated `SubsystemId`, nested containment, cycle rejection, stable iterative paths, and a provisional depth guard.
- [x] Add explicit external interfaces and conductor continuity across every exited subsystem boundary without duplicate terminal objects.
- [x] Add cable assemblies with unique core membership and consistency between overview endpoints and detailed terminations.
- [x] Build the controller/conveyor example with a junction, nested sensor assembly, one cable, and internal wiring.
- [x] Test boundary mappings, reorganization rejection, atomic rollback, save/reopen, undo/redo, canonical loading, and schema compatibility.
- [x] Instantiate immutable project-bundled revisions with complete independent ID mappings, locked internals, and root-interface wiring.
- [x] Make an occurrence local while retaining ancestry; restore pinning and metadata through undo/redo.
- [x] Persist reuse in v4, preserving v2/v3 electrical identities; reject malformed mappings, aliases, and conflicting revision bodies.
- [x] Compare immutable revisions and explicitly adopt selected pinned occurrences with compatible identity preservation, atomic undo, and rejection of invalid external references.
- [x] Test independent two-project adoption, offline reopening, stale plans, structural changes, and exact rollback.
- [ ] Build interactive conflict resolution and a deliberate local-variant merge/migration workflow.

## 4. Foundational editing contracts

- [ ] Implement typed property metadata, applicability, mixed values, and ownership.
- [ ] Implement creation defaults, per-property inherited styles, explicit overrides, and reset.
- [ ] Bulk-edit mixed selections atomically, reporting affected objects and restrictions.
- [ ] Establish the action registry, context rules, keymaps, and command-palette queries before wiring UI shortcuts.
- [ ] Add document units and structured per-view geometry, kept separate from electrical identity.
- [ ] Implement whole-route/vertex/segment editing, snapping, and local connection-preserving adjustments.
- [ ] Test property/style precedence, shortcut conflicts/typing protection, geometry invariants, and all command undo paths.

## 5. Desktop interaction prototype

- [ ] Select the frontend/rendering approach through a bounded Tauri prototype on Linux, Windows, and macOS.
- [ ] Add typed transport DTOs, revisioned projections, stale-result rejection, and resynchronization.
- [ ] Build the stable shell and reusable inspector/tool/menu controls before growing feature panes.
- [ ] Implement pan/zoom, selection, wire placement, Properties, and keyboard actions using shared infrastructure.
- [ ] Verify preview/commit agreement, cancellation, focus retention, and stable workspace bounds.
- [ ] Add native UI automation appropriate to each platform and representative rendering tests.
- [ ] Demonstrate coordinated overview and detailed wiring views backed by the same objects.

## 6. Libraries, labels, and project Git integration

- [ ] Add generic device/symbol, cable/wire, and manufacturer-part definitions with explicit terminal mapping.
- [ ] Add project-owned/pinned library content and explicit update adoption.
- [ ] Evaluate importing existing Pinout Generator documents; implement live connection-label bindings.
- [x] Retain project-local variant ancestry and verify two-project pinned adoption independently.
- [x] Publish immutable revisions to a local shared library with explicit recommendations, strict canonical schema, and guarded coherent persistence.
- [x] Coordinate cooperating library writers, reject history rewrites/stale versions, and test independent two-project import/adoption plus offline reopening.
- [x] Discover registered project references read-only, separating exact pins, local ancestry, and bundled-only revisions with explicit coverage/freshness.
- [x] Retain labelled historical usage after failed scans, distinguish copied locations from project lineage, and detect Git checkouts/content changes.
- [ ] Add durable usage registries, directory discovery/watchers, scan budgets/cancellation, cross-machine library coordination, and local-variant revision merging.
- [ ] Add a Git adapter and document lifecycle orchestration for status, commits, branches, conflicts, and reload. Existing tests demonstrate interoperability, not this application feature.
- [ ] Add basic connection/cable outputs and verify a small system can be built from its documentation.

## 7. Scale and platform validation

- [x] Local release-build sweep of 100, 1,000, and 10,000 unique devices, with 1,000 local edits and complete undo checks per case.
- [ ] Establish an actual representative car-scale fixture and usable reference hardware budgets.
- [x] Add headless hierarchy probes for unrelated block growth and depths through the 256-level guard, preserving interface/cable invariants and undo correctness.
- [x] Probe 1/10/100 materialized instances sharing one bundled revision, with placement, validation, round-trip, external-edit/undo, and one-occurrence adoption measurements.
- [ ] Expand probes to nested reusable dependencies, visible geometry, properties/styles, libraries, and assets.
- [ ] Measure memory, cache invalidation, object visits, save/load IO, native frame times, and foreground response during background work.
- [ ] Add bounded soak/cancellation/failure cases and establish stable performance regression gates from measured variance.
- [x] Current core checks and examples executed on macOS/aarch64 with Rust 1.97.0.
- [ ] Execute the same core suites on Windows.
- [ ] Execute the same core suites on Linux.
- [ ] Choose supported OS versions, architectures, Linux display environments, and packages.
- [ ] Native app build, installation, launch, interaction, and package checks on every supported release target.

## Per-feature completion record

For each implementation slice, record the relevant contract/decision, completed behavior, unit and integration tests, regression cases, local commands/results, and any unverified platforms or limitations. Reuse existing fixtures and runners; extend them where the feature creates a new risk.

Current test owners:

- Domain invariants: [project tests](../crates/ecad-domain/tests/project_invariants.rs).
- Hierarchy and cable invariants: [hierarchy tests](../crates/ecad-domain/tests/hierarchy_contract.rs) and [cable tests](../crates/ecad-domain/tests/cable_contract.rs).
- Controller/conveyor persistence and history: [hierarchy workflow](../crates/ecad-adapters/tests/hierarchy_workflow.rs).
- Reusable revisions, ownership, and local ancestry: [instance contracts](../crates/ecad-app/tests/reuse_contract.rs), [local-origin contracts](../crates/ecad-app/tests/local_origin_contract.rs), and [reuse persistence](../crates/ecad-adapters/tests/reuse_persistence.rs).
- Known-use indexing: [headless read contracts](../crates/ecad-app/tests/usage_index_contract.rs), [registered-file discovery](../crates/ecad-adapters/tests/known_usage.rs), and the existing Git workflow regression.
- Shared-library publication and persistence: [library contracts](../crates/ecad-adapters/tests/library_contract.rs) and [consuming-project workflows](../crates/ecad-adapters/tests/library/workflow.rs).
- Revision review/adoption: [adoption workflows](../crates/ecad-adapters/tests/adoption_workflow.rs) and [identity compatibility](../crates/ecad-adapters/tests/adoption/identity_contract.rs).
- History contracts: [session tests](../crates/ecad-app/tests/history_contract.rs) and internal overflow test.
- Transaction atomicity and session-token isolation: [transaction tests](../crates/ecad-app/tests/transaction_contract.rs).
- Saved checkpoints and reopen behavior: [saved-state tests](../crates/ecad-adapters/tests/saved_state_contract.rs).
- Guarded saves and file bindings: [document save tests](../crates/ecad-adapters/tests/document_save.rs).
- Reload policies and replacement validation: [document reload tests](../crates/ecad-adapters/tests/document_reload.rs).
- Persistent identities and explicit legacy conversion: [migration tests](../crates/ecad-adapters/tests/migration_and_identity.rs).
- Persistence fixtures and rejection cases: [persistence tests](../crates/ecad-adapters/tests/persistence_contract.rs).
- Git interoperability: [local Git workflow](../crates/ecad-adapters/tests/git_workflow.rs).
- Scale correctness and measurements: [scale example](../crates/ecad-adapters/examples/scale.rs).

## Later scope

Alignment/distribution, retained arrays, advanced drawing constraints, image attachments, complete output conventions, dimensionally accurate DXF layout, and STEP/manifest handoff remain in the product scope. Introduce each with its own executable slice and tests; they are not part of the current headless foundation.

## Source repository

Local branch `main` is initialized and `origin` points to `https://github.com/a-g-child/ECAD.git`. The remote was reachable and its HEAD was unpopulated when connected. Repository setup does not imply that application-level Git integration or hosted CI has been implemented.
