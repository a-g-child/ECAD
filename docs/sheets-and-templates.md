# Drawing sheets, layouts and metadata

Status: included product feature. Project-owned metadata, reusable company/contact profiles and reviewed selective imports are implemented. Ordered paper sheets, one optional view placement, reviewed editing and read-only paper display are implemented. Table editing and placeholder placement remain planned.

## Scope and ownership

Keep drawing views (model representations and geometry), sheets (paper, numbering and local entries), reusable templates (borders, blocks and bindings), view placements (position/scale on paper), and ordered drawing sets distinct. A sheet never owns electrical connectivity. Existing free drawing views remain valid without a sheet. Begin eventual sheet placement with one view per sheet, leaving multiple placements possible later.

Variables have explicit scopes:

| Scope | Examples | Ownership |
| --- | --- | --- |
| Personal/company profile | Company name, address, logo, engineer/contact | Reusable starting values outside project undo |
| Project | Project title, product, client, contract, issue, copied company/contact values | Portable document values with undo |
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

## Paper sheets and view placement

Drawing → Sheets opens a complete numeric paper editor. Create a blank sheet or place one existing drawing view. Set a literal drawing number/title, oriented paper dimensions, nonnegative left/right/top/bottom guide margins, a positive rational scale, paper origin and source-view origin. A4/A3/Letter presets and portrait/landscape controls set editable dimensions; custom paper is supported. All dimensions/origins use integer micrometres; scale is paper/model, so 1/2 means half size. Margins must leave a positive interior.

The mapping is paper_point = paper_origin + (view_point - view_origin) × numerator/denominator. Source geometry and electrical data never change with paper edits. Blank sheets support future covers. One view may appear on several sheets; removing a sheet retains the view, while removing/replacing a referenced view is rejected. Sheets have stable IDs independent of unique editable drawing numbers and explicit order. The current project has one implicit ordered drawing set; multiple named sets and multiple placements per sheet remain future work.

Preview validates and retains the exact proposed paper edit; Apply commits it once. Invalid input, cancellation and stale/superseded proposals cannot apply an older layout. Move up/down and Remove sheet are explicit undoable operations. Reordering never renumbers sheets. Changing an already-matching layout preserves redo and checkpoints. Source-view and paper layout edits use separate operations.

The Sheet display selector switches between Model view and read-only paper preview. Paper preview clips to page bounds, draws margin guides, supports cursor zoom, modifier/middle-button panning and F fit, and disables source drawing tools. Return to Model view (or choose an object in the explorer) to edit source geometry. Right-click paper to open that sheet's manager. Model viewport/selection remain transient; deleting/undoing the displayed sheet returns to Model view. Margin guides are not printer hardware limits. This is not a PDF/print/export implementation.

Project schema v8 adds an ordered sheets array alongside v7 metadata. V2–v7 load without invented sheets; explicit save writes v8. Older fixtures remain available and new v8 canonical outputs are tested. Sheet settings are outside electrical subsystem revision bodies. Title-block cells/bindings, template revisions, shared sheet defaults, multiple drawing sets, interactive paper placement, unified bulk sheet properties and output validation remain future work.

## Delivery checklist

- [x] Include the feature and define scope, variable ownership and portability rules.
- [x] Add project metadata, stable field catalog, exact history, versioned persistence and a bounded multiline editor.
- [x] Add personal/company profile storage and explicit previewed copy/update into a project.
- [x] Add sheets, paper settings, numbering, initial drawing-set order and one optional view placement per sheet.
- [ ] Add table/cell geometry and editing with stable identities and bulk properties.
- [ ] Add bindings, sheet-local values, missing/required diagnostics and override/reset behavior.
- [ ] Add standard/draft/cover/contents templates, shared blocks and contents pagination.
- [ ] Add pinned template revisions and explicit reviewed adoption.
- [ ] Add portable logos, output validation, frozen issue metadata and reproducible drawing output.

Each slice requires headless invariant/history/persistence tests and real bridge/browser workflows where UI is added. Native Windows/Linux execution and output/rendering parity must be measured independently.
