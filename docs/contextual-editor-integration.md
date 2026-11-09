# Contextual editor integration roadmap

Status: accepted direction, replacing the form-heavy template workflow. Selection ownership, hit priority, contextual inspection, field completion, selection-aware Split/Merge, simplified paper controls, shared template/annotation preview queuing, template click-shape placement, inline template text creation, multi-point template paths and shared font/colour controls are implemented. Later stages below are requirements, not shipped capabilities. See the implementation checklist for verification.

## User workflow

The inspector follows selection automatically. Nothing selected shows paper/template settings; cells show their contents and applicable formatting; artwork shows only that object's properties. A single workspace selection owns highlighting, inspector targeting and contextual operations. Clicking blank paper clears selection. Escape cancels the active tool first, then clears selection when the canvas owns keyboard input. Text/IME input retains its own keys. Right-click selects and focuses the same inspector; Properties buttons are not required for ordinary editing.

The target ribbon has permanent Insert (margin border, table, image, text) and Draw (line/polyline, rectangle, circle, ellipse) groups, with selection-specific controls in a reserved group. Keep shell/ribbon dimensions stable. File/library lifecycle commands remain available. Existing numeric artwork management and Apply/Revert controls are transitional until placement and field-edit transactions replace them safely.

Insert Text takes an anchor click, then edits multiline text/variables inline before committing creation once. Table insertion asks only for row/column counts before placement, with sensible initial sizes and interactive resizing. Image insertion chooses a portable asset before placement. Insert Border creates or selects the paper-linked visible border. Draw uses start/centre and finish clicks; polylines accept successive vertices and Enter finishes. Escape leaves no partial object behind. Lines, rectangles and ellipses now support start/finish clicks as well as dragging. Template text now accepts an anchor click or sized drag, followed by canvas-local multiline/variable editing and one creation commit. Template Path accepts successive clicked vertices; Enter finishes clicked points only, Backspace revises them, and Escape cancels.

## Integration stages and dependencies

1. **Selection and inspector foundation.** One typed template selection: paper, cell range or artwork. Interaction controllers retain gesture state and render projections, not competing selection authorities. Table interiors receive hits above background artwork. Selection resolves by stable identity after undo/reload. Inspector sections follow selection; selected artwork does not leave table handles active. Preserve pending fields rather than silently dropping them on selection changes.
2. **Field transactions and contextual actions.** Shared begin/validate/commit/revert lifecycle captures original target/revision. Finish valid fields before changing selection; invalid fields remain visible and recoverable. Group continuous edits into coherent undo operations. Preserve native dirty guards and text/IME undo. Typed template selection now reaches bridge/action context, with shared Rust eligibility for Split/Merge and target checks at dispatch. Explicit Apply/Revert remain fallback controls and support the transitional artwork catalog.
3. **Simplified tables and paper (implemented).** Split accepts one cell or a contiguous vertical stack of cells with matching left/right edges; subdivision counts are the only required input. Apply column splitting atomically. Merge accepts only a rectangular cell union; retain the top-left cell’s caption and contents by default, even when blank. Offer combining contents explicitly when other selected cells contain data. Reuse partition/identity validation and exact undo. Paper presents ISO A/US family, preset, orientation and general margin; Custom dimensions and separate margins live under Advanced. Preserve exact existing custom dimensions. No automatic scaling of drawings.
4. **Shared tool and appearance modules (in progress).** Shared preview queuing now serves template gestures and schematic annotations; template vector shapes support click–click placement. Schematic annotation finish waits for the final preview instead of dropping an in-flight click. Template cells, free text and new text now share one multiline field with inline placeholders and a button/right-click variable tree (decision 0053). New template text uses this shared editor in a bounded canvas overlay and commits after final preview validation. Template paths use complete ordered vertex proposals through the shared preview queue; path finish/back/cancel keys are shared with schematic paths. Shared font-family and colour choices now serve template artwork, schematic properties and drawing defaults, preserving exact values and owner-specific write semantics (decision 0051). Extract point collection, preview queuing, finish/cancel and inline text editing from working tools. Template adapters supply bounded paper coordinates; schematic adapters supply source-view coordinates, snapping and electrical restrictions. Share text rendering/appearance controls, variable-capable text runs and line geometry without flattening schematic inheritance or embedding project-specific bindings into generic text. Add style capabilities consistently through domain, DTOs, storage and rendering. Deliver improvements in both editors incrementally rather than building a speculative framework.
5. **Insertion and persistence extensions.** Add reviewed table insertion, independent border visibility/appearance and portable image assets. Margins remain a geometric guide even if the visible border is absent. Assets must survive library publication and project bundling/reopen without machine-local paths. New persisted features use explicit versioning and preservation tests; existing immutable template bodies and project pins remain unchanged until explicit adoption.

## Existing work to preserve

- Headless geometry validation, commands, exact undo/redo and separate template/project histories.
- Saved template pins, immutable library revisions and project-owned offline bodies.
- Structured table partitions, rectangular merge checks, cell IDs, variables, resizing and margin snapping.
- Closed revision-bound pointer previews, final-release semantics, monotonic input IDs and native lifecycle guards.
- Schematic style inheritance, explicit overrides, source-coordinate transforms and electrical topology rules.
- Shared multiline rendering and literal/variable editing already implemented.

## Required rework

| Current implementation | Resolution |
| --- | --- |
| Separate artwork selection and table/editor selection | One editor selection; controllers read it or receive projections. |
| Unfilled table interiors let underlying artwork receive pointer hits | Explicit full-cell hit areas in template view, consistent with paint order. |
| Complete template/artwork/cell form displayed together | Contextual sections with automatic visibility and stable pane bounds. |
| Whole-layout Apply blocks other actions while fields are pending | Contextual fields now complete through captured target/revision-bound proposals; retain explicit fallback and guards for structural/catalog operations. |
| Template action context has no selected cells/artwork | Implemented typed context and read-only Rust Split/Merge eligibility; extend to future contextual actions. |
| Split/merge dialogs repeat object selectors and expose IDs | Implemented: derive targets from selection; only counts/content policy remain. Merge still has an explicit final review step. |
| Separate schematic click-path and template drag-shape workflows | Shared tool state machines behind coordinate/command adapters. |
| Explicit artwork appearance versus inherited schematic styles | Common controls/types with owner-specific write semantics. |
| Border is always derived/rendered from margins | Separate visible border intent from geometric margin guides. |
| Portable PNG assets implemented | Shared controls/rendering, versioned embedded asset tables and transitive template bundling; click placement and advanced transforms remain follow-up. |

## Verification gates

Regression workflows must cover artwork → cells → artwork → blank paper, overlapping background artwork, range selection, right-click/explorer equivalence, Escape, pending-invalid edits, and selection changes after undo/deletion/reopen. Later stages add field-change ordering, typing/IME and grouped undo, eligibility across ribbon/palette/keys, shared template/schematic drawing behaviour, exact custom-paper preservation, and old-file/asset round trips. Continue local full/UI/desktop checks; native Windows/Linux and font/output parity require separate validation.

## Initial slice boundary

The first slice supplies contextual selection and inspector routing. The next slice completes contextual fields on Enter (Ctrl/Cmd+Enter for multiline), Tab/blur and choice changes, and completes valid edits before pointer selection changes. Invalid edits remain visible; Escape reverts pending fields. Apply/Revert remain as fallback. A temporary artwork-management view remains available for numeric insertion/removal. The reusable field-completion module supplies DOM input policy; template commands still own validation/history. Numeric artwork catalog edits remain explicitly staged. A pointer used to finish pending fields selects its destination; it does not also start a drag. Selection-aware Split/Merge and exact paper presets are now implemented; see decision 0047. Shared preview queuing is implemented (decision 0048); inline template text creation and multi-point paths are implemented (decisions 0049–0050), while further shared tool extraction and new insertion workflows remain pending. Paper presets do not imply verified margin-standard profiles. Historical decisions 0041–0046 describe delivered prototypes; this roadmap supersedes their future UI direction where it differs.

Table appearance and units extension: caption/content now use separate per-property defaults/overrides and shared appearance controls. Range formatting is one template edit; text editing requires a single-cell selection. Multi-cell and whole-table selections share one compact palette applying changed properties to both caption and content channels. Dimension controls convert user mm/in/µm to unchanged exact bridge contracts; measurement preferences do not edit documents. See [decision 0055](decisions/0055-table-text-styles-and-measurement-units.md).

### Shared font controls

Table caption/content, template text artwork, schematic text properties and drawing-default fonts use the same `FontPalette` and font-family list. The UI-only `FontProperties` contract exposes supported fields; each editor supplies its own adapter and retains validation, commit timing and undo ownership. Table text exposes its richer style/alignment fields; artwork and schematic fonts expose only supported fields. Custom and multiline family values remain editable. Default is available only for properties with a parent/default source. Drawing-wide colour and stroke retain their separate applicability to non-text objects.

Schematic font/height changes use a strict `edit_appearance` bridge request: string values assign explicit overrides and null removes overrides. A revision-guarded application batch preserves exact property state through undo and rejects invalid batches atomically. No persistence schema change is required.

### Row-specific table layout integration

The Tables ribbon now uses **Layout** instead of Resize: row count dynamically creates per-row column inputs, with explicit removal acknowledgement. This is a registered action with captured template revision and one core-validated layout/history replacement. Size changes remain pointer operations or the numeric Resize command. Contextual column insertion fits inside the existing width, resolving margin rejection for wide cells. See [decision 0057](decisions/0057-row-specific-table-layout.md); this supersedes earlier ribbon Resize and growing-column behavior.

Portable pictures now share browsing/property controls and rendering across schematic placements, template artwork and cells. Initial insertion reviews a placement dialog, then uses existing drag movement; click-to-place remains planned. See [decision 0058](decisions/0058-portable-pictures.md).
