# Drawing sheets, layouts and metadata

Status: target architecture revised to Project → Documents → Sheets. The bounded desktop now edits directly on template-backed sheets; the numeric sheet review remains a separate clipped preview. Project metadata, profiles, paper/history primitives, logical document ownership and v11 project creation defaults, typed pages and generated drawing lists are implemented foundations. See [decision 0030](decisions/0030-sheet-first-project-documents.md).

## Scope and ownership

Actual project drawings always belong to template-backed sheets. Organize them as Project → Documents → Sheets. Documents are logical drawing packages, not automatically separate files or electrical subsystems. A sheet is the normal editable drawing workspace, with paper, border, drawing information and template together with its schematic content. Cable overviews, hierarchical diagrams, detailed conductor/connector schematics and project layout drawings all follow this rule. Cover and contents sheets use the same paper model with their own content type.

Each sheet has one owning document and a stable identity independent of numbering/order. Each document has a stable identity, metadata, an ordered sheet collection and a default template. Each sheet must resolve an explicit project-bundled template or its document's default. New drawings receive a basic standard template; there is no intentionally untemplated project-drawing mode. The table/template editor customizes this frame rather than making sheets optional.

Internal drawing views may remain representation/coordinate structures, but are not separate free-canvas project documents exposed to users. Source geometry remains separate from paper/template geometry so changing paper or a title block does not move or rewire the schematic. Tools operate directly on the active sheet through its validated coordinate transform, using existing headless commands and exact undo.

Symbol, generic component/device, footprint and template-definition editors are distinct workspace kinds. They use their own origins, units, grids and preview contexts and do not require project drawing borders. Opening one must not create a project sheet or turn library definitions into schematic occurrences.

Keep electrical hierarchy independent of document ownership. A hierarchical block on an overview links to the target sheet by stable identity and to its electrical subsystem/interfaces explicitly. Moving a sheet between documents or renumbering it must not change electrical containment or break navigation. Drawing sets are output/issue selections over documents/sheets, not a replacement for the document hierarchy.


## Project explorer and sheet workspace

The left Project explorer can hold multiple project roots and defaults to each project's pages/sheets, rather than devices and conductors. Typical entries are Cover, Drawing list, Overview schematics and detailed wire-to-terminal/connector sheets. Logical documents can group related pages. When there is only a default document group, omit its group node from the displayed tree and show its pages directly under the project. Electrical/component/functional structure remains accessible as a separate explorer mode scoped to the selected project.

The main workspace opens sheets in selectable tabs/panes. Selecting a page opens or focuses its existing tab by stable sheet identity; it does not create another sheet or another electrical model. Tab switches preserve each sheet's viewport/selection and do not dirty the project or commit/cancel a tool implicitly. Active tools remain bound to their original sheet; an initial implementation may block switching until finish/cancel or retain the draft, but must never silently retarget it. Project sheet tabs share their own project's authoritative session and history; tabs and cached selections include the runtime open-project handle so copied projects cannot collide. Split views may follow later; tab content must stay inside stable shell-owned pane bounds.

Each page adopts its resolved CAD template, including border and drawing information, and is edited directly in that frame. Cover, generated drawing-list, overview and detailed pages have different content roles, not different requirements for paper/template backing. Drawing-list entries derive from project document/sheet identity, order, titles, drawing numbers and revisions rather than a manually maintained index.

Hierarchical sheet blocks and off-sheet connectors support drill-down and cross-reference navigation. A block targets a stable sheet and electrical interface; a continuation connector represents shared electrical connectivity, not a new physical connector/device. Actual connector components retain their device/terminal identities. Displayed sheet names, drawing numbers and text labels are not connectivity keys. Navigating a link opens/focuses the target tab and retains a route back to the source sheet.

Variables have explicit scopes:

| Scope | Examples | Ownership |
| --- | --- | --- |
| Personal/company profile | Company name, address, logo, engineer/contact | Reusable starting values outside project undo |
| Project | Project title, product, client, contract, issue, copied company/contact values | Portable document values with undo |
| Document | Document title/number/revision and default template | Logical drawing-package values with undo |
| Sheet | Drawing title/number/revision, drawn/checked/approved responsibilities, custom entries | Sheet-owned values with undo |
| Derived | Sheet number, drawing-set count, placement scale, contents rows | Read projections from the chosen drawing set |

Copy profile values into a project explicitly; never resolve issued drawings against whichever user opens the file. Applying updated profile values must offer a reviewable project edit. Current user, contact engineer, drawn by, checked by and approved by are different concepts. Project issue, sheet revision, template revision, Git commit and internal session revision remain separate.

## Table and template editing

The [consolidated library editor plan](template-editor-plan.md) and [decision 0037](decisions/0037-library-template-editor-workspace.md) define the next ownership/workflow change: templates are authored in a separate editor view through the Libraries ribbon, with contextual tools and template properties. New sheets select saved variants only. Paper/margins and title-block structure move out of sheet creation/settings after a preservation migration. Saved-reference migration and read-only sheet layout controls are implemented; the independent library editor remains planned.

Use structured tables with stable cell and field identities. Support row insertion/removal, different cell counts per row, cell split/merge, exact/drag sizing, captions and values within a cell, padding, alignment, borders and shared formatting. Users should not need to construct tables from loose lines or calculate a fine underlying grid. All text supports multiple lines. Bulk cell properties use the common inspector and shared action system.

Cell contents may be fixed text, a typed binding selected from a grouped field catalog, a sheet-specific entry or a simple literal/field combination. Custom field definitions declare identity, label, type, default and required status. Preserve missing bindings as diagnostics; do not guess unknown fields. Show local overrides and provide reset-to-linked-value. Arbitrary executable formulas are outside the first slice.

Templates inherit drawing defaults with explicit formatting overrides. Keep template structure editing separate from normal sheet-value editing. Pin immutable template revisions in each project and adopt updates explicitly with a preview; retain local values by stable field identity. Electrical subsystem revision bodies are not template storage.

Starter templates: standard drawing, draft drawing, cover sheet and contents sheet. Share reusable title blocks between them. Draft markings do not change approval status. Contents tables derive from ordered drawing-set membership and may paginate; their layout/inclusion rules are editable. They are not manually maintained sheet-number lists.

## Layout and output rules

Store borders/text sizes in paper units independently of placement scale. Define paper size, orientation, margins and corner/margin anchoring. Changing sheet layout must not silently transform schematic geometry. Keep stable sheet IDs separate from editable numbers and ordering; subset output must not unexpectedly renumber the document.

Wrap text and show overflow; shrinking requires an explicit policy. Carry logos as portable project assets with sizing/aspect-ratio controls. Required-field, missing-asset and overflow checks belong before output. Historical issues eventually need frozen metadata and reproducible outputs. Font resolution and native output parity need dedicated verification.

## First executable slice

Project metadata has eight catalogued fields: company.name, company.address, engineer.name, project.title, project.product, project.client, project.contract and project.issue. These are project snapshots, not live links to machine settings. Empty strings remove entries; other text is preserved literally. The contact field confers no author/approval role.

Project schema v7 adds a strict canonical metadata map. V2–v6 load with empty metadata; v1 remains an explicit migration. Updates use one reversible metadata command and a shared registered project-settings action. No-op edits preserve redo and checkpoints. Unknown persisted fields and duplicate metadata keys are rejected. Metadata is excluded explicitly from electrical subsystem bodies rather than silently discarded.

## Reusable company/contact profiles

File → Profiles opens the shared profile manager. Save multiple named profiles, update or rename them while retaining stable identity, then select fields for a reviewed import. Profiles contain company.name, company.address and engineer.name only. Project title, contract, issue and author/checker/approver roles are excluded. All text, including names and addresses, remains multiline-capable.

Saving a profile is outside project undo and does not dirty a project or clear redo. Preview shows exact old/new project values. Populated fields are selected initially; empty fields are unselected and clear project entries only if explicitly selected. Apply commits the closed revision-bound copy as one project edit. Close/Escape cancels the retained proposal. Changing field choices or editing a profile invalidates the UI review; save edits before previewing again. Reload profiles explicitly discards profile drafts and refreshes external file changes.

Native profiles live in company-profiles.json under the Tauri-provided per-user application configuration directory. Startup/configuration and missing-file reads do not create it; explicit Save profile creates directories and writes it. Projects retain copied values and never reference this path. The browser test harness uses a separate file alongside its temporary project; ECAD_UI_PROFILES can explicitly override its location. Profile schema v1 is separate from project schema v7. Saves use observed-byte/location checks and staged replacement; concurrent uncooperative writers can still race between check and rename. Automatic default-profile selection, deletion, profile import/export packaging and logos remain future work.

## Paper sheets and view placement

The numeric paper editor and unified drawing canvas now share the same sheet/frame contract. The desktop still has a single open-project context and a flat sheet selector.

Drawing → Sheets opens a complete numeric paper editor. Create a blank sheet or place one existing drawing view. Set a literal drawing number/title, oriented paper dimensions, nonnegative left/right/top/bottom guide margins, a positive rational scale, paper origin and source-view origin. A4/A3/Letter presets and portrait/landscape controls set editable dimensions; custom paper is supported. All dimensions/origins use integer micrometres; scale is paper/model, so 1/2 means half size. Margins must leave a positive interior.

The mapping is paper_point = paper_origin + (view_point - view_origin) × numerator/denominator. Source geometry and electrical data never change with paper edits. Blank sheets support future covers. One view may appear on several sheets; removing a sheet retains the view, while removing/replacing a referenced view is rejected. Sheets have stable IDs independent of unique editable drawing numbers and explicit order. The current project has one implicit ordered drawing set; multiple named sets and multiple placements per sheet remain future work.

Preview validates and retains the exact proposed paper edit; Apply commits it once. Invalid input, cancellation and stale/superseded proposals cannot apply an older layout. Move up/down and Remove sheet are explicit undoable operations. Reordering never renumbers sheets. Changing an already-matching layout preserves redo and checkpoints. Source-view and paper layout edits use separate operations.

The Sheet display selector chooses an editable template-backed sheet. Cursor zoom, modifier/middle-button pan, marquee, dragging and exact numeric placement operate in source coordinates; the frame uses inverse placement. F fits the paper, including blank sheets. Source content outside paper remains visible for editing/recovery, while numeric paper review is clipped. Shared views display an editing warning. Right-click a source object opens its inspector; right-click the background opens sheet settings. No-sheet projects show a registered action for explicit reviewed attachment of a retained view, preserving clean loads. See [decision 0034](decisions/0034-editable-sheet-workspace.md).

Project schema v8 adds an ordered sheets array alongside v7 metadata. V2–v7 load without invented sheets; current explicit saves write v10 with document ownership and mandatory template defaults. Older fixtures remain available and new v8 canonical outputs are tested. Sheet settings are outside electrical subsystem revision bodies. Title-block cells/bindings, template revisions, shared sheet defaults, multiple drawing sets, interactive paper placement, unified bulk sheet properties and output validation remain future work.

See [the workspace and sheet implementation task list](workspace-and-sheet-implementation.md) for dependency order and rework outside the sheet model.

## Delivery checklist

- [x] Include the feature and define scope, variable ownership and portability rules.
- [x] Add project metadata, stable field catalog, exact history, versioned persistence and a bounded multiline editor.
- [x] Add personal/company profile storage and explicit previewed copy/update into a project.
- [x] Add v8 prototype sheets, paper settings, numbering, initial ordering and optional view placement/history.
- [x] Add logical drawing documents with stable IDs, ordered owned sheets and literal document metadata.
- [ ] Add actual document default-template references with the basic frame implementation.
- [ ] Migrate existing v8 layouts/free views without losing geometry, electrical bindings, appearance or numbering; preserve or explicitly reconcile shared legacy views.
- [x] Replace Model view/read-only paper navigation with direct template-backed sheet editing, including source-unit pointer/snapping input through inverse placement and one coherent viewport.
- [x] Replace the default component tree with a project/document/sheet explorer; retain electrical structure as an alternate mode for the current project.
- [ ] Add sheet tabs/panes with stable identity, retained per-sheet workspace state and one project session/history.
- [ ] Separate library/definition editor workspace kinds from project drawing sheets.
- [ ] Add stable hierarchical-sheet navigation independently of document ownership and output drawing sets.
- [ ] Add table/cell geometry and editing with stable identities and bulk properties.
- [ ] Add bindings, sheet-local values, missing/required diagnostics and override/reset behavior.
- [x] Add bundled standard/cover/drawing-list templates and explicit list-page pagination.
- [ ] Add draft/custom templates and shared reusable blocks.
- [ ] Add pinned template revisions and explicit reviewed adoption.
- [ ] Add portable logos, output validation, frozen issue metadata and reproducible drawing output.

Each slice requires headless invariant/history/persistence tests and real bridge/browser workflows where UI is added. Native Windows/Linux execution and output/rendering parity must be measured independently.

## Implemented logical drawing-document ownership

ProjectDrawingDocument carries stable identity, unique project-level document number, literal multiline title and engineering revision. Each project starts with an empty Drawings package. Each sheet has exactly one owner and an owner-local order; documents also have explicit order. Sheet drawing numbers remain project-wide unique as in v8, with no automatic renumbering on transfer. At least one package remains in a project; nonempty packages cannot be removed. Metadata, document order, sheet transfers and complete membership changes use reversible commands without changing view geometry or electrical content.

Schema v9 stores physical sheet records by stable ID and records order only in each document's sheet-ID membership list. Missing/duplicate ownership and invalid metadata are rejected. V2–v8 ownership upgrades create the initial package deterministically from the project ID in a separate typed namespace, preserving existing sheet IDs/order/numbers/transforms and shared/unplaced views. No random document IDs are generated on decode; unplaced views are not yet wrapped into template sheets. The bridge exposes document metadata/membership, while the current UI remains flat and does not yet manage packages. Its legacy flattened sheet moves cannot cross a document boundary; new workflows use explicit owner-local operations.

## Implemented bundled standard frame

Each drawing package now persists standard-v1 as its mandatory default. All sheets resolve a margin border and a fixed 180 × 40 mm title block in paper units. Its eight live fields use portable company/project metadata, package title/number/revision, sheet title/number and owner-local sheet ordinal/count. The numeric proposal preview and paper display use the same Rust-resolved frame, including proposed title/count changes without editing the live project.

Font/value height/stroke/colour inherit project drawing fallbacks; caption height is a template override. Literal multiline values remain unchanged; the browser clips overflowing cell text, marks a corner and exposes the full value in a tooltip. Small valid paper retains its border and reports that the table cannot fit. These diagnostics do not establish output/font parity. Custom cells, alternative templates, sheet overrides, logos, required-field checks and print/export remain planned. See [decision 0033](decisions/0033-bundled-sheet-frame.md).

Schema v10 requires a known default_template in each package. V2–v9 upgrade deterministically to standard-v1 without changing source drawing/electrical data or wrapping unplaced views. The desktop now uses the unified editable sheet surface described in [decision 0034](decisions/0034-editable-sheet-workspace.md). Unplaced sources attach through explicit reviewed sheet edits; bulk wrapping remains future work.

## Implemented page and electrical explorers

Pages is now the desktop default: one real project root, ordered document memberships and sheets, including blank pages and empty packages. A sole unchanged default package is elided; custom and multiple packages remain visible. Page clicks and context menus target stable sheet identities through the shared sheet-switch guard. Electrical mode retains real functional/component objects and drawing representations with the full inspector, while mode changes preserve drawing selection and each mode's expansion state. Header buttons, View menu and command search use two explicit unbound registry actions.

This slice supports the currently open desktop project. Multiple roots, document-management controls, project-scoped tabs remain pending. See [decision 0035](decisions/0035-project-page-and-electrical-explorers.md).

## Project properties and typed page creation

Right-click a project for Properties or Add new schematic, cover or drawing-list sheets. Properties include portable per-role template defaults, applied with metadata as one undoable edit. Defaults are copied into each new sheet's pinned frame choice, including generic sheet-manager creation, and do not restyle existing drawings. Existing sheets can explicitly inherit their document template instead. The bundled catalogue provides standard-v1, cover-v1 and drawing-list-v1 with distinct information-block geometry. All three remain general frame choices; page content role is separate.

A new schematic reviews and creates its independent view and sheet together; Cancel creates neither and one Undo removes both. New covers and lists have no schematic placement. Drawing lists derive current non-list sheets, naturally ordered by number and shown with literal titles/stable targets. Their capacities follow actual paper/template/text size; multiple list sheets consume consecutive entries in document order. Remaining entries are diagnosed until additional list pages are explicitly created. Whole-row pointer links and keyboard Enter navigate within the app. PDF export/link annotations, automatic page creation, custom template authoring and richer output sets remain planned.

Schema v11 stores project template defaults plus sheet role/nullable frame override. V2–v10 upgrades keep old geometry, identities and appearance without guessing a role from a title. See [decision 0036](decisions/0036-project-template-defaults-and-typed-pages.md).

## Project-owned template bodies and legacy capture

Schema v12 adds portable complete template revisions independently of electrical subsystem bodies. Explicit template imports use normal project commands and exact history; shared publication/recommendation changes remain separate. The headless preservation proposal captures every existing enum/paper layout and unused document/creation choices with deterministic pins, preserving tiny sheets and explicit/inherited intent. Capture/import does not yet change which template renders a sheet. Current enum/paper controls remain implemented pending saved-reference assignment migration and the library editor. See [decision 0039](decisions/0039-project-template-bundles-and-legacy-capture.md).

## Saved references and inherited paper variants

Normal desktop sheets and project/document defaults now resolve saved project-bundled references. A sheet can inherit the owning document's revision while retaining its stable paper variant; explicit equal-valued choices remain overrides. Exact paper/margins are validated projections of that saved body and shown read-only during sheet editing. Selecting a different saved variant is reviewed through the existing proposal/Apply path; source geometry never resizes implicitly. In-use revision removal, missing bodies and incompatible inherited transfers fail coherently.

Desktop loading opts into an in-memory preservation migration before establishing a clean checkpoint. Explicit saves write v13, observing the original file version. Retained tiny layouts reopen with their omitted-table diagnostic and exact undo; normal authoring excludes them until a repaired saved revision exists. Template editor/ribbon/property panels, standard profiles and interactive table authoring remain pending. See [decision 0040](decisions/0040-saved-template-assignments-and-inherited-variants.md).
