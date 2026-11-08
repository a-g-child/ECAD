# Consolidated drawing-template library editor

Status: saved definitions, local publication, project bundling, assignment migration and saved-only desktop controls are implemented. A first independent library editor now provides its own history, contextual ribbon, placeholder preview and exact paper/content/cell-content properties. Exact numeric table resize, cell grid split, rectangular merge and aligned-boundary editing are now available. Direct cell-range selection, move/resize handles and contextual row/column insertion are implemented. Verified standards presets remain planned. See [editor boundary](decisions/0041-independent-template-editor.md) and [template requirements](sheets-and-templates.md).

## 1. User workflow and separation

Libraries ribbon → Drawing templates opens the template library and its dedicated editor view. Browse saved templates by name, role, paper family/size and revision; create, duplicate, edit a draft, save a revision, archive or export/import a template. Reusable title blocks are available within the same library/editor workflow rather than a second unrelated drawing tool.

A template editor is a distinct workspace kind, with its own document controller, validation, dirty state and undo/redo. It can open without an electrical project and must not create project sheets, electrical objects or hidden source views. Its preview uses the definition's own paper directly; creating a template does not require assigning another template to its editor. Selecting a template view activates its contextual ribbon and properties; returning to a project sheet restores the drawing tools. Library edits do not enter project undo. Initial switching/close guards protect unfinished tools and unsaved template drafts rather than silently discarding or retargeting them.

New sheet creation offers saved templates and saved size/orientation variants only. Show a thumbnail, template/revision, role, paper dimensions and border summary; collect drawing number/title, owner and permitted sheet-owned entries. A schematic still creates its source view atomically with the sheet. New-sheet creation contains no cell/table editor, paper-size editor or margin editor. Sheet properties allow reassignment to another saved template and editing of sheet values/placement; they do not secretly create a modified template.

Project Properties retains per-role defaults, now pinned references to saved templates/variants. Changing a default affects future sheets. Reassigning existing pages is a separate reviewed project command, with preview, compatibility/value-mapping diagnostics and exact undo. Editing a saved library revision never updates consuming projects automatically.

## 2. Editor view, contextual ribbon and properties

Keep the existing shell bounds, pane scrolling and shared controls. The editor uses paper coordinates and sample data, independently of the active project canvas.

| Surface | Planned behavior |
| --- | --- |
| Left pane | Template/variant, title blocks, tables, cells, content regions and guides; stable selection identities. |
| Main view | Editable paper preview with border, anchoring guides, title blocks and content regions; zoom/pan/fit and exact numeric editing. |
| Right properties | Full properties for the template or selection. With no selection, show template properties. Right-click → Properties targets the same inspector. |
| Contextual ribbon | File and Libraries remain available; the active template view adds/activates a Template ribbon. Returning to a project sheet restores its drawing context. Keep the ribbon height fixed as its groups change. |
| Preview | Switch between placeholders and explicit sample project/document/sheet data; display missing values, overflow and role/layout diagnostics. |

Contextual ribbon groups:

- **Template:** New, Open, Duplicate/Save as, Save revision, Validate and Close.
- **Paper:** size family/preset, orientation, margin/border profile and binding edge.
- **Title blocks:** insert an existing block, create a table block, select rows/cells, insert/delete rows, split, merge and resize.
- **Contents:** literal text, grouped variable picker, sheet-entry fields and generated-content regions.
- **Arrange:** exact position/size, anchoring, alignment and distribution of selected template elements.
- **Preview:** sample-data context, placeholders/resolved text, diagnostics and Fit.

Register eligible operations centrally for ribbon, menus, context menus, keyboard and command search. Reuse protected text/IME behavior and explicit default/unbound key policies. Context is the captured template session/revision and selection; a delayed action cannot target the project document or another template tab. Shared controls remain keyboard accessible without an action for every possible property value.

Template properties include name/description/tags, supported roles, identity/revision/source, paper preset and dimensions, orientation, border profile/binding edge/margins, default text/border appearance, block references and anchors, field definitions and generated-content regions. Selection exposes the full applicable table/cell/region properties. Mixed selection uses the existing union/applicability approach: edit applicable properties together, skip inapplicable pairs and reject invalid/read-only changes coherently. Multi-element drags/resizes are one undo operation.

## 3. Paper sizes and border profiles

Keep paper dimensions separate from drawing-border/margin rules and printer hardware limits. Size alone does not select every layout convention. Standards profiles carry designation, edition/amendment information, supported formats/orientations, binding allowances, border rules and verification provenance.

Initial size catalogue target, written short edge × long edge with orientation selected separately:

| Family | Presets |
| --- | --- |
| ISO A | A0 841 × 1189 mm; A1 594 × 841 mm; A2 420 × 594 mm; A3 297 × 420 mm; A4 210 × 297 mm. |
| ANSI | A 8.5 × 11 in; B 11 × 17 in; C 17 × 22 in; D 22 × 34 in; E 34 × 44 in. |
| Custom | Exact dimensions entered and saved in the template editor. |

Use ISO 216 for the paper-size catalogue, ISO 5457 for drawing-sheet layout profiles, ASME Y14.1 for the ANSI/ASME drawing-format family, and ISO 7200 as a reference for title-block field semantics. Their publisher descriptions establish these scopes: [ISO 216:2007](https://www.iso.org/standard/36631.html), [ISO 5457:1999](https://committee.iso.org/standard/29017.html), [ASME Y14.1-2020](https://www.asme.org/codes-standards/find-codes-standards/drawing-sheet-size-and-format), [ISO 7200:2004](https://www.iso.org/standard/35446.html).

Exact numeric margin/border tables, format exceptions and amendments must be verified against the selected normative edition during implementation. The public catalogue pages do not provide those complete tables. Do not invent a universal ANSI margin, infer compliance from paper size, or label today's three prototype frames as standard-compliant. Customising a standards profile preserves its source/provenance and labels the result Custom/modified.

Store exact integer micrometres; an inch is exactly 25,400 µm. Offer millimetre/inch display without repeated floating-point conversion of saved geometry. Orientation changes oriented dimensions and recomputes anchored layout according to explicit rules. Binding-edge policy is visible and deterministic. Oversized blocks produce diagnostics instead of shrinking or moving schematic content. New sheet creation filters/selects saved size variants; an unavailable size is created and saved in the library first.

## 4. Structured title-block/table editing

A title block is a retained structured block, not independent loose line segments. Provide row/column counts, different subdivisions per row, exact dimensions and drag handles. Support literal multiline contents, captions, variable bindings, sheet-entry fields, padding, horizontal/vertical alignment, fonts/text heights, borders and shared formatting. All text is multiline-capable from creation.

Plan a table extent with stable rectangular cell IDs and integer cell bounds. Validate positive sizes, containment and a complete nonoverlapping partition of the table. This permits unequal columns per row and rectangular merged cells without forcing users to draw a fine global grid. Shared borders are derived once from the partition, avoiding doubled strokes. Expose logical rows/columns as editing operations over the structure.

Split creates cells through one table command. Merge is allowed only when the selected cells form a rectangle; the user reviews how multiple contents/bindings are retained or replaced. Row/column resizing adjusts the coherent partition, including spans, or rejects an ambiguous proposal. It never leaves disconnected line segments. Define deterministic stable-ID/field-mapping rules for split/merge/delete before implementing them; do not tie sheet values to cell positions or displayed captions.

Cell contents are typed literals, field references, sheet-entry fields or simple literal/field runs. Variables are grouped by project, logical document, sheet and derived output context. Company/contact values come from portable project metadata copied from profiles, not the opening user's machine. Drawn/checked/approved responsibilities remain distinct from engineer/contact. Unknown/missing bindings remain visible diagnostics. Arbitrary executable formulas are outside the first release.

A sheet-entry definition declares stable semantic field identity, label, type, default and required status. Values live on sheets; reusable template definitions contain structure/bindings. Two cells may intentionally reference one field. Adoption maps by field identity and preserves unmatched user values for review, including when cells are removed or merged.

Text wrapping and overflow are explicit policies. Mark clipped/overflowing text and retain complete values. Font shrinking requires an explicit choice. Template appearance supports explicit overrides and declared inheritance; the editor uses labelled sample project values when previewing inherited styles, never accidental active-project or personal state.

## 5. Template composition and generated content

Separate a reusable title-block definition from the complete sheet template. A saved sheet-template revision pins paper/variant settings, border/guide structure, title-block revision references and anchors, style intent, field schemas and content-region definitions. Referenced blocks/assets are immutable and bundled transitively; resolving an external latest revision at render time is forbidden.

Tables owned by a template draft can be edited locally. Editing a linked shared title block requires an explicit local copy or its own library editor/revision; saving a template must never rewrite the shared published block.

Use explicit named regions for schematic content, cover information and drawing lists. The current shortcut of taking everything above a fixed footer is insufficient once blocks are freely positioned. Reuse the same deterministic layout projection for editor preview, actual sheets and future output. Template geometry remains in paper units; source view geometry and placement remain separate.

Drawing-list definitions configure columns, widths, row sizing, header and overflow/pagination rules. Entries still derive from real sheet numbers/titles/stable targets. Template adoption or paper/style changes recalculate capacities and show uncovered entries before Apply. Automatic list-page creation and output-set pagination remain separate explicit project operations; saving a library template cannot silently add/remove project sheets.

## 6. Saved revisions, portability and boundaries

Definitions have stable lineage identity; saved revisions are immutable. Drafts have their own session/history and never appear in assignment pickers. Save validates and stores a revision; first-save templates become available for selection. Later saves keep earlier bodies and project references. Save as/Duplicate creates a new definition, with explicit identity mapping. Archive hides a definition from normal new-assignment lists while retaining referenced revisions.

Sheet assignment resolves an exact saved template/variant and bundles the complete required body into the project. A project reopens/render offline without the originating library path. Existing saved project-bundled templates remain usable if the library is missing. Missing/corrupt referenced bodies fail visibly; there is no silent fallback to the standard frame. Recommendations/default selections and project adoption are distinct actions. Known-use discovery is advisory, with coverage/freshness, not proof of global absence.

Library writes stay outside project undo and use validated versioned storage, staged saves and cooperating-writer coordination. Failed saves retain drafts/history; never silently overwrite external changes or steal locks. Project assignments/adoption use ordinary validated commands and exact inverses. Do not store template bodies inside electrical subsystem revisions.

Preserve the headless dependency direction. Domain owns identities, paper profiles, table partitions, field schemas and layout validation. Application owns independent editor sessions, commands, preview/adoption and history. Adapters own library/project DTOs, migration, assets and durable saves. Bridge DTOs remain separate, using strings for exact values/identities. The UI owns transient selection, handles and preview presentation. Extend workspace/action targeting with a distinct template-editor session and revision; the current Project-specific Session/Document, ActionContext, dispatcher and bridge projections cannot be reused by pretending a template is an electrical project or drawing view. Route Save, Undo/Redo and Close to the captured editor kind and protect all dirty editor sessions on Quit. Refactor the fixed Project.sheet_frame calculation into a headless template-layout resolver with explicit binding/style context, called by both template preview and actual sheet projection. Start with canonical versioned library files consistent with current Git-friendly storage; a future SQLite index may be a derived cache, not a second authority. Add concrete modules as features land, without a plugin framework.

## 7. Rework and preservation migration

| Current implementation | Required change |
| --- | --- |
| Three SheetTemplate enum variants with computed fixed tables | Replace assignment references with real template/variant IDs and immutable saved bodies; retain built-ins as shipped saved definitions. |
| Per-sheet SheetPaper and editable paper/margins in creation/settings | Move layout ownership into saved variants; show resolved read-only layout during creation and permit saved-template reassignment later. Keep source placement and sheet values separate. |
| Project default enum keys and document fallback | Migrate to pinned saved references, retaining explicit sheet pin versus inheritance intent. |
| FrameCell values generated from fixed semantic keys | Preserve stable bindings while adding saved cell geometry/content/formatting and sheet-owned field values. |
| Drawing-list capacity inferred above a footer | Resolve an explicit saved list region and formatting rules. |
| One project editor/tool state and planned Library controls | Add a separate template-editor workspace/session and contextual action availability; protect project and template drafts independently. |

Before removing current paper controls, capture every existing distinct frame/paper/margin combination as a deterministic project-bundled saved variant. Preserve source coordinates, electrical identities, sheet IDs/numbers/titles/order, placements, roles, style inheritance and explicit equal-valued overrides. Do not guess a standards profile or silently correct legacy margins. Shared views remain shared. Repeated migration must produce the same identities/body references and no load-time library writes or random IDs.

Legacy layouts which do not meet new authoring validation still need faithful rendering and diagnostics. Retain their saved legacy bodies for existing assignments; require an explicitly repaired saved revision before offering an invalid legacy variant for new assignment. Migration must not reject previously valid projects merely because their tiny paper omits a title block. Adopt changed layouts only through a reviewed operation. Preserve sheet-entry values by field identity and leave schematic geometry unchanged, with visible fit/overflow diagnostics.

## 8. Ordered implementation checklist

- [ ] Record ownership/reference contracts and exact standard-profile sources, editions and numeric acceptance fixtures.
- [x] Add template definition/revision/variant and table/cell IDs, immutable variant bodies, validated rectangular partitions, literal/known-field runs and project-independent legacy layout projection.
- [x] Add strict standalone template-variant schema v1 encoding/decoding with exact units and retained presentation order.
- [ ] Complete reusable title-block revisions, custom field schemas, style/overflow intent and general anchored layout projection.
- [x] Implement closed complete template revisions, append-only catalogue/recommendations, strict canonical library persistence and observed-version publication with cooperating-writer coordination.
- [x] Bundle/import complete current template bodies through project history and strict v12 persistence, with offline reopening.
- [x] Add deterministic revision-bound preservation capture of current sheets and unused document/project defaults, retaining legacy geometry and pin/inherit intent.
- [x] Convert actual sheet/document/project assignments to saved references with inherited paper variants, in-use guards and exact style/placement preservation.
- [ ] Complete reviewed bulk revision adoption and custom field/value mapping.
- [x] Integrate saved project-template choices and creation defaults; paper/margin values are read-only and inline size/preset/orientation authoring is removed after preservation migration.
- [x] Add Libraries → Drawing templates access and a separate template workspace/session with independent save/close/undo and contextual ribbon/properties.
- [ ] Deliver the first usable editor: paper presets/profiles, one anchored structured title block, numeric/drag cell sizing, per-row subdivisions, literals/known bindings and sample preview.
- [ ] Complete split/merge/spans, reusable title-block composition, custom sheet-entry definitions, mixed formatting and reviewed revision adoption/value mapping.
- [ ] Connect explicit generated-content regions, list formatting/capacity diagnostics, standards-profile deviations and asset/font diagnostics.
- [ ] Add portable logos/assets, output preflight and PDF layout/link parity as subsequent output work.

Ship foundational storage/migration before enforcing the new picker contract. The first usable editor must edit real saved structured content; a painted mock-up or an unsaved inline variant does not satisfy this feature.

## 9. Acceptance and local verification

- Create an A3 landscape template from an edition-labelled profile, split title-block rows into differing cell counts, resize cells and bind company/project/sheet fields. Save, reopen and assign it through project defaults.
- New-sheet pickers show saved variants only. Unsaved edits to a template do not leak into pickers or consuming sheets. An unsupported size requires a separately saved variant.
- Switch project/template views: the correct ribbon, inspector, selection and history are active; delayed actions, close/cancel and text/IME input cannot affect another editor session.
- Edit sample values in the template preview without changing a project. Two real sheets resolve the same template against different titles and user entries.
- Resize/split/merge with spans and multiline content. Invalid extents/content handling fail coherently; Undo restores exact cell geometry, IDs and bindings.
- Save a new library revision while several projects use the old one. Existing pages remain pinned; explicit adoption previews metadata/field mapping, bounds and list capacity, with one project undo.
- Upgrade fixtures containing custom paper/margins, tiny sheets, shared/unplaced views, covers/lists and equal-valued overrides. Reopen offline and prove stable IDs, exact source geometry and preserved appearance/intent.
- Exercise malformed partitions/references/units, unknown bindings, failed/conflicting saves, missing library files, recommendation changes and archived definitions.
- Measure deterministic 10/100/1,000-cell fixtures and reuse across many sheets; record layout/cache invalidation, memory and visible editing costs before setting performance gates.

Implementation runs cargo xtask full, cargo xtask ui and cargo xtask desktop where relevant, plus migration/golden/regression fixtures and native host checks. Linux/Windows execution and PDF/font/output parity remain independently verified gates. The initial implementation routes existing sheet frames through the headless legacy layout projection, with exact geometry/field preservation regressions. Project persistence is now v13; standalone variant serialization remains a separate contract from publication and project bundling.

## First implemented foundation

`TemplateLayout` owns exact paper/margins, an explicit content region and validated structured tables. `TemplateTable` accepts complete rectangular cell partitions, including unequal subdivisions per row and rectangular spans. It rejects overlaps, gaps, invalid extents and repeated identities using widened integer arithmetic. Table/cell geometry is immutable after construction; future editor commands will replace complete validated proposals rather than mutating loose edges.

`TemplateCell` retains multiline captions and ordered literal/typed-field runs. Resolution uses an explicit caller-supplied context and reports missing fields separately from an intentionally empty value. No project, user profile or renderer is consulted implicitly. `SavedTemplateVariant` retains a typed definition/revision/variant pin and body; constructing or encoding it does not publish it. The standalone strict v1 DTO is separate from project persistence, rejects unknown bindings/fields and stores no sample data, timestamps or machine paths.

The three existing enum templates now materialize this model through `SheetTemplate::layout`, used by actual `Project::sheet_frame` rendering. Existing border/table/content geometry, cell ordering, captions and missing-value behavior remain preserved, including tiny legacy sheets with no title block. These are explicitly legacy/custom borders, with no standards-compliance claim. This extraction preceded the preservation and assignment migrations described below.

Checks cover unequal partitions/spans, repeated identities, overlap/gaps, owner containment, maximum integer extents, multiline runs, missing/empty values, exact legacy geometry, tiny-sheet preservation and strict canonical serialization. Transitive project bundles, deterministic migration pins, paper catalogues/verified profiles, cell styling/border projection, independent application history, editor/ribbon/properties and saved-only pickers remain pending.

## Implemented local library publication

`TemplateRevision` closes a nonempty complete set of saved variants under one definition/revision pin. Adding a variant to that set requires a new revision; a published revision cannot grow incrementally. `TemplateLibrary` stores immutable complete revisions and explicit advisory recommendations, while exact variant lookups retain their original pins. Recommendations can move backward or be cleared without deleting history.

`TemplatePublisher` and `VersionedTemplateLibraryStore` keep publication outside project editing. `TemplateLibraryFile` writes strict canonical library schema v1, validates observed destination/content tokens even for no-ops, rejects history removal/rewrites, coordinates cooperating writers and retains observed state on failed save/reload. Shared file-lock mechanics preserve the subsystem library's existing lock convention/guarantees. See [decision 0038](decisions/0038-closed-template-revision-publication.md) for the closed-revision contract and filesystem limits.

The offline [template publication example](../crates/ecad-adapters/examples/template_library.rs) saves/reopens two variants; [real-file checks](../crates/ecad-adapters/tests/template_library.rs) cover strict fixtures, invalid/dangling recommendations, conflicts, complete-revision immutability, concurrent creators, locks and failed IO/reloads. Project bundling/import is implemented in v12 as described below. Saved-reference assignment is implemented below; reviewed bulk adoption and the independent editor lifecycle remain pending.

## Implemented project bundle and preservation stage

Projects now store exact complete template revisions in strict schema v12. `template_library_import` prepares an explicit ordinary project command, with immutable-key checks and exact undo/rollback. Reopening these bodies needs no library. V2–v11 input retains its original model with an empty bundle; opening a file never publishes/captures templates implicitly. V1 remains an explicit identity migration.

`capture_legacy_templates` retains a closed proposal at the current session revision, including exact sheet-specific paper/margins and mappings distinguishing inheritance from explicit equal-valued frame choices. It also captures unused document/creation defaults at default paper size. Deterministic versioned IDs depend on template/layout inputs rather than project paths, lineage or personal metadata. Importing the proposal is one undoable batch. Shared/unplaced views, source/electrical geometry, metadata and existing style/default intent remain untouched; tiny valid sheets are preserved. See [decision 0039](decisions/0039-project-template-bundles-and-legacy-capture.md), [bundle/capture checks](../crates/ecad-adapters/tests/project_template_bundles.rs) and [offline example](../crates/ecad-adapters/examples/project_templates.rs).

This preservation stage preceded assignment conversion. The saved-reference, inheritance and in-use contracts are now implemented below, with paper values read-only in sheet settings. The separate library editor remains pending. Current bodies own their tables; transitive reusable-block/asset bundling remains coupled to implementing those dependency types.

## Implemented saved-reference assignments

[Decision 0040](decisions/0040-saved-template-assignments-and-inherited-variants.md) completes assignment conversion. The desktop migrates legacy choices in memory before its clean checkpoint, using complete saved families with stable inherited paper variants. Source bytes remain unchanged until explicit Save writes v13. Normal sheet/default controls select saved project-bundled variants, and numeric paper/margins are read-only projections. All references resolve locally; in-use revision removal and invalid inherited transfers are rejected. Project creation defaults still affect future sheets only.

Retained tiny legacy layouts carry explicit authoring eligibility in independently versioned variant v2 bodies. Existing sheets render/reopen/undo exactly; new selection requires a repaired revision. Normal v1 bodies retain canonical encoding. Actual sheet rendering now uses arbitrary retained table geometry and literal/composed multiline bindings rather than manufacturing eight legacy cells.

The [migration regressions](../crates/ecad-adapters/tests/template_assignment_migration.rs) and 42 real browser workflows cover these contracts. Raw headless compatibility remains explicit, while normal desktop loading/creation uses saved references. The Libraries manager, dedicated draft editor/session, contextual ribbon/properties and table editing are the next major delivery phase.


## First independent editor slice

Libraries → Drawing templates opens a separate editor over the existing project workspace, with a fixed contextual ribbon and bounded library/preview/properties panes. The project canvas and selection remain mounted. New starts a separate custom layout; Open selects a saved variant from the local catalogue or project bundle. The preview uses literal placeholders rather than company/project values. Cursor zoom, Shift horizontal pan, Ctrl vertical pan, middle pan and Fit apply to its own camera.

Template name, exact paper/margins/content bounds and selected cell captions/literal/variable runs are editable. Text remains multiline. Apply validates the complete replacement layout and records one template history entry; invalid proposals retain the fields. Revert discards uncommitted fields. Right-click a cell opens the same properties. The first slice exposed table/cell bounds read-only. Numeric structural editing and content-run insertion are now implemented below; insertion/deletion of tables, drag tools, styling, anchors, reusable blocks and standard-profile presets remain future work. This slice does not claim full table authoring or standards compliance.

The headless `TemplateDocument` needs no electrical project, sheet or source view. It owns an independent epoch, exact bounded undo/redo and publication checkpoint. The desktop currently hosts one template editor alongside one project. Save publishes a complete fresh immutable revision outside project undo; undo never retracts a publication. Add saved revision to project is an explicit project-history operation; existing sheet pins/defaults do not change. New/Open/Close require a clean draft; explicit Discard closes without publishing. Pending property fields must be applied, reverted or discarded before other operations. Native close/quit includes draft and pending-field dirty state; project Open/Save As is unavailable until the template editor closes. Unapplied fields are transient browser state, not crash recovery.

The native library is stored at the platform application configuration directory as `drawing-templates.json`. Reads do not create it. Explicit Save creates a missing parent and uses observed-version/lock protection. Reload library refreshes publication state after a conflict without losing the draft. Project bundles remain sufficient for offline drawings. The browser harness uses its temporary project directory or an explicit `ECAD_UI_TEMPLATES` path.


## Implemented numeric structured table tools

The Template ribbon and command palette now expose Resize title-block table, Split title-block cell, Merge title-block cells and Move table boundary. Review dialogs capture the variant/table/editor revision. A split keeps content/identity upper-left and allocates blank cells for the other subdivisions; repeated splits support unequal rows. Merge requires a rectangular union and an explicit survivor/content policy, defaulting to combine typed runs and captions in reading order. The dialog shows current captions/contents before applying. Whole-table scaling maps shared boundaries with exact integer ratios; boundary movement adjusts all aligned endpoints together and preserves spans. Invalid extents or crossing/collapsing boundaries reject without changing history.

Each Apply is one exact undoable variant edit. Cancel retains redo; pending dialog fields participate in dirty guards and cannot commit through another operation. Blank cells can now add/remove ordered multiline literal or known-variable runs through the properties panel. Save still publishes a complete new library revision; sheets do not adopt it automatically. See [decision 0042](decisions/0042-structured-template-table-edits.md) for identity/rounding/content rules and the bounded split request. Interactive drags, table insertion/removal, full style properties, standards presets and reviewed field-value adoption remain future work.


## Implemented direct table interaction

Drag across cell interiors to select a rectangular range; merged cells expand selection to complete spans. The Merge dialog starts with that range checked. Selected tables expose a move grip, draggable interior/outer edges and contextual plus controls. Interior drags move aligned boundaries coherently, outer edges resize the complete partition, and the move grip preserves local cell geometry. Plus controls insert a blank band after the selected range, keeping the bottom/right edge fixed. Cells spanning an inserted boundary extend; other divisions receive blank cells with fresh IDs. Numeric insertion actions are also available through the registry/palette and can choose either anchor.

Pointer previews are validated in Rust without changing history. Release commits the exact latest successful retained proposal as one undo step; Escape and invalid releases preserve the original geometry and redo. Later invalid/malformed proposals cannot fall back to an earlier valid token. Re-rendering and delayed responses cannot change the captured table/boundary target. All controls use the existing fixed shell. See [decision 0043](decisions/0043-direct-template-table-interaction.md). Whole-table creation/removal, localized span-aware boundary drags, shared multi-cell formatting, verified standards profiles and reviewed value adoption remain planned.
