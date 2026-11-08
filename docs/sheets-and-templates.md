# Drawing sheets, layouts and metadata

Status: target architecture revised to Project → Documents → Sheets. The current v8 prototype still implements an editable Model view plus read-only paper display; that workflow is superseded and must be migrated before further template UI expansion. Project metadata, profiles and paper/history primitives remain useful foundations. See [decision 0030](decisions/0030-sheet-first-project-documents.md).

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

## Current v8 prototype: paper sheets and view placement

The following describes implemented prototype behavior, not the target workflow. Its Model view/read-only paper split is superseded by the sheet editor described above.

Drawing → Sheets opens a complete numeric paper editor. Create a blank sheet or place one existing drawing view. Set a literal drawing number/title, oriented paper dimensions, nonnegative left/right/top/bottom guide margins, a positive rational scale, paper origin and source-view origin. A4/A3/Letter presets and portrait/landscape controls set editable dimensions; custom paper is supported. All dimensions/origins use integer micrometres; scale is paper/model, so 1/2 means half size. Margins must leave a positive interior.

The mapping is paper_point = paper_origin + (view_point - view_origin) × numerator/denominator. Source geometry and electrical data never change with paper edits. Blank sheets support future covers. One view may appear on several sheets; removing a sheet retains the view, while removing/replacing a referenced view is rejected. Sheets have stable IDs independent of unique editable drawing numbers and explicit order. The current project has one implicit ordered drawing set; multiple named sets and multiple placements per sheet remain future work.

Preview validates and retains the exact proposed paper edit; Apply commits it once. Invalid input, cancellation and stale/superseded proposals cannot apply an older layout. Move up/down and Remove sheet are explicit undoable operations. Reordering never renumbers sheets. Changing an already-matching layout preserves redo and checkpoints. Source-view and paper layout edits use separate operations.

The Sheet display selector switches between Model view and read-only paper preview. Paper preview clips to page bounds, draws margin guides, supports cursor zoom, modifier/middle-button panning and F fit, and disables source drawing tools. Return to Model view (or choose an object in the explorer) to edit source geometry. Right-click paper to open that sheet's manager. Model viewport/selection remain transient; deleting/undoing the displayed sheet returns to Model view. Margin guides are not printer hardware limits. This is not a PDF/print/export implementation.

Project schema v8 adds an ordered sheets array alongside v7 metadata. V2–v7 load without invented sheets; explicit save writes v8. Older fixtures remain available and new v8 canonical outputs are tested. Sheet settings are outside electrical subsystem revision bodies. Title-block cells/bindings, template revisions, shared sheet defaults, multiple drawing sets, interactive paper placement, unified bulk sheet properties and output validation remain future work.

See [the workspace and sheet implementation task list](workspace-and-sheet-implementation.md) for dependency order and rework outside the sheet model.

## Delivery checklist

- [x] Include the feature and define scope, variable ownership and portability rules.
- [x] Add project metadata, stable field catalog, exact history, versioned persistence and a bounded multiline editor.
- [x] Add personal/company profile storage and explicit previewed copy/update into a project.
- [x] Add v8 prototype sheets, paper settings, numbering, initial ordering and optional view placement/history.
- [ ] Add logical project documents with stable identities, ordered owned sheets, document metadata and default templates.
- [ ] Migrate existing v8 layouts/free views without losing geometry, electrical bindings, appearance or numbering; preserve or explicitly reconcile shared legacy views.
- [ ] Replace Model view/read-only paper navigation with direct template-backed sheet editing, including transformed pointer/snapping input and one coherent viewport.
- [ ] Replace the default component tree with a page/sheet explorer; retain electrical structure as an alternate mode.
- [ ] Add sheet tabs/panes with stable identity, retained per-sheet workspace state and one project session/history.
- [ ] Separate library/definition editor workspace kinds from project drawing sheets.
- [ ] Add stable hierarchical-sheet navigation independently of document ownership and output drawing sets.
- [ ] Add table/cell geometry and editing with stable identities and bulk properties.
- [ ] Add bindings, sheet-local values, missing/required diagnostics and override/reset behavior.
- [ ] Add standard/draft/cover/contents templates, shared blocks and contents pagination.
- [ ] Add pinned template revisions and explicit reviewed adoption.
- [ ] Add portable logos, output validation, frozen issue metadata and reproducible drawing output.

Each slice requires headless invariant/history/persistence tests and real bridge/browser workflows where UI is added. Native Windows/Linux execution and output/rendering parity must be measured independently.
