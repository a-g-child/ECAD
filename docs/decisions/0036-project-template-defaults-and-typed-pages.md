# 0036: Project properties, drawing-template defaults and typed new pages

Status: implemented for the single-project desktop. Project context menus now expose Properties and Add new schematic/cover/drawing-list sheets. Custom template authoring and PDF export remain separate work.

## Ownership and templates

Project properties combine portable company/project metadata with three creation-template defaults: schematic, cover and drawing list. Apply uses the existing registered project-properties action and one application transaction; Undo restores metadata and defaults together, while no-op Apply retains redo/checkpoints. Profile copying remains explicit and separate. Unknown template identities fail before either part is edited.

Defaults are creation intent. New drafts copy the chosen immutable template into an explicit sheet override, including equal-valued choices and the existing generic sheet-manager entry point. Changing project defaults does not restyle existing sheets. Legacy sheets keep a nullable override and inherit their owning document's mandatory template; users can explicitly reset to this inheritance in sheet settings. Existing document defaults remain intact. This avoids unexpectedly changing issued/arranged pages when someone chooses a new starting template.

The bundled catalogue now has standard-v1 (180 × 40 mm bottom-right information block), cover-v1 (180 × 80 mm centred information panel) and drawing-list-v1 (180 × 20 mm four-column footer). All use real paper-space geometry and the existing eight live fields. A role and a template are distinct: any bundled frame may be deliberately selected for a role. Too-small paper retains its border and reports that the information panel cannot fit. Project appearance fallbacks still control font, value height, stroke and colour. Custom tables, logos, required fields, wrapping and verified font/output parity remain future work.

Sheet roles are schematic, cover and drawing-list. Covers and lists cannot carry a source-view placement. Legacy blank sheets remain schematic; titles and view presence are never used to guess intent during migration. Source geometry and electrical identities stay separate from paper/template/role metadata.

## Captured project context and creation

Right-click the project root for Properties or Add new…; three registered unbound drawing.sheet.new_* actions also appear in File and command search. The popup captures the project revision with no electrical selection/view/sheet target. Existing draft/action guards protect the context menu. The full sheet review permits number, multiline title, paper, template and an explicit owning document; ownership changes on an existing sheet remain dedicated document operations, not implicit property edits.

New schematic drafts prepare a fresh independent drawing-view identity. Successful project replacement discards retained creation-view drafts, even when the new file has matching project/document lineage IDs. Preview validates view insertion plus sheet insertion in a cloned candidate, without adding either to the live project. Apply commits the closed command sequence as one undoable transaction and activates the created page. Cancel creates neither; Undo removes both creation records in dependency order and Redo retains their IDs. Normal later sheet removal still retains its source for reviewed recovery, as before. Cover/list creation does not create a source view or duplicate physical components.

The bridge retains validated proposals at their original revision. Candidate frames and generated list rows are part of the reviewed projection; stale/superseded/cancelled proposals cannot apply an older state. Exact editor values and identities cross the bridge as strings. Display-only geometry uses floating-point millimetres. Action-result projection was moved into its own controller submodule to keep orchestration manageable.

## Generated drawing lists and navigation

List entries derive from all actual non-list sheets in the project, including covers, with stable sheet/document IDs and literal number/title text. Sort by natural ASCII-digit drawing number, then literal title/ID for deterministic ties. Arbitrarily long digit runs compare without integer parsing/overflow. Generated text is never stored as duplicated project records or counted as extra electrical content.

Each physical list sheet consumes the next entries in document/sheet order. Capacity derives from its actual frame/paper content region and inherited text height, so different list-page formats can coexist. Existing list sheets exclude themselves. If their combined capacity is insufficient, every list projection reports the remaining count; the sheet heading/badge warns users to add more list pages. A zero-capacity page reports no room rather than pretending the index is complete. Add another drawing-list sheet explicitly; no unreviewed pages are inserted by background regeneration. Reorder, rename, removal and paper/font edits recompute the table from current state.

Rows are full-area pointer and keyboard links in the editor and retain stable destinations after renumbering. They use the existing guarded sheet navigator; the numeric review shows noninteractive table content. Literal multiline cell text is clipped/marked on overflow rather than silently shrunk. Output subsets, automatic list-page creation and richer table layout remain future work.

These targets are groundwork for PDF link destinations and outlines. This slice does not create PDF files, output bookmarks or PDF link annotations. A future exporter must map the selected output sheets to stable destinations and report unavailable/omitted targets, rather than use mutable drawing numbers as identity.

## Persistence, evidence and limits

Schema v11 adds project template_defaults and sheet kind/template fields. Separate strict v2–v10 decoders remain; upgrades supply deterministic creation defaults and preserve existing sheet IDs, source geometry, placements and inherited appearance without load-time edits or random IDs. Unknown roles/templates and invalid role/source combinations fail validation. Electrical reusable bodies reject custom project template defaults rather than silently discarding them through their distinct schema.

Local checks cover defaults/property history and no-ops, independent view/sheet creation/cancel/undo/save/reopen, exact migration/validation, natural ordering, capacity/overflow, live stable destinations and reusable-body boundaries. Browser workflows cover project right-click properties, template pinning, fresh editable schematics and generated list pointer/keyboard navigation with rename/undo. All tests remain local with real Rust state and temporary project files.

The desktop still opens one project. Multi-project host routing, project-scoped tabs, custom template libraries/editors, PDF generation and native Windows/Linux interaction are not established by this slice.

Local macOS/aarch64 verification: cargo xtask full (340 behavior tests and two doctests), cargo xtask ui (41 browser workflows), cargo xtask desktop and embedded-asset debug native build. Browser tests are not native interaction or PDF-output automation.
