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

- [x] Implement typed metadata and union inspection for all existing electrical object fields, with applicability, mixed/uniform values, semantic ownership, typed reference kinds, and access restrictions.
- [x] Implement portable drawing creation presets, named/project/view per-property inheritance, explicit override intent, and reset for fonts, text height, stroke width, and colour.
- [x] Bulk-edit supported electrical fields across mixed selections atomically; report changes, no-ops, and inapplicable pairs; reject actual restrictions/validation failures without partial mutation.
- [x] Extend union inspection/bulk editing to annotations and graphical polylines, with typed document lengths, effective origins, and exact inheritance-aware undo.
- [ ] Extend properties to physical wire attributes, library assignments, and custom schemas with their real fields.
- [x] Establish a headless action registry with shared availability, typed invocation requirements, explicit default/unbound policies, and searchable disabled actions.
- [x] Compile portable user keymaps with unknown-reference/scope/duplicate/prefix diagnostics; route direct keys, chords, function keys, and sequences with protected input, repeat policy, captured targets, timeout/cancellation, and replay protection.
- [x] Dispatch supported actions through the real document/history/save/property/creation contracts, preserving original typed failures.
- [ ] Build native/web event normalization and menu wiring, keymap editor/preference-file lifecycle, physical/layout/reserved-key policy, action-ID migrations, and actual palette/placement controls.
- [x] Add integer-micrometre drawing views, positioned annotations, and retained whole graphical polylines separate from electrical identity.
- [x] Add view-local real-terminal anchors and whole conductor routes with exact electrical continuity and independent geometry across views.
- [x] Implement headless whole-route replacement, indexed vertex/segment edits, and deterministic first-leg/bend anchor adjustment with exact localized undo.
- [x] Add preflighted numeric group translation, explicit-coordinate origin alignment, and fixed-extrema origin distribution for annotations, graphical paths, and terminal anchors.
- [x] Move anchor groups simultaneously, translate complete routes when both ends share an offset, and retain exact union-layout inverses for one-operation undo.
- [x] Register M movement and remappable alignment/distribution actions; test selection validation, overflow, distribution ties, rollback/checkpoints, stale replies, and strict save/reopen.
- [x] Query deterministic terminal/endpoint/midpoint/grid snap candidates, with explicit radius, exclusions, stable ties, integer rounding, suppression, and cycling.
- [x] Prepare exact transient layout/move geometry at a captured revision and commit matching proposals through M and normal document/session history; reject stale/different-session previews, including no-ops.
- [x] Add exact pivot-based quarter-turn/mirror group transforms, complete internal-route transforms, local external-route adjustment, fixed-origin/self-loop edit reporting, exact previews, and R/remappable mirror actions.
- [x] Test transform overflow/rollback, four-turn and mirror cycles, upright annotation policy, independent views, closed-preview action guards, and v6 save/reopen/checkpoints.
- [ ] Add visual-bounds/edge-gap layout, symbol/live-label attachment, arbitrary-angle/scale and text-orientation transforms, interactive tools, spatial indexing/hysteresis/alignment snapping, pinned waypoints, and broader geometry operations.
- [x] Test electrical property applicability/types, pinned ownership, label uniqueness, topology validation, compound rollback, stale plans, saved checkpoints, and exact undo/redo.
- [x] Test style/default precedence, captured creation intent, equal overrides, reset/undo, focused shared-style persistence, strict drawing schema, malformed geometry/roles/references, and v2–v4 upgrades to v5.
- [x] Test shortcut conflicts/typing/composition/repeat/replay protection, context capture, sequence behavior, unknown imports, and shared real-document invocation.
- [x] Test route attachment/deletion guards, revision-adoption conflicts, no accumulated elbows, shared-anchor and self-loop movement, mixed properties, rollback/checkpoints, W requests, separate creation presets, and strict v6 persistence with v5 compatibility.
- [ ] Test native keyboard/menu focus behavior and interactive geometry invariants with their implementation.

## 5. Desktop interaction prototype

- [x] Build a bounded Tauri/TypeScript/SVG prototype and verify native macOS build/launch.
- [ ] Validate the frontend/rendering choice natively on Linux and Windows before treating it as a release decision.
- [x] Add bridge-v1 DTOs and full revisioned projections, stale-request/context validation, lossless identity/revision/editor strings, and host input-ID resynchronization.
- [ ] Add incremental projections, cancellation/background budgets, generated comprehensive transport schemas, and large-document scene virtualization.
- [x] Build fixed shell-owned pane bounds, shared menu/tool/property/table controls, reserved status feedback, and internal scrolling; test long values, errors, and resizing.
- [x] Implement SVG pan/zoom, fit, whole-object/mixed selection, right-click/full Properties, bounded typed bulk editors, shared undo/redo/save/open/fit/palette/report paths, and protected keyboard input.
- [x] Integrate M with host-retained closed previews, explicit reference choice, exact numeric placement, terminal/endpoint/midpoint/grid feedback, cycling/suppression, and one-step connected-route movement.
- [x] Verify preview/commit agreement, cancellation, invalid proposals, saved geometry/history, and frontend-clock reload protection through local bridge/browser tests.
- [x] Establish CAD wheel controls, directional/additive marquee selection with stroke/text bounds, direct group dragging with preserved grab offset, and asynchronous release/cancellation regressions.
- [x] Integrate R rotation and mirror controls with exact pivot fields, shared preview geometry, invalid/stale/cancelled proposal guards, one-step history, and saved-file/browser regressions.
- [x] Integrate T annotation placement with generated UUIDs, real text presets, literal previews, snapping/exact input, one-step creation/save/history, deleted-selection cleanup and control-focus regressions.
- [ ] Add interactive wire/symbol placement, sub-element hit testing/selection, docking/splitters, and broader native interaction.
- [x] Verify browser edit-to-core-to-render agreement, failed-edit focus retention, one-step undo, real saves/conflicts, report values, and fixed pane bounds using Chromium with the real Rust bridge.
- [ ] Verify interactive tool preview/commit agreement and cancellation in the native UI.
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
- [x] Add a revision-bound electrical connection projection and shared read-only report action, retaining all terminal attachments, self-loops, spare ends, empty cables, actual cores, and overview endpoints.
- [x] Export deterministic wire lists, terminal tables, and cable schedules as CSV with freshness guards, stable IDs, explicit draft states, Unicode/quoted/multiline labels, hierarchy paths, and independent golden fixtures.
- [ ] Add placed live pinout labels/templates, user-facing report/export controls, physical wire attributes/numbers, and verify a small system can be built from its documentation.

## 7. Scale and platform validation

- [x] Local release-build sweep of 100, 1,000, and 10,000 unique devices, with 1,000 local edits and complete undo checks per case.
- [ ] Establish an actual representative car-scale fixture and usable reference hardware budgets.
- [x] Add headless hierarchy probes for unrelated block growth and depths through the 256-level guard, preserving interface/cable invariants and undo correctness.
- [x] Probe 1/10/100 materialized instances sharing one bundled revision, with placement, validation, round-trip, external-edit/undo, and one-occurrence adoption measurements.
- [x] Add mixed-property inspection/preflight/commit/undo probes at 20/200/2,000 selected objects with correctness checks.
- [x] Probe shared-style updates, resolution, mixed graphical edits, and exact undo at 20/200/2,000 drawing objects.
- [x] Probe synthetic registry growth at 100/1,000/10,000 unbound entries with palette coverage and 1,000 direct-key routes per case.
- [x] Probe 100/1,000/10,000 routes with fixed-degree anchor movement, 1,000 edits, bounded bends, round trips, and complete undo.
- [x] Measure grouped fixed-degree anchor movement and separate layout preflight/commit/undo over 20/200/2,000 annotation origins.
- [x] Probe deterministic endpoint/midpoint snap queries over 100/1,000/10,000 graphical paths, documenting the initial linear scan and separate native/UI limits.
- [x] Measure 1,000 fixed-degree two-anchor half turns through 10,000 routes and quarter-turn annotation-position preview/commit/undo through 2,000 objects.
- [x] Measure report construction and three in-memory CSV encoders at 100/1,000/10,000 paired devices with individual cables; verify attachment coverage and deterministic round-trip outputs.
- [ ] Expand probes to large bound keymaps, nested reusable dependencies, visible rendering, libraries, and assets.
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
- Action/keymap contracts: [registry/router tests](../crates/ecad-app/tests/action_contract.rs), [real invocation workflows](../crates/ecad-adapters/tests/action_workflow.rs), and [keymap persistence](../crates/ecad-adapters/tests/keymap_persistence.rs).
- Desktop bridge and frontend: [bridge contracts](../crates/ecad-bridge/tests/document_bridge.rs), [move lifecycle](../crates/ecad-bridge/tests/move_bridge.rs), [transform lifecycle](../crates/ecad-bridge/tests/transform_bridge.rs), [annotation creation](../crates/ecad-bridge/tests/annotation_bridge.rs), [browser workflows](../apps/desktop/tests/editor.test.mjs), and [core host requests](../crates/ecad-app/tests/desktop_surface_contract.rs).
- Electrical reports and CSV outputs: [projection/workflows](../crates/ecad-adapters/tests/connection_report_workflow.rs), [golden/escaping/persistence contracts](../crates/ecad-adapters/tests/reports/persistence.rs), and [shared action](../crates/ecad-adapters/tests/reports/actions.rs).
- Orthogonal transforms: [coordinate rules](../crates/ecad-domain/tests/transform_contract.rs), [connected workflows](../crates/ecad-adapters/tests/transform_workflow.rs), [failure/rollback cases](../crates/ecad-adapters/tests/transforms/failures.rs), and [shared action previews](../crates/ecad-adapters/tests/transforms/actions.rs).
- Snapping and exact previews: [snap rules](../crates/ecad-domain/tests/snap_contract.rs), [placement workflows](../crates/ecad-adapters/tests/placement_workflow.rs), and [shared M preview/commit](../crates/ecad-adapters/tests/placement/actions.rs).
- Origin layout and group movement: [layout workflows](../crates/ecad-adapters/tests/layout_workflow.rs), [failure/rollback regressions](../crates/ecad-adapters/tests/layout/failures.rs), and [shared action workflows](../crates/ecad-adapters/tests/layout/actions.rs).
- Whole conductor routes: [workflow contracts](../crates/ecad-adapters/tests/route_workflow.rs), [persistence/action/adoption regressions](../crates/ecad-adapters/tests/routes/persistence.rs), and [routed fixture](../crates/ecad-adapters/tests/fixtures/routed-conveyor-v6.json).
- Drawing defaults/styles: [workflow contracts](../crates/ecad-adapters/tests/style_workflow.rs), [persistence/compatibility contracts](../crates/ecad-adapters/tests/styles/persistence.rs), and [styled golden fixture](../crates/ecad-adapters/tests/fixtures/styled-overview-v6.json).
- Electrical properties: [inspection and bulk workflows](../crates/ecad-adapters/tests/property_workflow.rs) and [restriction/rollback contracts](../crates/ecad-adapters/tests/properties/restrictions.rs).
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

Visual-bounds alignment and edge-gap distribution, retained arrays, advanced drawing constraints, image attachments, complete output conventions, dimensionally accurate DXF layout, and STEP/manifest handoff remain in the product scope. Introduce each with its own executable slice and tests; they are not part of the current headless foundation.

## Source repository

Local branch `main` is initialized and `origin` points to `https://github.com/a-g-child/ECAD.git`. `main` has been pushed and tracks `origin/main`. Repository setup does not imply that application-level Git integration or hosted CI has been implemented.
