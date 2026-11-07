# Hierarchy, scale, and shared revisions

Status: project-local containment, explicit terminal exposure, cable grouping, and a depth guard are implemented. Project-bundled immutable revisions, independent occurrences, and make-local ancestry are also implemented. Explicit adoption of exact pins is implemented. External publication, known-use discovery, and local-variant merging below remain planned. See [decision 0005](decisions/0005-reusable-subsystem-revisions.md).

## Target and object boundaries

Target equipment systems roughly at the scale of a car. This includes a complete connected system of devices, connectors, cables, and individual conductors. Building services are outside the initial target. Do not assume automotive-specific standards or vehicle manufacturing workflows solely from the size comparison.

An individual conductor is the finest electrical design element in this scope. Components/devices are the highest ordinary equipment objects before hierarchical composition. Terminals and contacts are connection anchors, not another equipment hierarchy. Cables group conductors and their applicable shield/pair structure. Subsystems compose components and other subsystems through explicit interfaces.

Electrical object relationships and drawing groups are separate. A graphical group does not become a reusable electrical subsystem merely because it can be moved as a whole.

## Scale and bounded work

Hierarchy makes focused editing and reuse possible; nesting alone does not reduce computation. Avoid flattening every nested occurrence and rendering or validating the full expanded project on each edit.

Use explicit subsystem interfaces to support local connectivity queries. Cache immutable definition-derived data by revision, and instance-derived data by the instance/override state it depends on. Recompute affected regions and invalidate dependent results explicitly. Unchanged shared geometry or definitions can be reused without sharing mutable per-instance electrical state.

Render visible views and query relevant objects. Prepare hierarchy summaries for navigation without forcing every sheet to load into the UI. Whole-project validation and reporting still need complete, logically consistent results; incremental or lazy processing must not hide unresolved errors or omit off-screen connections.

Model physical containment as acyclic. Reusable definition dependencies must also reject recursive cycles. A definition can be reused many times without being permitted to contain itself indirectly. Reference links between devices are not containment and may legitimately form electrical cycles.

The initial local hierarchy has a provisional 256-level guard, iterative traversal, and explicit failure diagnostics. Tests cover the boundary and failed reorganization; the scale probe measures depth and unrelated block growth. This does not establish general expanded-instance or memory limits. A shallow hierarchy can still expand enormously when reusable definitions are introduced. Add cancellation and those broader budgets as that work develops; do not silently truncate results.

Define representative fixtures by conductor count, terminal count, devices, repeated subsystem instances, nesting depth, drawing geometry, and assets. Measure document opening, local edits, navigation, validation, memory, and undo growth. Select numeric targets before making performance claims.

Probe usable capacity through representative workloads, controlled growth sweeps, and bounded stress/soak runs. In particular, measure whether adding unrelated subsystems increases the cost of a local edit. See the [scalability experiments](testing-strategy.md#scalability-experiments) for workload dimensions, correctness checks, environment records, and baseline policy.

## Definition, revision, and occurrence

| Identity | Purpose |
| --- | --- |
| Definition ID | Identifies a reusable subsystem lineage independently of its name or location. |
| Revision ID | Identifies an immutable version of that definition and its pinned dependencies. |
| Instance/occurrence ID | Identifies one placed use in a particular project and hierarchy. |
| Definition-local object ID | Identifies a terminal, component, or interface element across compatible revisions. |

The same definition can appear in two projects, or several times within one project. Each occurrence has distinct electrical identity even when it shares the same definition revision. Renaming a subsystem must not change its identity. Typed IDs use canonical 128-bit hexadecimal encoding; complete per-kind mappings relate definition-local IDs to independently placed IDs. Nested reusable dependencies are currently rejected.

Each project records the resolved revisions and enough dependency/asset data to reopen without a central library. A mutable "default" pointer may recommend a revision for new adoption; it is not a substitute for a project's revision lock. Existing occurrences never silently follow the latest default.

Separate occurrence parameters, such as a permitted designation or configured option, from structural edits to a reusable design. Do not treat arbitrary changes to internal wiring as an unexplained parameter override. Define supported overrides and variant behavior before implementing update propagation.

## Implemented reuse boundary

A revision contains one self-contained local hierarchy. Placement materializes independently editable electrical identities and binds them to that immutable source. Pinned internals are locked; root name and parent are occurrence parameters, and external conductors must use root-exposed terminals. One compound command places the complete occurrence, with exact rollback and undo.

Make-local removes the lock while retaining the original source and ID mapping, including IDs of subsequently deleted internal objects. This is historical ancestry, not a promise that the occurrence still matches its source. Forgetting ancestry is a separate undoable operation. Bundled sources cannot be overwritten or removed while a pin or local origin references them. Importing another revision changes no existing occurrence.

Placement currently caps each occurrence at 100,000 objects including terminals, with a caller-selectable lower limit. This provisional guard is not a project-wide memory budget. Definitions share an immutable body, but occurrence content is materialized; whole-project validation checks it against the source. Ownership indexes keep ordinary mutation guards local; placement and full validation still perform broad scans. The scale probe covers 1/10/100 repeated instances and does not establish production capacity.

## Editing and publishing workflow (planned)

When changing reusable content, make its scope explicit. Offer:

1. **Keep in this project:** Create a project-local revision or variant with recorded ancestry. Identify whether it affects only the selected occurrence or selected/all matching occurrences in the current project.
2. **Publish shared revision:** Create a new immutable library revision, retaining the previous revision. Offer making it the recommended/default revision for future adoption.

Show known usage when making this choice and explain which occurrences in the current project will change. Do not ask after every keystroke: capture the edit scope when entering reusable-content editing, and review the accumulated change when publishing or completing the edit. The exact UI flow remains open.

Checking for other instances informs the choice; it is not the sole safeguard. A definition marked shared may have unknown consumers even if no other use is currently indexed. Do not overwrite its published revision in place.

Publishing a library revision and updating projects are separate operations. The publish operation must either complete coherently or report failure. Project undo restores project edits; it must not pretend to erase a shared revision that another project may already have adopted. Define publication history/reversion separately, for example by publishing a subsequent revision or changing the recommended revision.

## Known-use discovery

Maintain an optional local index of registered projects and their definition/revision references. Refresh it when projects open/save or through an explicit rescan of configured project/library roots. Read project metadata without changing consuming projects. Remote repository discovery and organization-wide catalogues are later capabilities, not requirements for local use.

Show discovery coverage and freshness: "used in these known projects" rather than "used nowhere else". Offline machines, unregistered directories, archived copies, and other branches may contain additional uses. The index is rebuildable metadata and must not be required to open or edit a project.

Use stable identities to find uses, not matching file paths or display names. Registering a moved project and distinguishing a copy from another working tree require an explicit policy to avoid misleading usage counts.

## Adopting updates

The headless implementation provides a typed source-body change report and a validated adoption plan for a selected exact pin. Matching local IDs retain placed IDs; additions require fresh IDs, and removed IDs cannot be recycled for different local objects. Selected plans may execute in one compound transaction. Root name/parent remain occurrence parameters. Commit rechecks the expected binding and all resulting connections, including newly added external references. Undo restores the previous revision, mapping, and content. There is no file-format change beyond v4. See [decision 0006](decisions/0006-explicit-revision-adoption.md).

Adoption currently requires one lineage, a stable definition-local root, and stable device ownership of surviving terminals. It rejects local variants rather than overwriting their edits. Interface conflicts return typed model errors without mutation; reconnect/disconnect explicitly, then create a fresh plan. An interactive resolution panel, local merge, discovery/default notifications, and publication remain planned. `RevisionChanges::between` can inspect source changes even when an adoption plan fails validation.

When a newer shared revision is available, a consuming project can compare, accept, defer, or stay pinned. Do not silently write changes into closed projects. Reopen/update checks can identify available revisions without forcing adoption or requiring network access.

Before applying an update, show changes to interfaces, terminals, components, wiring, library dependencies, and local variants. Preserve compatible instance identities and parameters. Stable interface/terminal IDs support matching; position in a list or equal display names are not sufficient.

Removed or incompatible interface elements require explicit resolution of external connections. Do not reconnect a removed terminal to a different terminal with the same number automatically. Local structural differences require a deliberate rebase/merge or continued pinning, not automatic overwrite.

Update selected project occurrences as a coherent command with validation, undo/redo, and a reviewable Git diff. Definition revisions are independent of Git commits: Git stores project/library history, while explicit revision IDs identify the reusable content resolved by a project.

## Acceptance scenarios

- Two projects use revision 1 of one subsystem. A local change in project A leaves project B and the shared default unchanged.
- Project A publishes revision 2 and marks it recommended. Project B still opens using revision 1 and can review an available update.
- Project B accepts revision 2, preserving compatible occurrence and terminal identities. One undo restores its previous adopted revision and connections.
- An update removes an externally connected terminal; adoption stops for explicit resolution instead of silently remapping it.
- The known-use index is missing or stale; editing and publication remain safe and do not claim complete discovery.
- A definition dependency cycle is rejected. Deep valid nesting and wide repetition are handled within explicit limits, with cancellation and useful diagnostics.
- Repeated instances share reusable definition data but retain independent assignments, designations, and external connections.
- Open, validate, save, and report the complete representative equipment fixture without dropping hidden or unloaded subsystem contents.

## Next decisions

Choose the representative system and scale fixtures; extend occurrence parameters and structural variants; define nested dependency locks and local-variant update matching; and decide the first shared-library storage and known-project indexing workflow. Defer automatic cross-project propagation and sophisticated variant merging until the simpler explicit update workflow works reliably.
