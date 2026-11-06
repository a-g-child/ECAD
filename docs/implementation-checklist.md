# Implementation checklist

Status: headless foundation, compound editing, persistent identities, explicit v1 migration, and optimistic guarded save/reload implemented. Checked items describe executable behavior or completed checks, not the full intended CAD product. Keep this file current as features and tests land.

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
- [x] Define explicit v1-to-v2 migration assigning a chosen project ID; retain the original v1 fixture and test preservation of existing electrical identities.

## 3. Hierarchy and cable vertical slice

- [ ] Model definition lineage, immutable revision, and placed occurrence as distinct identities.
- [ ] Introduce nested subsystem containment, cycle rejection, stable traversal, and resource diagnostics.
- [ ] Add explicit external interfaces and conductor continuity across a subsystem boundary.
- [ ] Add cable assemblies and conductor membership without conflating drawn routes with connectivity.
- [ ] Build the controller/conveyor example from the scope with one nested subsystem and one cable.
- [ ] Test boundary mappings, independent repeated occurrences, save/reopen, rejection, undo/redo, and schema compatibility.

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
- [ ] Implement shared subsystem publication, project-local variants, known-use discovery, and two-project adoption tests.
- [ ] Add a Git adapter and document lifecycle orchestration for status, commits, branches, conflicts, and reload. Existing tests demonstrate interoperability, not this application feature.
- [ ] Add basic connection/cable outputs and verify a small system can be built from its documentation.

## 7. Scale and platform validation

- [x] Local release-build sweep of 100, 1,000, and 10,000 unique devices, with 1,000 local edits and complete undo checks per case.
- [ ] Establish an actual representative car-scale fixture and usable reference hardware budgets.
- [ ] Expand probes to hierarchy depth/fan-out, repeated definitions, visible geometry, properties/styles, libraries, and assets.
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
