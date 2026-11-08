# 0030: Project documents and direct template-backed sheet editing

Status: accepted target architecture, clarified by the user on 2026-11-08. Documentation only; implementation remains at the v8 prototype. Supersedes the free-Model-view/read-only-paper workflow and flat sheet ownership in decision 0029.

## Decision

Organize actual drawings as Project → Documents → Sheets. A logical drawing document owns an ordered collection of sheets, shared drawing-package metadata and a default template. It is separate from Document<S>, the existing application session/save controller, and does not imply splitting the authoritative project into separate files.

Each actual drawing is edited directly on its CAD sheet, with its template border and drawing information visible. This covers cable overviews, hierarchical overview sheets, detailed conductor/connector schematics and project layout drawings. Cover and contents sheets also belong to documents. Each sheet must resolve an explicit bundled template or a document default, with a basic standard frame available from the first implementation. No free-canvas project schematic mode is intended.

Drawing representations, paper/template geometry, electrical objects and viewport state retain separate ownership. Internal view/placement structures can support coordinates, scale and reuse, but must not force users to leave a sheet to edit its schematic. All pointer positions, snapping and committed edit coordinates pass through the validated sheet transform; paper/template changes must not move or rewire source objects. Existing command/history/revision guards remain authoritative.

Symbol, generic component/device, footprint and template-definition editors are different workspace kinds, with appropriate origins, units, grids and preview contexts. They are not schematic sheets and do not receive project drawing borders. Their library definitions remain distinct from placed electrical occurrences and sheet representations.

Electrical functional hierarchy and document/sheet organization remain independent. A hierarchical block references its target sheet by stable identity and its subsystem/interfaces explicitly. Renumbering or moving a sheet between documents must preserve links and connectivity. Output/issue drawing sets select documents/sheets; they do not replace document ownership.

## Metadata and identity

- Stable document IDs are distinct from sheet IDs, view IDs, filenames and drawing numbers.
- Each sheet has one owning document. Preserve explicit order and editable identification independently of derived ordinal/page count.
- Add document-scoped title/number/revision and template defaults alongside existing project/profile values and sheet-local title/entries. Bindings select their scope explicitly; captions do not identify fields.
- Templates and required drawing information remain portable project-owned snapshots or pinned revisions. Resolving a sheet must not depend on the current user's external profile/library changing.
- Define numbering uniqueness at the appropriate scope without silently reinterpreting existing v8 sheet.number values during migration.

## Implementation sequence and migration safeguards

1. Introduce the logical document model, stable ownership/order and commands/persistence. Preserve existing public session-controller responsibilities.
2. Add a reviewed, deterministic migration plan for v8 and earlier content. Existing sheets retain identities/identification and transforms. Unplaced schematic views become sheet-backed content rather than disappearing or remaining a normal free-canvas mode. Migration must not generate new IDs on every ordinary decode; retain its allocated identity map across retries.
3. Resolve views previously shared by several v8 paper placements explicitly. Preserve their sharing/appearance or provide an explicit reviewed materialization policy; do not silently create independent copies or duplicate electrical objects.
4. Provide a basic bundled frame/drawing-info template and unify sheet rendering and input. Open project drawings from the document/sheet explorer; remove the normal Model view/read-only preview switch. Retain arbitrary paper/scale precision and existing editing operations.
5. Add the structured table/template editor and richer variable bindings against this sheet-first model. Keep definition-editor workspace kinds separate.

## Required evidence

Test exact migration of old free views, placed/blank sheets, shared source views and electrical bindings; deterministic identity mapping; strict schema boundaries; document/sheet ordering and moves; numbering preservation; template resolution; undo/redo/rollback/checkpoints; cross-sheet hierarchy links; and geometry-preserving template/paper updates.

Real browser tests must create/open and edit an actual sheet, including wires/paths/annotations, selection, drag, numeric placement, snapping, zoom and transformed origins/scales. Verify there is no need to switch to a free Model view and that definition editors retain independent contexts. Native/platform and output parity remain separate checks.

The current code is not changed by this decision. Existing v8 tests still describe its implemented behavior; they must be replaced or extended alongside the migration and unified sheet editor, not relabelled as proof of the new workflow.
