# Multi-project workspace and sheet-first implementation tasks

Status: implementation started with a headless ProjectWorkspace owning independent project controllers. Logical drawing-document ownership and schema-v9 upgrades are also implemented. The desktop still has one DesktopBridge/project and the prototype Model view/paper split. Checked items are executable foundations; UI migration is not yet complete.

## Target and boundaries

One application workspace can hold several projects. Each root owns its electrical model, drawing documents/sheets, metadata, undo/redo, dirty state, save binding and Git context. The electrical component/functional explorer is scoped to the selected project. The default explorer shows project roots and their document/page hierarchy. Main-area sheet tabs are keyed by runtime open-project handle plus stable document/sheet identity, never by visible names or project lineage alone.

Projects may share ProjectId after file copy/save-as/Git branching; they must still have independent open sessions. Use a process-local workspace handle per open project and normal session/revision guards. Keep that handle stable across reload while renewing the document revision epoch. Active selection is workspace state; delayed callbacks retain their captured project rather than targeting whichever project is active later.

Every actual drawing is edited directly on a CAD-template sheet. Cover, generated drawing-list, overview and detailed wiring pages share this contract. Electrical hierarchy and off-sheet connectivity remain independent of document organization. Library symbol/component/footprint/template editors are different workspace kinds. See [decision 0030](decisions/0030-sheet-first-project-documents.md).

## Rework map

| Existing area | Required change / risk |
| --- | --- |
| Native Host(Mutex<DesktopBridge>) | Own a workspace with multiple project contexts. Opening adds/focuses a project rather than replacing the current document. |
| DesktopBridge.document and transient tools | Move document ownership into ProjectWorkspace; do not retain a second authoritative document inside each bridge context. Route tools/projections by captured workspace-project handle and revision. Project previews, drafts, reports and inspector selections must never leak into another project. |
| Request/response DTOs and transport | Add an explicit project envelope to project-scoped operations/projections; keep workspace operations separate. Unknown/closed targets fail, including delayed replies. Keep one monotonic host input stream. |
| Native open/save-as choosers | Capture target project/revision before awaiting. Canonicalize locations/aliases before deduplication and keep Save-as from targeting another open project. Do not infer a target from later active state. |
| Close/quit protection | Check every open project, including inactive/unsaved projects. Close one project independently; preflight Close all/Quit before dropping any unresolved sessions. |
| DrawingSheet/project.sheets v8 | Add logical drawing-document identity, ownership, metadata, order and template defaults. This is distinct from Document<S>, the existing project session/save controller. |
| Legacy view/paper migration | Retain stable IDs, geometry/styles/connectivity, numbers and transforms. Wrap unplaced views into template-backed sheets. Explicitly preserve/reconcile views shared by several old paper placements. No fresh IDs on each load. |
| main.ts global snapshot/selection/view and shell tree | Separate workspace selection from per-project/per-sheet state. Default to Projects/pages; provide selected-project electrical structure as another explorer mode. |
| DrawingCanvas / PaperSurface | Replace separate model editing and read-only paper with a unified editable sheet. Validate inverse input transforms before snapping/commands; preserve exact micrometre edits and paper-space template geometry. |
| Sheet tabs and tool focus | Restore each sheet's camera/selection. Bind drafts to original project/sheet; initially block switching while a tool is unfinished if draft suspension is not implemented. No implicit retarget/commit. |
| Properties, defaults and bindings | Distinguish workspace preferences, reusable personal profiles, project metadata, document metadata and sheet-local values. Support template defaults and explicit overrides with visible origin. |
| Profiles / library services | Per-user services stay workspace-wide; project adoption/copies remain explicit. Avoid treating a global profile save as a project undo action. |
| Action registry / undo/save/shortcuts | Resolve the captured project before dispatch; retain shared registration and protected text/IME behavior. Each project has its own history; one sheet tab must not create its own electrical undo model. |
| Reports/Git and external changes | Scope cached rows, exports, repository paths, checkout/reload and save conflicts to their original project/revision. |
| Projection cost / memory | Build/render the active sheet scene and load inactive electrical trees on demand. Scope caches by project/revision; account for accumulated history across open projects before claiming capacity. |
| Browser harness / fixtures / native smoke | Support at least two real temporary projects, same-lineage copies, multiple sheets and stale delayed callbacks. Replace superseded model/paper assertions rather than retaining the old workflow as a success criterion. |

## Delivery sequence

### 1. Independent open-project sessions — first code slice

- [x] Own several Document<S> controllers in opening order with stable runtime handles and transient active selection.
- [x] Keep electrical data, history, dirty/checkpoint state, observed store versions and save bindings isolated.
- [x] Require an explicit target for edits/actions/save/reload/close; validate captured revision before mutations/lifecycle actions.
- [x] Focus an already-open normalized location without discarding edits; permit same-lineage projects at different locations.
- [x] Guard Save-as against reserved locations and keep the old binding on failure.
- [x] Protect dirty projects on close, expose every dirty project for quit preflight, and retain a shared action input sequence across project switches.
- [x] Verify with real local project files; add an executable headless workspace example.
- [ ] Integrate this owner with the host/bridge; no desktop multi-project support is claimed by the headless slice.

### 2. Logical drawing documents and migration

- [x] Add stable logical drawing-document IDs, metadata, ordered sheet ownership and undoable moves/renames.
- [x] Keep document numbers unique per project and preserve the existing project-wide sheet-number rule without renumbering.
- [x] Add strict v9 ownership persistence and deterministic default-package upgrades for v2–v8; v1 remains explicit.
- [ ] Add reviewed migration of unplaced views into template-backed sheets when the frame/editor is ready.
- [x] Preserve shared and unplaced view references unchanged during ownership upgrade, without electrical copies.
- [ ] Expose shared-view editing implications in the unified sheet workspace before any explicit materialization policy.
- [x] Test ownership migration round trips, stable IDs, rollback, source geometry and strict membership validation.
- [ ] Add document default-template references and their migration/validation with the actual frame implementation.

### 3. Template-backed sheet editor

- [ ] Add a basic bundled CAD frame and drawing-info fields with mandatory resolved template/default.
- [ ] Open/create drawings directly on sheets; remove the normal free Model view/paper-preview switch.
- [ ] Unify paper drawing, hit testing, selection, marquee, drag, snapping, numeric placement and navigation through validated coordinate transforms.
- [ ] Retain blank cover/content pages and independent definition-editor contexts.
- [ ] Verify transformed pointer edits, paper/template changes, undo/redo and exact save/reopen.

### 4. Multi-project host and scoped bridge

- [ ] Give project-scoped requests/replies an explicit runtime project target; separate workspace requests.
- [ ] Migrate native file choosers, profile service ownership, dirty close/quit checks and browser harness.
- [ ] Resolve canonical paths/aliases consistently and prevent duplicate opening/save-as collisions.
- [ ] Reject late operations after close/reload; invalidate or retain previews only in their original context.
- [ ] Verify two same-lineage projects with identical electrical IDs and independent dirty/history/save states.

### 5. Page explorer, electrical explorer and tabs

- [ ] Show multiple project roots with document groups/sheets; elide only a default group node when useful.
- [ ] Add selected-project electrical/component view as a separate explorer mode.
- [ ] Open/focus sheet tabs by scoped stable identity and preserve per-sheet camera/selection.
- [ ] Keep shell bounds stable and action metadata consistent across tree/tabs/ribbons/menus/search.
- [ ] Test project switches, closing inactive/active tabs/projects, keyboard routing and draft protection.

### 6. Hierarchy and production layouts

- [ ] Add stable hierarchical sheet links, continuation connectors and return navigation tied to electrical interfaces.
- [ ] Add generated drawing lists, typed page roles and project/document/sheet variables.
- [ ] Add structured title-block/table editing, reusable blocks and pinned template adoption.
- [ ] Add output preflight, frozen issue metadata and print/export parity separately.

## Verification gates

Each code slice runs cargo xtask full and focused real-file regressions. UI/host slices also run cargo xtask ui and cargo xtask desktop plus native build/launch on the available platform. Include failed opens/reloads/saves, wrong-project revisions, shared input replay, dirty inactive projects, copied project lineage and close/save-as races. UI tests must verify actual sheet editing and independent project roots; screenshots alone are not evidence of project isolation. Linux/Windows native execution and large-workspace rendering remain separate validation work.
