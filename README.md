# ECAD

An electrical CAD application in early development, targeting equipment systems around the scale of a car. The intended desktop platforms are Linux, Windows, and macOS.

A first **Tauri desktop editor prototype** now runs against the real Rust engine. It provides a fixed-pane workspace with File/Drawing/Library ribbons, SVG drawing with cursor-centred zoom, modifier panning, directional marquee selection and snapped/exact group dragging, pivot-based rotation/mirroring, exact origin alignment and equal-spacing distribution, multiline annotation and retained graphical path placement, a full property inspector with bounded typed editors, project drawing defaults and portable project information, undo/redo, guarded open/save, command search, and a live connection table. See [desktop setup and limits](docs/desktop-prototype.md).

The headless engine supports electrical hierarchy, cables, pinned reusable revisions, document/history guards, mixed property edits, defaults/styles, retained conductor routes, deterministic layout/snapping/previews/transforms, and electrical CSV outputs. It remains independent of Tauri and the frontend. Interactive wire/symbol placement, manufacturer parts, full native interaction automation, and Linux/Windows execution remain planned.

The document controller now supports guarded saves, save-as, external-change detection, and validated reloads. Persistent project IDs and OS-random UUID-v4 object generation are available without adding IO dependencies to the core.

## Start here

- [Implementation checklist](docs/implementation-checklist.md): completed work, next slices, and pending platform checks.
- [Product scope](docs/product-scope.md): intended product behavior and boundaries.
- [Architecture](docs/architecture.md) and [coding standards](docs/coding-standards.md).
- [Testing strategy](docs/testing-strategy.md): local suites and scalability experiments.

## Local setup

Install Rust 1.97 or newer with Cargo, rustfmt, and Clippy, plus Git for the local repository integration test. Node and desktop system dependencies are unnecessary for the headless core/bridge checks. The desktop UI requires Node 22.12+ and Tauri platform prerequisites. The current baseline was tested with Rust 1.97.0 on macOS/aarch64; Windows and Linux execution is pending. An exact rustup toolchain pin remains a checklist item; `rust-version` currently records the minimum version.

With dependencies available locally:

```sh
cargo xtask full
```

On a fresh machine, populate the dependency cache once first:

```sh
cargo fetch --locked
```

`Cargo.lock` is included and should be versioned with the project. The local check runner invokes Cargo with `--offline --locked`; subsequent checks do not need GitHub or network access. A fresh platform may need its own dependency cache and system tooling.

## Launch the desktop prototype

```sh
cd apps/desktop
npm ci
npm run desktop
```

The app starts with a real, editable conveyor demonstration. Save it to a new file to keep changes. Native open/save choosers are available in the desktop app; existing unobserved destinations are protected. Full prerequisites, offline setup, local tests, and prototype limits are in [desktop setup](docs/desktop-prototype.md).

## Working commands

| Command | What it does |
| --- | --- |
| `cargo xtask quick` | Dependency-boundary checks, formatting, Clippy, unit/integration/regression tests |
| `cargo xtask full` | Quick checks plus example-target compilation, doctests, and documentation with warnings treated as errors |
| `cargo xtask ui` | Bridge tests, frontend formatting/type/build checks, and Chromium workflows against the real Rust engine |
| `cargo xtask desktop` | Frontend build/format checks plus offline native-host Rust formatting and Clippy; does not automate the native window |
| `cargo xtask scale` | Optimized headless probe at 100, 1,000, and 10,000 devices; writes `target/scale/latest.json` |
| `cargo run --offline --locked -p ecad-adapters --example headless` | Creates two devices and one conductor, exercises undo/redo, and saves/reopens in an automatically cleaned temporary directory |
| `cargo run --offline --locked -p ecad-adapters --example conveyor` | Builds a controller, nested conveyor/sensor assembly, interfaces, cable, and internal wiring as one transaction; verifies undo/redo and save/reopen |
| `cargo run --offline --locked -p ecad-adapters --example actions` | Routes T to captured placement input, creates one annotation, rejects duplicate delivery, and verifies undo/redo/guarded save/reopen |
| `cargo run --offline --locked -p ecad-adapters --example drawing_documents` | Groups sheets into logical documents, preserves shared source views, and verifies exact undo and v10 save/reopen |
| `cargo run --offline --locked -p ecad-adapters --example workspace` | Opens two same-lineage projects in isolated temporary files and verifies independent editing, dirty state, targeted saves and close guards |
| `cargo run --offline --locked -p ecad-adapters --example reports` | Prints a wire list, checks report freshness after a rename, refreshes values, and verifies undo/reopen output stability |
| `cargo run --offline --locked -p ecad-adapters --example transforms` | Previews rotation and mirroring of connected anchor groups, then verifies exact route geometry, unchanged connectivity, save/reopen, and undo/redo |
| `cargo run --offline --locked -p ecad-adapters --example placement` | Previews a grid-snapped group move without document mutation, commits matching geometry, and verifies save/reopen and exact undo/redo |
| `cargo run --offline --locked -p ecad-adapters --example layout` | Translates connected anchors together, aligns origins numerically, verifies distribution no-ops, and checks exact grouped undo/save/reopen |
| `cargo run --offline --locked -p ecad-adapters --example routes` | Edits a whole route, moves a shared terminal anchor, verifies electrical guards, and checks save/reopen and exact undo/redo |
| `cargo run --offline --locked -p ecad-adapters --example styles` | Demonstrates captured creation presets, shared inheritance, equal-valued override intent, reset, exact undo, and portable save/reopen |
| `cargo run --offline --locked -p ecad-adapters --example properties` | Inspects a mixed selection, edits applicable subsets in one transaction, and verifies save/reopen and exact undo |
| `cargo run --offline --locked -p ecad-adapters --example known_usage` | Scans registered files, reports incomplete coverage, distinguishes pins/local ancestry, and retains labelled historical use after deletion |
| `cargo run --offline --locked -p ecad-adapters --example shared_library` | Publishes two revisions, explicitly imports/adopts one consumer, and verifies project undo leaves shared publication intact |
| `cargo run --offline --locked -p ecad-adapters --example adoption` | Reviews/adopts one occurrence, preserves the other pin and external wiring, saves/reopens, and verifies undo |
| `cargo run --offline --locked -p ecad-adapters --example reusable` | Places two instances of one revision, saves/reopens offline, then verifies make-local/undo without changing the other instance |
| `cargo doc --offline --locked --workspace --no-deps --open` | Opens generated Rust API documentation |

The full suite currently contains 318 behavior tests and two doctests. Integration tests use isolated temporary files and repositories, including a real Git branch/commit/checkout workflow and safe reload after checkout. They do not alter this workspace's repository or contact a remote. This workspace's `origin` is configured as [a-g-child/ECAD](https://github.com/a-g-child/ECAD).

Four additional browser workflows verify actual edits/saves, mixed selection, protected typing, external conflicts, long values, and stable pane geometry. Native macOS build and launch are verified; automated native interaction, Linux, and Windows remain pending.

The scale probe varies unique device/terminal/conductor counts, unrelated subsystem counts, and hierarchy depth while editing a local connection 1,000 times per case. It records core timing and verifies round trips and undo correctness. Hierarchy cases cover 100/1,000/10,000 unrelated blocks and depth 16/128/256, with explicit terminal exposure and a real cable grouping. Repeated-definition cases additionally cover 1/10/100 materialized instances and one local interface edit. Each repeated-instance case also measures one adoption preview, commit, and undo; these are single samples, not regression gates. Property cases measure inspection, preflight, commit, and undo over 20/200/2,000 selected device/conductor objects. Drawing cases measure one shared-style update, per-property resolution, mixed graphical edits, and undo at 20/200/2,000 drawing objects. Correctness checks cover unchanged dependent records, preserved override intent, round trips, and exact restoration. Synthetic action cases add 100/1,000/10,000 unbound registry entries and measure registration, keymap compilation, palette search, and 1,000 direct-key routes; this does not measure large bound-keymap conflict sets or native input latency. Route cases grow from 100 to 10,000 routes while moving one anchor or a pair with two touching routes 1,000 times each, also applying 1,000 two-anchor half turns, checking bounded bends, unchanged electrical data, round trips, and complete undo. Layout cases separately time preflight (including a temporary full-project candidate), batch commit, and exact undo for origin alignment over 20/200/2,000 annotations; these are single samples. The same layout cases separately measure quarter-turn annotation-position preview/commit/undo and verify exact preview agreement. Snap cases measure 100 repeated endpoint/midpoint queries over 100/1,000/10,000 graphical paths, checking deterministic candidate identity. The initial snap query scans linearly; it has no spatial index or native latency guarantee. Report cases separately time a full electrical projection and each CSV over 100/1,000/10,000 paired devices with individual cables and fixed two-level hierarchy. They verify all attached ends and stable output after a project round trip; these are single samples without file IO. It does not measure UI rendering, memory, nested reusable-dependency expansion, disk IO, or release capacity. Results are measurements, not pass/fail timing guarantees.

## Source boundaries

```text
crates/ecad-domain/    Electrical objects, typed identities, validation
crates/ecad-app/       Commands, session revisions, undo/redo, storage contract
crates/ecad-adapters/  Explicit JSON schema, file IO, integration tests/examples
crates/ecad-bridge/    Revisioned desktop DTOs, projections, guarded controller
apps/desktop/         TypeScript/SVG frontend and separate Tauri host
xtask/                Portable local checks and dependency-boundary enforcement
docs/                 Product plans, decisions, and implementation checklist
```

`ecad-domain` has no dependencies. `ecad-app` depends only on the domain. Adapters use serde/serde_json for explicit disk DTOs, tempfile for staged replacement, getrandom/uuid for fallible UUID-v4 generation, and SHA-256 for observed content tokens. Public APIs require rustdoc; unsafe code is forbidden throughout the workspace.

## Prototype boundaries

The experimental v10 JSON format adds mandatory bundled frame defaults to logical drawing-document metadata and authoritative sheet membership/order alongside v8 paper/view placements, portable company/contact and project metadata, and the electrical/drawing structure: anchored whole routes, separate creation presets, hierarchy, bundled revision bodies, instance mappings and local ancestry. IDs have canonical lowercase 32-digit hexadecimal encoding. `RandomIds` generates UUID-v4 identities from OS entropy; explicit constructors remain useful for imported data and deterministic fixtures. Save-as/file copies and Git branches retain project lineage. A general design-fork operation is not implemented yet.

V2–v9 files preserve electrical IDs/references and save in v10 on the next explicit save. V5–v9 retain their drawings; earlier formats receive empty drawing content and portable project defaults. Legacy projects receive a deterministic initial Drawings document, empty metadata/sheets where absent and no invented routes; they do not acquire invented definitions. Opening v1 returns `MigrationRequired(1)` rather than inventing a project ID. Convert with `migrate_v1(legacy_text, project_id)`, retaining that ID, then save to a new destination through `Document`. Old fixtures are retained and object identities/assignments are preserved. Unknown fields and future versions are rejected.

Subsystem IDs always identify placed occurrences. A separate definition lineage/revision pair identifies reusable content, with a complete typed mapping to occurrence IDs. A port exposes an existing internal terminal; every boundary crossed by a completed conductor must expose its terminal. Cable ends reference devices/subsystems and validate their cores against those endpoints. Spare ends and empty cables remain valid drafts. Traversal is iterative with a provisional 256-level guard. Reparenting, device-scope changes, and interface removal currently validate the full model; ordinary termination edits follow only the affected ancestry and cable constraints.

Undo history is session-local and limited by transaction count, not bytes. Every session receives a process-local epoch; revision guards reject tokens from other sessions but are not persistent identifiers across application restarts. New documents start unsaved. `Document::open` starts clean, and `Document::save` moves the checkpoint only after success. Undo/redo tracks saved history positions; independently recreated equivalent contents are conservatively treated as dirty.

Normal editing should use `Document<JsonFile>`. It checks observed path/content versions before replacing a file, rejects unobserved existing destinations on first save/save-as, and requires an explicit policy before discarding local edits during reload. Invalid, missing, or different-lineage replacements leave the live model, history, and file checkpoint intact. Successful reload creates a fresh session epoch and clears history. Low-level `ProjectStore`/`Session` saves remain unconditional primitives for fixtures/imports and do not provide those external-change checks.

Project file saves stage and synchronize a replacement file. Change detection is optimistic: an uncooperative writer can still race between the last content check and rename. Full locking/concurrent-writer coordination, power-loss recovery, permission preservation, path-alias policy, and multi-file transactions remain pending. See the checklist for these next requirements.

Reusable revisions currently contain one self-contained local hierarchy, with no nested reusable dependencies. Pinned internals are read-only; root name and parent are occurrence parameters. External wiring uses the root interface. Make-local unlocks the instance while retaining its source and original mapping in `local_origins`; undo restores the pin. Forgetting that ancestry is explicit and undoable. Sources remain bundled for offline use and cannot be overwritten under the same key or removed while referenced by a pin/local origin. Importing a newer revision does not update existing instances.

Placement materializes independent electrical objects and shares one immutable source body per revision. A provisional 100,000-object per-instance guard includes terminals, and callers can choose a lower budget. This is not a whole-project memory or expansion guarantee. Adoption compares source bodies by stable local ID and updates selected exact pins atomically, preserving root name/parent and surviving placed IDs. Removed connected anchors or invalid external interfaces stop adoption for explicit resolution; names/numbers never remap identities. Local variants, cross-lineage/root changes, and terminal moves between devices require a future merge/migration workflow. Preview and commit validate a temporary full-project candidate; undo stores bindings and reconstructs source content. Durable usage registries, cross-machine distribution, and promotion of arbitrary existing subtrees remain future work. See [adoption decision](docs/decisions/0006-explicit-revision-adoption.md).

Shared subsystem libraries use an independent canonical JSON schema v1, retaining every published revision and optional explicit recommendations. `LibraryPublisher<LibraryFile>` commits publication and recommendation together, with path/content version checks and a cooperative writer lock. Stale writers must reload explicitly; retained locks are reported rather than stolen. The adapter rejects destination symlinks and attempts to remove/rewrite history. These guarantees require cooperating writers and suitable filesystem behavior; crash recovery, directory durability, hard-link aliases, metadata preservation, and cross-machine coordination remain open.

Publishing changes no consuming project. `library_import` bundles an explicitly chosen revision through project commands; placement/adoption remain separate. Project undo never retracts published library history. Projects reopen from their bundled sources even when the library is unavailable. See [library publication decision](docs/decisions/0007-shared-subsystem-library.md).

`KnownProjects` provides an optional read-only, in-memory index of explicitly registered files. It reports bundled-only revisions, exact pins, and local ancestry separately, with last-observed, matching, changed, missing, or unavailable status. Failed refreshes retain labelled historical references. Canonical aliases deduplicate; copies and worktrees remain separate locations even with the same project ID. Content checks detect Git checkouts and same-length edits. Results cover only registered locations at their last explicit operation, not all consumers. Durable registries, watchers, recursive discovery, cancellation/scan budgets, and UI integration remain planned. See [known-use decision](docs/decisions/0008-registered-project-usage.md).

The electrical property registry addresses subsystems, devices, terminals, interfaces, conductors, and cables through typed references. Its original 15 electrical fields expose the current electrical object values, including read-only identities and structural membership. Shared designations and names combine by semantic identity; mixed, uniform, and absent-reference states remain distinct. Union inspection reports applicable/editable/restricted/inapplicable subsets. Bulk plans skip only unsupported pairs and reject actual restricted/invalid edits as a whole. Commands preserve identity, ownership guards, topology constraints, undo/redo, and existing serialization. Fourteen further drawing fields cover graphical styles, annotation content/position, retained vertices, view membership, named style links, and read-only terminal/route bindings and route mode. Physical wire attributes, custom schemas, and desktop inspector controls remain planned. See [property decision](docs/decisions/0009-electrical-property-contract.md).

Drawing lengths/coordinates use integer micrometres; output RGB and font-family requests are independent of personal UI/theme settings. Effective style values resolve separately through object overrides, view overrides, a linked named style, and project fallbacks. A creation preset copies an initial style link/overrides only into future objects. Changing shared styles/fallbacks updates inherited values without rewriting dependent records. Explicit values equal to inheritance retain distinct intent, including through reset, bulk editing, undo, and save/reopen. Inspectors expose effective origins separately from uniform/mixed values.

Origin layout supports annotations at their positions, graphical polylines at their first vertices, and terminal anchors at their placements. Plans report changed/unchanged objects and touching routes, reject unsupported or mixed-view selections, and commit as one transaction. Distribution holds extrema fixed, sorts coordinate ties by typed identity, and rounds fractional offsets down to integer micrometres. Visual bounds, equal edge gaps, symbols, and interactive previews remain planned. See [origin layout decision](docs/decisions/0013-origin-layout-and-group-movement.md).

The core drawing objects are renderer-independent. The desktop prototype renders paths, annotations, and terminal placements; installed-font/export parity, device-linked symbols/labels, and interactive wire/symbol tools remain planned. Whole routes reference existing conductors and view-local real-terminal anchors; geometry edits preserve electrical attachments. Moving an anchor adjusts only touching routes and retains exact layout for undo. Conflicting electrical changes require explicit route resolution. See [whole-route decision](docs/decisions/0012-whole-conductor-routes.md). Reusable electrical definitions currently reject drawing-bearing content/settings explicitly. See [drawing defaults/style decision](docs/decisions/0010-drawing-defaults-and-styles.md).

Quarter-turn rotations and horizontal/vertical-axis reflections use explicit pivots and checked integer coordinates. Fully selected connected routes transform every vertex without changing A/B order or electrical identity; singly selected ends adjust locally. Annotation positions transform while glyphs remain upright. Reports distinguish moved origins, changed selected geometry, unchanged selected records, and actually changed routes, so a stationary path origin or self-loop anchor cannot hide a real edit. All layout actions support closed preview/commit proposals. Arbitrary angles, scaling, symbol orientation, and native interaction remain planned. See [transform decision](docs/decisions/0015-orthogonal-group-transforms.md).

Snap queries rank terminal anchors, endpoints, midpoints, then grid; distance and typed identity resolve ties within each class. The tolerance is an explicit inclusive document-space radius. Grid half ties choose smaller coordinates; midpoint coordinates round down. Move previews exclude the selection and its touching routes, expose exact proposed records and candidates, and retain their original session revision. Numeric input bypasses snapping; suppression and candidate cycling are explicit. Commit does not snap again and rejects changed/different sessions. Snapping never creates electrical connectivity. M supports this preview/commit path alongside immediate numeric commands. Host pixel-tolerance conversion, hysteresis, spatial indexing, and interactive controls remain planned. See [snapping/preview decision](docs/decisions/0014-deterministic-snapping-and-previews.md).

`ConnectionReport` derives current devices, terminals, all attached conductor ends, cable memberships, and functional hierarchy from electrical identities. It preserves multiple wires, self-loops, unconnected terminals, free ends, and empty cable drafts. Immutable bundled definitions and drawing representations do not add extra physical occurrences. A report retains its originating revision; guarded current-value CSV encoding rejects stale/different-session snapshots, while deliberate historical encoding remains available. Deterministic UTF-8 CSV has stable IDs, explicit draft states, and separate cable overview/core terminations. Hierarchy names use JSON arrays inside quoted cells. No wire colours, sizes, user wire numbers, or manufacturer data are invented. Actual placed pinout labels and report UI/file dialogs remain planned. See [report decision](docs/decisions/0016-electrical-connection-reports.md) and [report/output contract](docs/connection-reports.md).

The action registry currently covers twenty-two core workflows: undo/redo, open/fit requests, guarded save, Properties, annotation/polyline/route input and creation, drawing settings, logical drawing documents, sheets/paper, project information, company/contact profiles, connection reports, palette requests, and transient-placement cancellation requests. All surfaces use the same metadata/availability/dispatch path. Default T/P/W/M/R/E/F keys are canvas-scoped; W requests a route for an existing conductor. M requests movement of selected drawing origins; R requests a quarter-turn and explicit pivot; alignment, distribution, and mirroring are registered with deliberate unbound defaults. Creating a new electrical conductor interactively remains planned. Primary-modifier bindings normalize to Control or Command for the selected profile. Direct keys, modifiers, F1–F24, and ordered sequences share one logical-key router. Empty user overrides explicitly unbind; strict canonical keymap schema v1 remains outside project data. Unknown action references and conflicts remain visible and prevent installing a partial effective map.

Typing, numeric fields, IME composition, modal UI, and control-owned input suppress application shortcuts; this first policy also suppresses Save/palette shortcuts while typing. Explicit non-keyboard actions remain available where safe, while text/control-owned undo is protected for every source. Sequences expose pending/deadline feedback, preserve initial targets, and cancel on Escape, timeout, target changes, or invalid continuation. Hosts must provide a shared monotonic input-ID stream, marking native repeats and reusing IDs for duplicate native/web deliveries. Full native menu/accelerator integration, reserved-key/layout diagnostics, physical-position keys, keymap editor/preference-file lifecycle, action-ID migrations, and actual palette/tool UI remain planned. See [action decision](docs/decisions/0011-action-registry-and-keymaps.md).

Drawing sheets and reusable table-based templates are included scope. Their first foundation is File → Project information: eight portable, multiline company/contact and project fields with exact history. File → Profiles now stores reusable company/contact defaults separately and previews selected copies into projects. Drawing → Sheets now manages paper, margins, numbering/order and one optional scaled view placement, with direct template-backed sheet editing, paper Fit and transient per-sheet cameras. The standard frame has eight live fields; custom cells and draft/cover/contents templates remain planned. See [sheets and templates](docs/sheets-and-templates.md).

The target drawing workflow is now Project → Documents → Sheets: actual drawings are edited directly on template-backed CAD pages. The bounded desktop now follows this sheet-first editing contract; symbol/component/footprint editors will have separate workspaces. See [the corrected architecture](docs/decisions/0030-sheet-first-project-documents.md).

Multi-project work has started with a headless workspace owner and real-file isolation/lifecycle tests. The desktop remains single-project until the host/bridge/explorer migration. See the [implementation task list and rework map](docs/workspace-and-sheet-implementation.md).

Logical drawing-document ownership and schema v10 frame defaults are implemented, with metadata, ordering, sheet transfers and deterministic legacy upgrades. Every sheet resolves the bundled standard-v1 border and eight live drawing-info fields in numeric review/paper display. The bridge exposes document groups; document-management UI, custom templates and multi-project explorer integration remain pending.

The desktop edits on template-backed sheets with source-unit CAD tools, paper Fit, transient per-sheet cameras and explicit reviewed attachment of retained unplaced views. Shared source views stay shared with a visible warning. See [the editable-sheet decision](docs/decisions/0034-editable-sheet-workspace.md) for coordinate, targeting and remaining UI boundaries.

The default explorer now shows project → documents → sheets, with a separate Electrical mode for components and drawing objects. Page navigation/context menus use stable sheet identities; modes retain expansion and selection. The desktop still opens one project at a time. See [explorer scope and checks](docs/decisions/0035-project-page-and-electrical-explorers.md).
