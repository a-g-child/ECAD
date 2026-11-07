# Testing and platform strategy

Status: intended development requirements with a working headless baseline. `cargo xtask quick`, `full`, and `scale` are implemented; core behavior, persistence, local Git interoperability, doctests, and a bounded scale probe run locally. A bounded Tauri native macOS build/launch and Chromium workflows against the real Rust bridge are verified. Automated native interaction and Linux/Windows execution remain pending. See the [implementation checklist](implementation-checklist.md).

## Local verification is the baseline

Developers must be able to build and verify the project locally. GitHub can hold the source and later run the same checks, but it must not be the only place tests run. Tests must not require pushing code, a GitHub account, or a remote repository.

Once dependencies and required toolchains are installed, routine unit, regression, and integration tests should run offline. Network-specific tests, if eventually needed, are explicitly opt-in and separate from the ordinary suite.

The current portable Rust runner is `cargo xtask quick`, `cargo xtask full`, and `cargo xtask scale`. A comprehensive `native` interaction/release-matrix profile remains planned and are not reported as passing. The runner returns failure for unavailable profiles or failed checks. The full profile currently tests the complete headless slice, not future CAD features.

The same underlying checks must be callable directly for debugging. Keep tooling reproducible through a pinned Rust toolchain, frontend lockfile, documented system dependencies, and explicit test data. Do not run every expensive UI or packaging check on every keystroke or make push hooks the only enforcement mechanism.

## Test responsibilities

| Kind | Purpose | Representative cases |
| --- | --- | --- |
| Unit | Verify focused rules and algorithms without a running application | Terminal mapping, style precedence, property applicability, geometry transforms, snap tie-breaking |
| Integration | Verify real behavior across module and IO boundaries | Command to document to save/reopen; Git checkout to reload/validation; asset and library resolution |
| Regression | Preserve a reproducer for a substantive previously fixed defect | Branch lost during route simplification; mixed selection incorrectly edits unsupported objects |
| Frontend/component | Verify reusable controls and interaction state | Mixed values, focus retention, inspector applicability, keyboard actions |
| Native end-to-end | Verify representative workflows in the actual desktop application | Wire editing, file dialogs, clipboard, shortcuts, repository operations, notification/layout stability |
| Scalability and performance | Find usable operating ranges, growth trends, and engine bottlenecks | Local edits as the total project grows; repeated hierarchy expansion; rendering density; memory and undo growth |

Regression describes why a case exists, not a separate technical layer. A regression may be a unit, integration, or native test. Keep its fixture and explanation near the owning behavior, with an issue reference when available.

## Critical behavior to protect

- **Electrical integrity:** Stable device/terminal IDs, explicit branch topology, hierarchy mappings, and no accidental connectivity from graphical crossings.
- **Editing transactions:** Rejected commands leave state and history unchanged; successful commands undo and redo coherently, preserving IDs and references.
- **Bulk properties and styles:** Correct applicability, mixed values, per-property inheritance, explicit overrides, and reset behavior.
- **Structured geometry:** Whole routes survive edits and round trips; moving equipment preserves intended endpoints and unaffected geometry.
- **Persistence:** Stable text output, schema migrations, portable assets, external-change detection, and recoverable multi-file saves.
- **Git:** Real temporary repositories exercising status, commits, branches, dirty-document handling, conflicts, and reload of a validated model.
- **Connection labels:** Contact identity and terminal bindings survive orientation changes, assignments refresh, and missing mappings produce diagnostics.
- **UI stability:** Long values, errors, progress, and notifications do not incidentally move pane boundaries or lose active edits.
- **Implemented headless actions:** Registry coverage/growth, all-profile primary modifiers, explicit unbinding/remapping, unknown-reference/scope/duplicate/prefix diagnostics, Unicode/function-key import, protected text/modal/IME/control input, repeat/replay handling, pending sequence timeout/Escape/mismatch/clock/context behavior, and original-anchor capture. Real document tests cover all invocation sources, creation requests/replies, stale contexts, typed validation/save conflicts, malformed application requests, native repeat policy, checkpoints, and duplicate delivery. These prove the core host protocol; native normalization/focus/OS-reserved behavior still requires desktop testing.
- **Keyboard control (broader native/UI coverage):** Registry coverage, consistent action invocation, context and target resolution, remapping, sequence conflicts, repeat behavior, typing/composition protection, and prevention of duplicate native/frontend dispatch. See [Keyboard and action model](keyboard-actions.md).
- **Implemented adoption contracts:** Local tests cover stable matching, rejected terminal ownership changes, removed connected anchors, lost root/outer exposure, new-ID collisions, stale plans, preview/commit revalidation, selected-instance compound undo, and offline independent two-project adoption. Repeated-instance probes time one preview/commit/undo at 1/10/100 instances without asserting timing thresholds.
- **Implemented shared-library contracts:** Strict library schema/golden round trips, immutable keys, recommendation validation/reversion, coherent publication, stale/no-op writer rejection, cooperative lock contention and cleanup, history deletion/rewrites, IO failure and invalid reload, path tokens/parent aliases, and offline independent consuming projects. Unix destination-symlink coverage is conditional; Windows behavior still needs a native run.
- **Implemented first desktop slice:** Strict bridge-v1 requests, captured-revision/target rejection, exact i64/u128 property/identity and host input-ID handling, full union inspection, real bulk edits/undo/redo/save/reopen/conflicts, and protected host lifecycle. Nineteen Chromium workflows run against the real Rust bridge and temporary saved files: edit/validation/save/history, mixed applicability/keyboard/pan-zoom, long/multiline/markup-like values and fixed pane geometry/focus, electrical reports/external save conflicts, connected-group preview/cancel/commit/save/history, pointer snap feedback/cycling/suppression, directional/stroke-aware marquee selection, modifier wheel controls/cursor-centred zoom, direct drag release/cancellation with delayed real bridge responses, exact pivot-based rotation/mirroring with invalid input, cancellation, fixed bounds and real saved geometry, and annotation creation/selection/save/history/cancellation with protected control navigation, multiline creation/property/history, blank-line selection/Fit, shared multiline captions/reports, and retained path numeric/draft/history/minimum-width controls, delayed click-intent preservation and cancellation. Bridge regressions also cover host timing across frontend reloads and invalidated/superseded/stale move proposals. Mac native compile/build and webview-to-core launch are verified separately. These tests do not validate native file-dialog interaction, OS accessibility, font/export parity, Windows/Linux, or large-document rendering capacity.
- **Implemented electrical reports/outputs:** Complete terminal/link coverage, self-loop A/B multiplicity, free endpoints and empty cable drafts, actual core terminations versus hierarchy overview endpoints, explicit occurrence identity, omission of bundled-only bodies/drawing duplicates, refreshed names/numbers/assignments and hierarchy, stale/current/historical exports, immutable snapshots, no document/history/file mutation from report actions, Unicode/quote/comma/newline/path-label escaping, independently authored CSV goldens, and byte-stable output after save/reopen/undo. CSV fixtures use LF checkout rules. Release probes separately measure full projection and each CSV through 10,000 paired devices with fixed depth and one sample per case. Placed live labels, native report/export controls, physical wire data, and a physical build from documentation remain unverified future work.
- **Implemented orthogonal transforms:** Exact pivot and axis conventions; full-span offsets/overflow; four-turn and mirror cycles; preserved bend/collinear waypoint order, A/B bindings and style intent; fixed-origin path changes and pivot-anchor self-loops; partial connections, free-end retention, and independent views; atomic overflow/rollback; no-op redo retention; exact generic preview/commit and action operation/context/revision guards; v6 save/reopen/checkpoints. Release probes cover fixed-degree group half turns through 10,000 routes and separate annotation-position quarter-turn preview/commit/undo through 2,000 objects. Glyph orientation remains upright; symbol orientation, arbitrary angles/scaling, and native interaction remain future work.
- **Implemented snapping/previews:** Source priority, exact distance and identity ties, coincident target cycling, inclusive circular/zero tolerance, negative grid half ties and offset origins, full signed/tolerance spans, rounded midpoints, view/exclusion guards, self-snap suppression, numeric/suppressed/unsnapped movement, invalid-candidate and invalid-winner failures, immutable projections matching commit, cancellation, stale/different-session/no-op guards, redo/checkpoints, save/reopen, and shared M preview/commit with replay/context checks. Release probes document linear endpoint/midpoint scanning through 10,000 paths. Pixel tolerance is integrated in the prototype and browser-tested; native interaction, hysteresis, spatial indexing, and measured pointer/UI latency remain future work.
- **Implemented origin layout:** Mixed numeric movement, explicit-coordinate alignment, fixed-extrema distribution with deterministic identity ties and fractional micrometre rounding; full signed-coordinate span and interior-point overflow; simultaneous shared-route movement; independent views and exact style intent; unsupported/missing/mixed-view selections; collapsed-path rejection; malformed union snapshots; rollback/no-ops/redo/checkpoints; captured action replies and typed parameter mismatches. Schema v6 remains unchanged. Release probes separate origin-plan preflight from commit/undo through 2,000 annotations and measure fixed-degree group anchor movement through 10,000 unrelated routes.
- **Implemented whole routes:** Retained identities through vertex/segment edits; exact A/B attachments; independent views; local shared-anchor and self-loop movement; no bend accumulation on repeated up/down moves; exact undo, failed-batch rollback, and saved checkpoints. Electrical retermination/deletion and revision adoption cannot invalidate routes silently. Strict v6 schema and v5 upgrade tests cover malformed bindings and geometry. W action input references an existing conductor and uses a separate route creation preset. Release probes grow unrelated routes to 10,000 while the moved anchor touches two, checking full undo and round trips. Crossing presentation and actual interactive tools remain future work.
- **Implemented drawing defaults/styles:** Per-property object/view/named/project precedence, creation-only presets, equal-valued override intent, reset, mixed-origin inspection, graphical/electrical bulk applicability, focused style diffs, localized history/checkpoints, role/reference/in-use guards, and explicit rejection of drawing loss in reusable bodies. Golden v6 fixtures and v2–v5 upgrades preserve electrical identity and existing drawings. Release probes cover style propagation and bulk drawing edits through 2,000 objects without timing gates. Native rendering/font resolution remain unverified future work.
- **Implemented electrical properties:** Registry coverage and reference kinds, union/mixed/uniform/absent values, deduplicated selections, restrictions, terminal/interface uniqueness, topology constraints, pinned occurrence parameters, make-local compound undo, stale/deleted targets, failed-prefix rollback, saved checkpoints, and canonical reopen. Bounded release probes measure mixed selection sizes through 2,000 objects; results are exploratory single samples, not timing gates.
- **Implemented known-use discovery:** Headless read-boundary tests forbid writes and verify unregistered/failed operations. File regressions cover pin/local/bundled-only summaries, partial scan coverage, labelled historical hits, missing/invalid/replaced/moved files, copies, canonical aliases, same-length edits, and a real isolated Git checkout. Unix symlink checks are conditional.
- **Shared subsystem revisions (broader planned coverage):** Two-project fixtures verify local edits, immutable publication, pinned dependencies, explicit update adoption, interface compatibility, and stale known-use indexes. Include deep and widely repeated hierarchies, cycle rejection, and resource-limit diagnostics. See [Hierarchy, scale, and shared revisions](hierarchy-and-revisions.md).

Use generated operation sequences or property-based tests where they expose meaningful combinations, such as command/undo sequences and geometry invariants. Record failing seeds or minimized cases so failures can be reproduced.

## Fixtures and failure handling

Keep small versioned fixtures representing valid, intentionally incomplete, invalid, and older-schema projects. Test known expected semantics as well as round trips: a serializer and loader can share the same bug and still round-trip successfully.

Use temporary directories and isolated repositories. Isolate Git identity, configuration, hooks, locale, and environment so developer settings cannot change the test outcome. Do not touch personal repositories or user library data. Clean up resources and support running tests in parallel without shared mutable fixtures.

Inject failures at save/recovery boundaries and verify that reopening yields a coherent project or an explicit recoverable state. Test cancellation and stale background results rather than relying on timing-sensitive sleeps. Use explicit synchronization and bounded waits for asynchronous tests.

Golden files are useful for stable project serialization, exchange schemas, and selected scenes. Review intentional changes; do not regenerate expectations automatically to hide failures. Prefer semantic geometry and layout assertions over exact cross-platform screenshots. Use targeted platform-specific image comparisons where rendering differences are expected and meaningful.

## Local check profiles

| Profile | Intended contents | When |
| --- | --- | --- |
| Quick | Formatting, lint/type checks, focused unit/component tests, directly affected regression cases | During an implementation change |
| Full | All core/frontend suites, local adapter integration tests, regression fixtures, doctests, documentation and bridge checks, supported-host build | Before treating a change as ready for review/backup |
| Native | Actual desktop workflow tests and platform-specific IO/UI checks | For affected host integration/UI work and release candidates |
| Scale | Reproducible workload sweeps, performance baselines, bounded stress/soak runs, and correctness checks | During engine development, after relevant changes, and when selecting supported limits |
| Release matrix | Full and native checks on each supported OS/architecture, plus package installation and launch checks | Before claiming a release supports that target |

Run the relevant profile locally before pushing work intended to be validated. Documentation-only edits need document checks, not an application build. Clearly identify any check that could not run and why; do not report an untested platform as passed. Detailed performance suites can run separately when performance-sensitive behavior changes.

## Scalability experiments

Use scalability testing to discover what is feasible and comfortable to use, then set supported limits from evidence. Do not treat opening a huge synthetic file as proof that it can be edited productively. The scale profile must run locally without GitHub or network services.

Maintain three workload families:

- **Representative equipment:** A realistic car-scale project with varied devices, connectors, cables, spare cores, subsystem interfaces, sheets, labels, and image assets. Establish its counts from a worked design rather than equating all vehicles with one size.
- **Controlled growth:** Seeded generators scale one dimension at a time, then selected combinations. Include unique devices/conductors, repeated definitions versus expanded occurrences, hierarchy depth and fan-out, cross-boundary connections, visible drawing density, route bends/crossings, large selections, styles, labels, and assets.
- **Stress and soak:** Explicitly bounded cases beyond the intended working range, plus long edit/undo/save/open sessions. Check cancellation, resource-limit diagnostics, and memory recovery rather than requiring arbitrary extreme projects to meet interactive targets.

Version generator recipes, seeds, expected object counts, and scripted operations. Use increasing sizes with several points per dimension; compare trends and identify cliffs instead of relying on a single large fixture. Repetition-heavy projects must be accompanied by unique-definition cases so cache reuse does not conceal poor behavior on varied designs.

### Measurements

| Area | Measure |
| --- | --- |
| Open/navigation | Time to usable first view, hierarchy expansion, subsystem/view switching; report any pending load work separately |
| Interaction | Input-to-visible-preview latency, pan/zoom frame times, selection and hit-testing latency, time to commit a drag or bulk edit |
| Engine | Command validation/application time, affected objects visited, cache invalidation/rebuild work, complete-project validation/report time |
| Persistence/Git | Save/reopen time, serialized size, local status/diff time, branch reload time, and number of unrelated files changed |
| Resources | Peak and steady-state memory, core/frontend process usage where measurable, cache size, undo-history growth, and memory after document closure |
| Background work | Cancellation response, progress responsiveness, and foreground latency while indexing/validation/export runs |

Measure the headless engine separately from the native UI, then trace representative end-to-end operations so an improvement in one layer does not conceal latency elsewhere. Distinguish cold and warm application caches and document what is controlled; do not claim a cold filesystem cache unless the test environment actually establishes one.

For interactive workloads report latency distributions and frame-time outliers, not only averages or FPS. For expensive batch operations report repeated-run elapsed time and variability. Record sample counts and avoid presenting tail percentiles as meaningful when too few observations were taken.

A key architectural experiment is to enlarge unrelated subsystems while keeping the visible view and edited region fixed. A local edit should not start doing full-project work merely because unrelated content grows. Compare objects visited and invalidation counts as well as wall time. Also test changes with genuinely broad impact, such as a widely used style or shared-definition update: their costs must remain explicit rather than hidden by stale caches.

### Correctness under load

After scripted edits, verify connectivity, terminal mappings, instance counts, styles, and save/reopen results against expected invariants. Sample complete undo/redo cycles and compare semantic state. For selected generated projects, compare incremental results with an independent full recomputation/reference path where practical. Off-screen or lazily loaded data must not disappear from validation and reports.

Every run has time/work/memory guards where measurable. Exceeding a guard is a reported failed, interrupted, or out-of-range result, never an incomplete result presented as success. Preserve failing seeds and useful diagnostics. Resource-limit rejection must leave the project usable and saveable; an out-of-process harness can contain deliberately extreme runs.

### Baselines and decisions

Benchmark optimized builds with recorded revision, toolchain, OS/architecture, webview version, hardware, display settings, fixture version, seed, and relevant cache conditions. Use controlled repeated runs and investigate machine load before calling a timing change a regression. Compare like environments; a single developer machine cannot establish every supported platform's capacity.

Keep machine-readable results and a compact human-readable comparison of latency, memory, and workload size. Establish a reference hardware class and provisional responsiveness/resource budgets after the first prototype, then tighten them through actual use. No numeric capacity or latency guarantee is asserted at this planning stage.

Initially use performance changes as investigation signals. Introduce stable regression gates once baseline variance is understood, combining relevant absolute user-facing budgets with relative changes. An optimization must preserve correctness and be supported by a profile or a repeatable measurement; avoid adding caches, threads, or complexity on speculation.

Use the results to choose or revise indexing, route algorithms, scene generation, serialization boundaries, history storage, and explicit hierarchy/resource limits. Keep small reliable cases in ordinary regression checks; leave expensive sweeps and soak runs in the separate scale profile.

## Platform matrix

Target all three desktop families from the beginning; finalize minimum versions and CPU architecture coverage before selecting release tooling.

| Target | Areas needing native verification |
| --- | --- |
| Linux | Chosen distributions and system dependencies, WebKitGTK behavior, selected X11/Wayland coverage, fonts, clipboard/dialogs, packaging |
| Windows | WebView2 behavior, path and file-lock handling, keyboard conventions, display scaling, installer and asset resolution |
| macOS | WKWebView behavior, Command-key conventions, application menus, dialogs/clipboard, display scaling, bundle and asset resolution |

Tauri uses platform webviews rather than one bundled engine; see its [webview reference](https://v2.tauri.app/reference/webview-versions/) and [platform prerequisites](https://v2.tauri.app/start/prerequisites/). Browser-based frontend tests help but do not replace native host tests. Select automation after a prototype on all three targets; consult the current [Tauri testing guidance](https://v2.tauri.app/develop/tests/) rather than assuming one driver covers every setup.

Cross-compilation and compile checks detect some compatibility issues but cannot demonstrate runtime behavior. A local machine verifies its host; other platforms need actual native environments, whether developer machines, appropriate VMs, or supplementary CI. Tests and setup instructions must remain runnable outside hosted CI.

Cross-platform fixtures should cover spaces and non-ASCII paths, case differences, line endings, relative asset paths, font availability, DPI/zoom, numeric precision, and portable Git serialization. Specify project-file newline and numeric conventions independently of host defaults.

## Completion standard

A feature is ready when its contract is documented, relevant automated behavior and failure tests pass locally, and platform-specific effects have been checked on the affected targets or explicitly recorded as unverified. Substantive bug fixes normally include a failing reproducer that passes after the fix.

Prioritize coverage of invariants and important workflows over a single coverage percentage. Avoid tests that simply mirror implementation or demand maintenance without protecting observable behavior. Track platform validation by revision and environment; passing an earlier version does not certify a later package.
