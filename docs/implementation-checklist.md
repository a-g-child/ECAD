# Implementation checklist

Current UI rework follows the [contextual editor integration roadmap](contextual-editor-integration.md); earlier entries below record delivered historical slices, not competing future UI plans.

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
- [x] Make text multiline-capable without object conversion: shared textarea controls, explicit line/blank-space rendering, complete selection/Fit bounds, multiline captions/reports, and creation/property/history regressions.
- [x] Integrate P retained graphical paths with accepted/tentative geometry, exact/snapped vertices, draft Backspace/cancel, queued click intents, one-step creation/history and stroke-preset/save/whole-object regressions.
- [x] Group commands into fixed File/Drawing/Library ribbons with quick access, accessible tab navigation, draft/viewport preservation, shared action dispatch and explicit planned library/Git controls.
- [x] Integrate origin Align/Distribute through shared ribbon/menu/palette eligibility, editable exact previews, connected-route movement, no-op redo preservation, and real save/history/minimum-width regressions.
- [x] Expose project font/text-height/stroke-width/colour defaults through shared actions, a bounded modal editor, atomic validation, inheritance-preserving updates, exact save/history and no-op redo checks.
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
- Desktop bridge and frontend: [bridge contracts](../crates/ecad-bridge/tests/document_bridge.rs), [move lifecycle](../crates/ecad-bridge/tests/move_bridge.rs), [transform lifecycle](../crates/ecad-bridge/tests/transform_bridge.rs), [origin arrangement](../crates/ecad-bridge/tests/arrangement_bridge.rs), [drawing defaults](../crates/ecad-bridge/tests/defaults_bridge.rs), [annotation creation](../crates/ecad-bridge/tests/annotation_bridge.rs), [path creation](../crates/ecad-bridge/tests/path_bridge.rs), [browser workflows](../apps/desktop/tests/editor.test.mjs), and [core host requests](../crates/ecad-app/tests/desktop_surface_contract.rs).
- Electrical reports and CSV outputs: [projection/workflows](../crates/ecad-adapters/tests/connection_report_workflow.rs), [golden/escaping/persistence contracts](../crates/ecad-adapters/tests/reports/persistence.rs), and [shared action](../crates/ecad-adapters/tests/reports/actions.rs).
- Orthogonal transforms: [coordinate rules](../crates/ecad-domain/tests/transform_contract.rs), [connected workflows](../crates/ecad-adapters/tests/transform_workflow.rs), [failure/rollback cases](../crates/ecad-adapters/tests/transforms/failures.rs), and [shared action previews](../crates/ecad-adapters/tests/transforms/actions.rs).
- Snapping and exact previews: [snap rules](../crates/ecad-domain/tests/snap_contract.rs), [placement workflows](../crates/ecad-adapters/tests/placement_workflow.rs), and [shared M preview/commit](../crates/ecad-adapters/tests/placement/actions.rs).
- Origin layout and group movement: [layout workflows](../crates/ecad-adapters/tests/layout_workflow.rs), [failure/rollback regressions](../crates/ecad-adapters/tests/layout/failures.rs), and [shared action workflows](../crates/ecad-adapters/tests/layout/actions.rs).
- Whole conductor routes: [workflow contracts](../crates/ecad-adapters/tests/route_workflow.rs), [persistence/action/adoption regressions](../crates/ecad-adapters/tests/routes/persistence.rs), and [routed fixture](../crates/ecad-adapters/tests/fixtures/routed-conveyor-v8.json).
- Drawing defaults/styles: [workflow contracts](../crates/ecad-adapters/tests/style_workflow.rs), [persistence/compatibility contracts](../crates/ecad-adapters/tests/styles/persistence.rs), and [styled golden fixture](../crates/ecad-adapters/tests/fixtures/styled-overview-v8.json).
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

## Multi-project workspace implementation

Follow the ordered [workspace and sheet task list](workspace-and-sheet-implementation.md), including the rework map for host/bridge lifecycle, project-scoped routing, paper input, explorers, tabs, profiles and Git/report caches.

- [x] Add headless ProjectWorkspace with independent Document<S> controllers, runtime handles, active selection, normalized-location keys and captured-project operations.
- [x] Verify same-lineage file copies, isolated history/dirty/save/reload state, reserved/failed Save-as, stale/dirty close, shared input replay protection and failed operations with real temporary files.
- [ ] Integrate the owner into the native host/bridge; desktop currently still has one project context.
- [x] Add default project/document/page navigation and an alternate electrical/component explorer for the currently open project.
- [ ] Add multiple project roots and selected-project context after native workspace integration.
- [ ] Add project-scoped sheet tabs and lifecycle/preview routing, with all-project close/quit protection.

## Drawing sheets and templates

Target corrected by [decision 0030](decisions/0030-sheet-first-project-documents.md): Project → Documents → Sheets, with direct template-backed sheet editing for every actual project drawing. The bounded desktop now edits directly on template-backed sheets; project properties/typed page creation are implemented; logical document management, multi-project host integration and page tabs remain pending.

- [x] Add logical drawing-document IDs, metadata, ordered ownership, transfer/rename/reorder commands and deterministic v9 ownership upgrade.
- [x] Add mandatory standard-v1 document default, bundled frame/live fields and strict v10 persistence with deterministic legacy upgrades.
- [x] Integrate the frame with a unified editable sheet canvas, paper Fit (including blank pages), source-unit input/tools and transient per-sheet cameras.
- [x] Preserve legacy sheets, source records and shared/unplaced views in a deterministic initial drawing document without renumbering or copying.
- [x] Offer explicit reviewed per-view sheet attachment for unplaced views and expose shared-view editing behavior; no silent load-time geometry copies.
- [x] Add reviewed independent schematic source creation through project-root/File/search actions, with exact combined view/sheet undo.
- [ ] Add bulk wrapping.
- [x] Replace the prototype Model view/read-only paper split with one editable template-backed sheet workspace and validated inverse placement input.
- [x] Resolve a basic standard frame/eight live fields for every drawing sheet in paper preview and numeric review, with small-paper/overflow diagnostics.
- [ ] Expand custom table/template editing and output preflight.
- [x] Make pages/sheets the default Project explorer, retaining electrical structure as an alternate mode, with stable selection/expansion and guarded page context menus.
- [ ] Add selectable sheet tabs/panes with retained camera/selection, stable tab identity and shared project history.
- [x] Add schematic/cover/generated drawing-list roles and project-owned creation-template defaults; every page resolves a real bundled frame.
- [ ] Add richer overview/detail specialization and output drawing sets.
- [ ] Add off-sheet continuation representations tied to actual connectivity and stable target references.
- [ ] Keep symbol/component/footprint/template-definition editors separate from project sheet workspaces.
- [ ] Link hierarchical overview blocks to stable sheet identities and electrical interfaces independently of document ownership.
- [x] Include reusable sheets/title-block tables, scoped variables, company profiles and standard/draft/cover/contents layouts in [the feature scope](sheets-and-templates.md).
- [x] Implement eight typed project metadata fields, literal multiline values, exact history/rollback, strict schema v7 and v2–v6 empty-metadata upgrades.
- [x] Integrate File/menu/palette project-information editing with fixed modal bounds, portable saved fields, cancel/no-op redo behavior and local Rust/browser regressions.
- [x] Add personal/company profile persistence and explicit reviewed selective copy/update into project values, with independent history, observed-file conflict checks and native/test host path isolation.
- [x] Add stable ordered sheets, oriented paper/margins, unique drawing numbers, one optional scaled view placement, exact history and strict v8 persistence.
- [x] Integrate reviewed paper editing, blank sheets, reorder/remove and direct template-backed sheet navigation/editing, with source-record preservation and minimum-width browser regressions.
- [ ] Add multiple named drawing sets, multiple placements, interactive paper placement and unified bulk sheet properties.
- [ ] Add stable table/cell identities, split/merge/resize and bulk formatting.
- [ ] Add field bindings, sheet-specific entries, override/reset and missing/required-field diagnostics.
- [x] Add bundled standard/cover/drawing-list frames and explicit physical drawing-list pagination with live stable destinations.
- [ ] Add custom shared/draft templates and reviewed pinned-library adoption.
- [ ] Add portable logos, output preflight and reproducible historical issue output.

## Later scope

Visual-bounds alignment and edge-gap distribution, retained arrays, advanced drawing constraints, image attachments, complete output conventions, dimensionally accurate DXF layout, and STEP/manifest handoff remain in the product scope. Introduce each with its own executable slice and tests; they are not part of the current headless foundation.

## Source repository

Local branch `main` is initialized and `origin` points to `https://github.com/a-g-child/ECAD.git`. `main` has been pushed and tracks `origin/main`. Repository setup does not imply that application-level Git integration or hosted CI has been implemented.

Frame slice: see [decision 0033](decisions/0033-bundled-sheet-frame.md), [frame workflows](../crates/ecad-adapters/tests/sheet_frames.rs) and sheet bridge/browser regressions. Direct sheet input and per-view reviewed attachment are now implemented; bulk wrapping and multi-project host integration remain unchecked.

Editable sheets: [decision 0034](decisions/0034-editable-sheet-workspace.md), [sheet-context bridge regressions](../crates/ecad-bridge/tests/sheet_editor_bridge.rs) and [scaled editing/attachment workflows](../apps/desktop/tests/sheet-editing.mjs). Native execution on Windows/Linux remains unverified.

Explorer slice: [decision 0035](decisions/0035-project-page-and-electrical-explorers.md), [bridge contracts](../crates/ecad-bridge/tests/explorer_bridge.rs) and [browser workflows](../apps/desktop/tests/explorer.mjs). One desktop project/root is supported; multi-project host integration, document management and page tabs remain pending.

Project-page creation: [decision 0036](decisions/0036-project-template-defaults-and-typed-pages.md), [project/default/list workflows](../crates/ecad-adapters/tests/project_pages.rs), [typed bridge contracts](../crates/ecad-bridge/tests/project_creation_bridge.rs) and [project-menu browser workflows](../apps/desktop/tests/project-pages.mjs). Project schema v11 preserves v2–v10 input; actual PDF generation, annotations/outlines, custom template authoring and native Windows/Linux execution remain pending.

## Consolidated template library/editor

See [decision 0037](decisions/0037-library-template-editor-workspace.md) and [requirements/task sequence](template-editor-plan.md). Headless layouts and local immutable template publication are implemented; project bundles and preservation capture are implemented; saved-reference assignments and saved-only desktop controls are implemented; a bounded independent template editor UI is implemented; interactive table authoring remains planned.

- [x] Add immutable template-variant bodies with typed IDs, explicit content bounds and validated structured table/cell partitions, including unequal rows/spans and literal/known-field text runs.
- [x] Add strict standalone template-variant schema v1 serialization and route existing sheet frames through project-independent legacy layout projection, preserving exact geometry/tiny-sheet behavior.
- [x] Publish complete immutable template revisions with explicit recommendations, strict library schema v1, observed-version saves and cooperating-writer coordination; failed writes/reloads retain state and storage rejects history rewrites.
- [x] Bundle/import complete current template bodies with exact project undo/rollback and strict canonical schema v12; reopen offline without the library.
- [x] Capture legacy sheet layouts and unused document/creation defaults deterministically, retaining paper, geometry and explicit override/inheritance intent in a closed revision-bound proposal.
- [x] Migrate normal desktop assignments/defaults to saved references with stable inherited variants; protect in-use revisions and reject invalid paper/variant changes and transfers.
- [x] Restrict sheet/project-default selectors to saved variants and make paper/margins read-only; retain source placement/scale editing and historical tiny-sheet recovery.
- [ ] Extend dependency closure/bundling when reusable title blocks and portable assets are implemented.
- [x] Store paper/margins and retained title-block structure in saved variants, with validated read-only sheet projections.
- [ ] Complete standard-profile provenance and the separate title-block authoring workflow.
- [x] Add Libraries access, a separate template editor view, independent draft history/publication, contextual ribbon and exact template/cell-content properties.
- [ ] Extend editor properties to full table geometry, shared styles, anchors and multiple selections.
- [ ] Add ISO/ANSI paper/profile presets with verified edition-specific layout rules and Custom provenance.
- [x] Add exact numeric table sizing, grid cell splits, rectangular merges with reviewed content policy, aligned-boundary edits, stable identities and editable multiline/variable runs.
- [x] Add direct rectangular cell selection, move/size grips, aligned-boundary drags and contextual row/column insertion with closed preview commits.
- [ ] Add/remove whole tables, localized span-aware resizing and reviewed custom sheet-entry value mapping.
- [x] Implement saved-only desktop assignment with deterministic legacy preservation and project bundling.
- [ ] Complete reviewed bulk revision adoption and custom value mapping.

Template foundations: [layout/partition regressions](../crates/ecad-domain/tests/template_layout.rs) and [standalone format checks](../crates/ecad-adapters/tests/template_variants.rs). The first independent editor is described below; encoding alone does not publish a variant or make it selectable.

Template foundation validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (41 browser workflows) and `cargo xtask desktop` passed, including nine new headless layout/serialization checks. The browser harness required loopback permissions outside the filesystem sandbox. Native Windows/Linux execution and template-editor interaction remain unverified.

Template publication: [decision 0038](decisions/0038-closed-template-revision-publication.md), [real-file catalogue/publication checks](../crates/ecad-adapters/tests/template_library.rs) and [offline executable example](../crates/ecad-adapters/examples/template_library.rs). Library publication is outside project history. Project import/bundling and preservation capture are implemented below; actual assignment migration, editor lifecycle and saved-only assignment remain pending.

Template publication validation (macOS/aarch64): `cargo xtask full` passed, including ten new template-publication checks and the thirteen existing subsystem-library checks. `cargo run --offline --locked -p ecad-adapters --example template_library` also passed. This is a headless storage/application change; UI/native interaction suites were not rerun. Project schema and sheet controls remain unchanged.

Project template bundles: [decision 0039](decisions/0039-project-template-bundles-and-legacy-capture.md), [bundle/capture regressions](../crates/ecad-adapters/tests/project_template_bundles.rs) and [offline example](../crates/ecad-adapters/examples/project_templates.rs). Explicit saves now write strict v12, retaining v2–v11 readers and historical fixtures. This captures preserved bodies without silently adopting them; saved-template sheet assignment and its UI remain pending.

Project bundle validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (41 browser workflows) and `cargo xtask desktop` passed, including seven new bundle/capture regressions. `cargo run --offline --locked -p ecad-adapters --example project_templates` passed. V12 canonical fixtures retain all earlier v11 fixtures/readers. Native Windows/Linux execution and a template-editor UI are not covered by these checks.

Saved template assignments: [decision 0040](decisions/0040-saved-template-assignments-and-inherited-variants.md) and [seven migration/assignment regressions](../crates/ecad-adapters/tests/template_assignment_migration.rs). Current desktop loads migrate in memory before the clean checkpoint; explicit saves write v13 with exact saved pins/inherited variants, and observed-byte conflicts remain protected. Standalone retained legacy bodies use v2 eligibility while ordinary v1 output stays unchanged. Template editor/ribbon/properties and interactive table authoring remain pending.

Validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (42 workflows) and `cargo xtask desktop` passed. Older v12 fixtures/readers are retained alongside canonical v13 fixtures. Browser migration tests use temporary project files and real Rust state; native Windows/Linux and output/font parity remain unverified.


Independent template editor: [decision 0041](decisions/0041-independent-template-editor.md), [headless history/publication checks](../crates/ecad-adapters/tests/template_editor.rs), [bridge lifecycle regressions](../crates/ecad-bridge/tests/template_editor_bridge.rs) and [real browser workflows](../apps/desktop/tests/template-editor.mjs). Library Save and project import have separate histories. Exact custom paper/content bounds and multiline cell captions/literal/variable runs are editable; numeric table geometry and split/merge/resize follow below; standard profiles and bulk adoption remain pending.

Independent editor validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (44 browser workflows) and `cargo xtask desktop` passed. Six new headless/bridge workflows cover draft history, durable failures, stale contexts, pending fields, explicit import and closed-editor guards. Browser checks cover the actual separate editor, multiline properties, fixed minimum-window bounds and independent publication/history. Native checks compile/lint the available host; they are not native interaction automation or Linux/Windows execution.


Structured template table tools: [decision 0042](decisions/0042-structured-template-table-edits.md), [domain regressions](../crates/ecad-domain/tests/template_table_edit.rs), [bridge history/publication checks](../crates/ecad-bridge/tests/template_table_bridge.rs) and [browser workflows](../apps/desktop/tests/template-tables.mjs). Exact numeric edits preserve full partitions, typed contents and stable survivor identities. Pending table/inspector proposals have separate ownership. No persistence schema change or automatic project adoption is introduced.

Structured table validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (46 browser workflows) and `cargo xtask desktop` passed. Four new domain regressions and two bridge workflows cover partitions, identities/content policies, extreme geometry, exact history/publication and pending-field ownership. Two browser workflows cover real dialogs, blank-cell content creation, error retention, cancellation/redo, publication and minimum-window bounds. Native Linux/Windows execution and native interaction automation remain unverified.


Direct template interaction: [decision 0043](decisions/0043-direct-template-table-interaction.md), [band insertion regressions](../crates/ecad-domain/tests/template_table_insert.rs), [closed-gesture bridge checks](../crates/ecad-bridge/tests/template_table_gestures.rs) and [pointer browser workflows](../apps/desktop/tests/template-table-interactions.mjs). Range selection closes over merged spans and feeds merge review. Drag previews retain original targets and commit exact validated bodies; contextual plus buttons grow rows/columns while preserving bottom/right placement. Scene/camera/selection remain transient and project/sheet pins do not change.

Direct interaction validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (49 browser workflows) and `cargo xtask desktop` passed. Three new domain tests cover span-aware band insertion and anchor/overflow/identity failures; three bridge workflows cover retained preview bodies and token/target/cancellation boundaries. Three browser workflows cover rectangular/merged-cell selection, resize/move/outer edges, zoom-stable controls, keyboard-accessible plus buttons, stable inserted identities and delayed/invalid/cancelled releases. Native Windows/Linux execution and native interaction automation remain unverified.


Template margins: [decision 0044](decisions/0044-template-margin-controls-and-snapping.md), [domain regressions](../crates/ecad-domain/tests/template_margins.rs), [bridge history/preview checks](../crates/ecad-bridge/tests/template_margin_bridge.rs) and [browser workflows](../apps/desktop/tests/template-margins.mjs). Pointer moves/outer resizes snap to the corresponding margin border with validated feedback and Alt bypass. General margin edits all four sides; the visible fitting option translates tables/intersects content without scaling cells. Numeric placement remains exact and click-only handles preserve it. No schema change, persistent attachment or implicit project adoption is introduced.

Margin validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (51 browser workflows) and `cargo xtask desktop` passed. Four new domain tests cover border/corner/tie/tolerance/extreme geometry and fitting; two bridge workflows cover snap feedback, numeric/bypass semantics, captured-edge protection and complete saved history. Two browser workflows cover shared/mixed margins, fitting, rejected edits, drag feedback, Alt/numeric alternatives and click-only placement preservation. The click-only regression also passed in a final targeted browser run. Native Windows/Linux execution and native interaction automation remain unverified.

### Template text and vector artwork

- [x] Retained stable artwork identities; multiline literal/variable text, rectangles, ellipses and coherent line/polylines in exact paper units.
- [x] Template ribbon/command-search Artwork action, explorer/canvas property access, property-based creation/edit/removal with one Apply/Undo and pending-field guards.
- [x] Explicit font, text height, outline width and colour; paper-bounds validation, clipped text with overflow feedback, artwork behind title-block tables.
- [x] Shared typed variable resolution from each project/document/sheet; independent template bodies never store resolved project values.
- [x] Strict nested variant v3 artwork persistence, existing v1/v2 encoding preserved for artwork-free bodies; project/library round trips and table-edit preservation.
- [x] Two-point pointer drawing and whole-object dragging (see pointer tools below).
- [ ] Artwork snapping/transform tools; image/logo portable assets, fills and arcs.

Validation: added adapter/domain integration coverage for all shapes, malformed/duplicate/out-of-paper geometry, unknown fields, version rejection, exact canonical round trips and per-sheet variable resolution; bridge regression covers Apply/Undo/Redo, failed edits, table preservation and library saves. Browser workflow covers multiline bindings, all shapes, exact polyline edits, validation, Revert, history and saved-template reopen. See decision 0045 for first-slice limits.

Local verification for this slice: `cargo xtask full`, `cargo xtask ui` (52 browser workflows), and `cargo xtask desktop` passed on macOS/aarch64. Browser tests use the real Rust loopback host; desktop checks are host build/lint checks, not native interaction automation. Windows/Linux runtime validation remains pending.

### Template artwork pointer tools

- [x] Registered text/line/rectangle/ellipse drag placement and whole-object movement, accessible through the template ribbon and command search.
- [x] Checked paper geometry, reverse box drags, retained line endpoint order, complete local path/text/style preservation during movement.
- [x] Shared revision/target-bound retained previews, guarded armed tools, coalesced pointer updates, exact final-release commits, Escape/capture-loss cancellation and coherent history.
- [x] Domain geometry/failure regressions, bridge arming/dirty guards/history/token/retarget tests, and Rust-backed browser placement/movement/delayed-release/cancel/invalid-release workflow.
- [ ] Multi-click polyline drawing, pointer resizing and vertex handles, artwork snaps and portable image assets.

See [decision 0046](decisions/0046-template-artwork-pointer-tools.md). Existing numeric properties and variable text editing remain available after placement.

Pointer-tool validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (53 browser workflows), and `cargo xtask desktop` passed. New coverage includes two domain regressions, two retained-gesture bridge workflows and a browser workflow covering every primitive, delayed movement release, click-only redo preservation, armed cancellation and invalid release. Desktop checks are host build/lint, not native interaction automation; Windows/Linux runtime validation remains pending.

### Contextual template selection — first integration slice

- [x] One typed editor selection for cell ranges, artwork and paper; gesture controllers consume projections or read the current selection.
- [x] Automatic contextual inspector; selecting artwork clears table handles, selecting cells clears artwork highlight, blank paper/canvas Escape clears selection.
- [x] Explicit full-cell template hit areas protect table selection over background artwork.
- [x] Stable-ID reconciliation after undo/delete/reopen; pending property edits remain protected.
- [x] Add shared contextual field completion with captured proposals, validation recovery and selection handoff.
- [x] Add typed template selection and shared core Split/Merge eligibility.
- [ ] Retire transitional Apply/Revert/catalog surfaces as insertion tools replace them.
- [x] Simplify Split/Merge and paper presets.
- [ ] Extract shared drawing/text modules, then add table/border/image insertion with persistence support.

The numeric artwork-management surface remains transitional. No document schema, library publication or project adoption semantics change in this slice.

Contextual-selection validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (54 browser workflows), and `cargo xtask desktop` passed. The new browser reproducer covers cell interiors overlapping artwork, automatic inspector switching, explorer/right-click targeting, blank-paper/Escape deselection, pending-field protection and unchanged undo semantics. A final focused run also verified cancellation of a cell-range gesture restores the previous artwork selection. Native interaction automation and Windows/Linux runtime checks remain pending.


### Contextual property completion

- [x] Complete contextual fields on Enter, Tab/blur, select and checkbox changes; preserve multiline Enter and IME composition.
- [x] Capture exact proposal/target/revision before asynchronous commit; deduplicate commits and defer pointer selection until validation succeeds.
- [x] Preserve invalid fields and the original selection; Escape reverts staged fields without changing committed history.
- [x] Restore inspector focus after completion and protect global shortcuts while controls are locked.
- [x] Keep numeric artwork catalog structural edits staged and explicit Apply/Revert available as fallback.

Coverage: [field completion workflow](../apps/desktop/tests/template-property-completion.mjs) and updated [contextual selection regression](../apps/desktop/tests/template-selection.mjs). Shared schematic adoption and selection-aware Rust action eligibility remain later work; persistence and publication contracts are unchanged.

Field-completion validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (55 browser workflows), and `cargo xtask desktop` passed. New coverage includes Enter/Tab completion, multiline/IME protection, invalid-selection recovery, redo retention and delayed commits with rapid target changes. Native host checks build/lint only; Windows/Linux runtime validation remains pending.


### Contextual structural tools and paper presets

- [x] Separate typed template selection in action context; read-only Rust availability drives ribbon/palette and dispatch revalidates targets.
- [x] Single-cell/contiguous-column splits commit atomically; bounded subdivision counts preserve exact identities, contents and undo.
- [x] Rectangular-only merge, deterministic spatial survivor, selected targets without redundant selectors, and content choices only when needed.
- [x] Paper family/size/orientation/general margin controls; exact custom geometry and individual margins under Advanced, with no drawing scaling.
- [x] Domain/bridge/browser regressions for invalid selections, stale/mismatched targets, column history, preset/custom preservation and library reopen.

See [decision 0047](decisions/0047-contextual-table-actions-and-paper-presets.md). Existing property Apply/Revert fallback, artwork management and numeric table placement remain available. Standardized margin profiles and native Windows/Linux execution remain pending.

Contextual-tool validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (57 browser workflows), and `cargo xtask desktop` passed. Desktop checks are host build/lint checks, not native interaction automation or Windows/Linux runtime validation.


### Shared previews and click placement

- [x] One reusable latest-preview queue for template gestures and schematic annotation placement, with coalescing, stale-response suppression and finish draining.
- [x] Template line/rectangle/ellipse start–finish clicks with anchor feedback; preserve drag placement and whole-artwork movement.
- [x] Accept schematic text finish while a preview is loading; validate the final text/position and retain invalid input for correction.
- [x] Regression workflows for delayed canvas clicks, delayed numeric/text finish, invalid final proposals, cancellation, exact history and anchor cleanup.
- [x] Inline template text/variable placement.
- [x] Successive-vertex template paths.
- [x] Shared font-family/colour controls for existing template artwork, schematic properties and defaults.
- [ ] Further tool extraction and additional style capabilities.

See [decision 0048](decisions/0048-shared-preview-queue-and-click-placement.md). Ordered schematic path intents remain separate from replaceable pointer previews. Persistence and electrical connectivity are unchanged.

Shared-preview validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (60 browser workflows), and `cargo xtask desktop` passed. Native desktop checks build/lint the available host; native interaction automation and Windows/Linux runtime testing remain pending.


### Inline template text placement

- [x] Click an anchor or drag a text box, then edit multiline literal/variable runs beside it inside a bounded canvas overlay.
- [x] Reuse the existing run controls; notify preview owners after updating the typed run value.
- [x] Preview complete text proposals without document edits; Place/Ctrl+Enter commits one retained token and one undo entry.
- [x] Protect composition, reject blank/unknown runs, retain invalid text for correction, and cancel without losing redo.
- [x] Cover bridge validation/history and browser live preview, delayed completion, viewport bounds, drag compatibility and library reopen.

See [decision 0049](decisions/0049-inline-template-text-placement.md). New text defaults to a bounded 60 × 20 mm box; appearance and exact box editing remain contextual properties. Persistence, template pins and electrical geometry are unchanged.

Inline-text validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (61 browser workflows), and `cargo xtask desktop` passed. A final focused inline workflow and frontend build also passed after refining focus restoration on invalid placement. Desktop checks are host build/lint, not native interaction automation; Windows/Linux runtime validation remains pending.


### Multi-point template paths

- [x] Registered ribbon/palette Path action; successive clicks, Backspace revision, Enter finish and Escape cancellation.
- [x] Shared schematic/template path key policy with composition and repeat protection.
- [x] Complete ordered vertex previews preserve rapid clicks across delayed responses; hover remains a visual guide.
- [x] Checked signed paper coordinates, closed outlines, 2–4096 vertex bounds, invalid-preview token rejection and recoverable invalid drafts.
- [x] Whole-object movement, numeric vertex editing, exact single-step history and saved-library reopen using existing polyline persistence.

See [decision 0050](decisions/0050-template-multi-point-paths.md). Template snapping, numeric placement, orthogonal constraints and pointer vertex handles remain pending; these paths do not affect electrical topology.

Path validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (62 browser workflows), and `cargo xtask desktop` passed. Desktop checks cover host build/lint; native interaction automation and Windows/Linux runtime validation remain pending.


### Shared appearance controls

- [x] Reuse generic font-family and named RGB choices alongside exact custom values across template artwork, schematic properties and drawing defaults.
- [x] Preserve mixed selection, inherited/explicit ownership, existing validation and each owner's commit/undo rules.
- [x] Treat exact/preset controls as one contextual focus group; preserve invalid drafts for correction.
- [x] Keep multiline font requests and bounded pane layout; cover behavior through real Rust-backed browser workflows.

See [decision 0051](decisions/0051-shared-appearance-controls.md). Font weight/style, advanced colour selection, table-cell typography and native font/output parity remain pending.

Appearance validation (macOS/aarch64): `cargo xtask full`, `cargo xtask ui` (64 browser workflows), and `cargo xtask desktop` passed. The inline overlay regression now waits for its resize-observer layout before checking bounds. Desktop checks cover host build/lint; native interaction automation and Windows/Linux runtime validation remain pending.
