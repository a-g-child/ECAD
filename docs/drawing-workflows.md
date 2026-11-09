# Drawing and editing workflows

Current editor integration direction: [contextual editor roadmap](contextual-editor-integration.md). It supersedes earlier form-heavy template UI proposals: one selection owner, automatic contextual properties, simplified Insert/Draw tools, shared field/tool modules and explicit persistence evolution. Historical implementation sections remain records of their respective slices.

Status: Headless views, annotations, graphical polylines, electrically anchored whole conductor routes, portable defaults/styles, and property edits are implemented. Route commands support indexed vertex/segment editing and local first-leg/bend anchor movement with exact undo. Numeric origin translation/alignment/distribution and simultaneous connected-anchor movement are also implemented. Headless terminal/endpoint/midpoint/grid snapping, exact pivot-based quarter-turn/mirror transforms, and revision-bound layout/move previews are implemented. The desktop prototype now creates annotations through T with real text presets and reviewed snap/numeric placement. P now creates retained whole graphical paths with accepted/tentative segments, draft revision/cancellation and one-step creation. It renders these objects and integrates M with exact/pointer group movement, snap feedback/cycling/suppression, and closed preview commit/cancel. Other interactive tools, visual-bounds layout, symbols, live label attachment, persistent constraints, alignment snapping, crossing/hop presentation, and the broader workflows below remain proposed. See [route decision 0012](decisions/0012-whole-conductor-routes.md). See [decision 0010](decisions/0010-drawing-defaults-and-styles.md). Complements the [product scope](product-scope.md); detailed interaction choices and implementation remain open.

Target clarification: project drawings are edited on template-backed CAD sheets organized as Project → Documents → Sheets. This includes overview and detailed electrical drawings; internal drawing views are not a separate free-canvas workflow. The v8 Model view/read-only paper split is an implemented prototype to replace. Symbol/component/footprint/template-definition editors have separate workspace contexts. See [decision 0030](decisions/0030-sheet-first-project-documents.md).

## Design intent

Make precise drawings easy to create and easy to revise. Users should be able to predict an operation before committing it, select an object at the right level, and change its geometry without reconstructing it from unrelated segments.

Take inspiration from the explicit geometry and constraints of a sketch editor and the keyboard-driven placement and connectivity tools of schematic CAD. A complete mechanical sketch solver is not required to deliver predictable electrical drawing tools.

## Structured drawing objects

Distinguish the following concepts:

| Object | Editing behavior |
| --- | --- |
| Electrical connection | Retains its terminal references and electrical identity independently of the shape used to display it. |
| Route | An ordered path representing a connection in a particular view, with editable vertices, segments, and endpoint anchors. |
| Graphical polyline | An editable path with no implied electrical meaning. |
| Shape | A rectangle, circle, arc, or other supported primitive that retains its defining parameters. |
| Symbol instance | A positioned representation linked to a device, with explicit terminal and label anchors. |
| Group | A collection that can be transformed together without changing the electrical identity of its members. |
| Array | A repeat-placement definition with a source, count, spacing, and instance membership. |
| Image attachment | A document object referencing an asset, with placement, crop, and display properties. |

Internal use of segments is acceptable; users must not be forced to manage every segment separately. Branch points are explicit topology boundaries. Selecting a route, a connected branch, or a complete network must be distinguishable operations rather than an ambiguous definition of "whole wire".

Every text value supports multiple lines without changing object type. Desktop text editors use Enter for a newline and Ctrl/Cmd+Enter or Apply to commit. Blank lines and indentation render within the same annotation, with complete text-block selection/Fit bounds. See [unified multiline text](decisions/0022-unified-multiline-text.md).

Labels should support attachment to their owning symbol or route, with editable offsets. Deliberately moving a label must not silently detach it from its owner.

## Predictable interaction and selection

- Clearly identify the active tool, selection level, snap target, and operation being previewed.
- Select whole objects by default; enter vertex, segment, or group-member editing deliberately.
- Allow expansion from a segment to its route and, separately, to a connected branch or network.
- Provide selection filters, lock controls, and cycling through overlapping candidates.
- Implemented desktop: left-to-right window selection encloses complete painted geometry; right-to-left crossing selects touched objects. Shift adds results; Escape cancels. Empty primary drag selects; Shift-scroll pans horizontally, Ctrl-scroll vertically, and unmodified scroll zooms at the cursor. Middle-button drag also pans. Direct object dragging preserves the grabbed offset and selected group, then commits one closed Rust proposal on release. See [pointer controls](decisions/0019-cad-pointer-controls.md).
- Preserve the current selection for successive property, alignment, and transform operations where practical.
- Preview operations before committing. Escape cancels the active preview without leaving partial document changes; define consistent behavior for ending a tool and clearing selection.
- Make each completed operation a coherent undoable command. Copy, paste, duplicate, grouping, and transforms follow the same command rules.

Nested groups, routes, and device symbols require explicit hit-testing priorities. Reference images and locked background geometry should not intercept ordinary electrical selection unless intentionally targeted.

## Implemented orthogonal group transforms

Quarter-turns and reflections use explicit pivots in micrometres, never an implicit selection centre. Positive 90 maps relative `(x, y)` to `(-y, x)` in document coordinates; visual clockwise naming depends on the future viewport convention. Vertical-axis mirroring changes x about `pivot.x`; horizontal-axis mirroring changes y about `pivot.y`. Every selected graphical path vertex transforms. Routes transform completely when both anchors are selected, including self-loops at the pivot; other connections adjust their moved endpoint locally. Electrical bindings, ordered endpoints, view membership, and appearance stay intact. Annotations transform position while glyphs remain upright.

A fixed origin is not proof of a no-op: reports separate changed geometry from changed origins and include actual route changes. Exact closed previews support all layout actions, with R for rotation requests and a remappable unbound mirror action. Overflow and invalid local routes reject atomically; undo restores stored geometry rather than applying an inverse transform. Desktop R/mirror controls now expose exact numeric pivots and reviewed preview/commit/cancel in the reserved tool strip. Arbitrary angles, scaling, symbols, text orientation, and pointer-picked pivots remain planned. See [decision 0015](decisions/0015-orthogonal-group-transforms.md).

## Coordinates, snapping, and constraints

Implemented subset: one view supplies terminal anchors, whole-path endpoints, rounded segment midpoints, and the nearest representable grid point. Rank those source classes in that order, then squared distance and typed identity. Tolerance is an explicit inclusive circle in document micrometres. Grid half ties choose smaller coordinates; midpoints round down independently on each axis. Numeric movement bypasses snapping. Pointer movement supports explicit candidate cycling/suppression and excludes selected geometry plus routes touching selected anchors. Snap coincidence creates no electrical attachment. Immutable previews expose the exact proposed selected records and affected routes; commit uses their original session revision, including for no-ops. Dropping a preview cancels without document edits. See [decision 0014](decisions/0014-deterministic-snapping-and-previews.md). The desktop now converts screen tolerance and shows snap feedback for move/annotation placement. Native interaction validation, hysteresis and alignment snapping remain future work.

Store geometry in defined document units independently of screen pixels and zoom. Support numeric positions, offsets, rotation, and dimensions through tool input or properties.

Provide grid, endpoint, terminal, midpoint, and alignment snapping as appropriate to the active tool. Resolve candidates using documented priorities and deterministic tie-breaking. Highlight the winning target and allow cycling or temporary snap suppression. Snap tolerance should be understandable at different zoom levels, without moving the target as the pointer remains nearby.

Support explicit orthogonal drawing and, where useful, 45-degree and unrestricted modes. Users can switch bend orientation during route placement and see the resulting path before clicking. Numeric input must take precedence over an incompatible snap, or clearly explain the conflict.

Distinguish temporary placement aids from persistent constraints. Snapping to a drawing alignment must not silently create a permanent relationship. Electrical endpoint attachment is a separate, explicit relationship.

Candidate initial persistent relationships are locked positions, horizontal/vertical segments, endpoint attachment, and label or image anchoring. Alignment and distribution can initially be one-shot commands. More extensive equal-spacing, dimensional, or parametric constraints require a separate scope decision.

If constraints cannot all be satisfied, report the conflict and retain the last valid state. Do not silently shift unrelated objects or discard constraints. Given the same document, settings, and operation inputs, committed geometry should be reproducible.

## Alignment and distribution

Implemented headless subset: annotation positions, first vertices of graphical polylines, and placed-terminal positions act as explicit origins. Numeric alignment preserves the other axis. Distribution fixes the extreme origins, sorts coordinate ties by typed identity, and rounds fractional offsets down to micrometres; gaps differ by at most one micrometre. Complete selections are deduplicated and must belong to one view; unsupported targets are reported. Connected routes move through selected anchors. These operations create no persistent constraints and do not apply hidden grid rounding. See [decision 0013](decisions/0013-origin-layout-and-group-movement.md). The desktop now exposes these origin operations through Drawing, Edit and command search, with exact coordinate/axis controls, retained previews and one-step history. See [decision 0025](decisions/0025-desktop-origin-arrangement.md). The broader requirements below remain planned.

Support left/right/top/bottom alignment, horizontal/vertical centres, equal centre spacing, and equal edge gaps. Let users choose whether alignment uses object bounds, origins, or meaningful anchors such as terminal rows. State the basis visibly.

Choose an explicit reference: a key object, selection bounds, page, or numeric coordinate. Do not rely on an undocumented selection order. Distribution should distinguish equal gaps from equal pitch; offer a fixed pitch where useful. Locked objects and connected routes need a defined policy and visible applicability.

Alignment may create coordinates outside the placement grid. Define whether exact alignment or grid adherence wins; do not quietly round the result so the requested alignment is lost.

When electrical symbols move, the operation uses a visible connection-preservation policy. Alignment is a layout operation and should not accidentally reassign terminals or create new junctions.

## Repeat placement and arrays

Start with linear and rectangular arrays: count, direction, pitch, and preview. Retain the array definition so users can change spacing or count after placement, rather than editing every copy. Radial arrays and elaborate pattern constraints can follow later.

Distinguish independent duplication from a retained array. Array instances need stable identities; increasing count should not regenerate existing instances. Reducing count must report affected electrical connections and instance overrides before removal. Provide an explicit way to release instances into independent objects.

Copying a symbol can mean creating a new device or adding another representation of an existing device. Make that choice explicit. New devices need new identities and designation handling; ordinary duplication must not inherit the same device identity accidentally.

Do not duplicate electrical connections to external objects merely because the selected source is wired. Define separately whether duplication includes internal wiring within a selected assembly. Repeated functional assemblies should eventually use reusable subsystem definitions, rather than relying solely on graphical arrays.

## Route creation and later editing

Users can move a whole eligible route, drag a segment, reposition a bend, insert or remove a waypoint, and replace a selected portion of a route. Preserve the identity, conductor assignment, labels, and terminal attachments during geometry-only edits.

Connection-preserving movement adjusts the affected route locally. Keep unaffected bends stable and allow users to pin waypoints. Candidate routing rules should favor a small, reproducible change and must not globally optimize the sheet on every move. If an acceptable local adjustment cannot be produced, show the problem and allow manual correction or cancellation.

Moving connected devices defaults toward preserving their terminal connections. A distinct detach or reconnect operation makes electrical changes visible. A free move of graphical objects has no implied electrical effect. Resolve the exact shortcuts and naming during interaction design.

Merging collinear segments and removing redundant points must preserve branch points, pinned geometry, endpoint references, and label anchors. Cosmetic simplification must not change electrical topology. Multiple views of the same electrical object retain independent route geometry.

## Crossings, junctions, and wire hops

A geometric crossing alone does not establish electrical connectivity. Creating a junction is an intentional electrical action with visible feedback. Endpoint placement on an existing route may offer connection creation, but the preview must make the resulting junction explicit.

Support project conventions for plain crossings, bridge/hop arcs, and gaps. A hop is a derived drawing treatment of a non-connected crossing, not an unrelated arc that users have to maintain manually. It must not change physical cable length or electrical properties.

Define a deterministic rule for which route receives the hop, with an explicit override where needed. Size and appearance inherit project styles. Recompute crossings after route edits and use the same result in print/export. Connected junctions must never be presented as non-connected hops.

Crowded crossings, overlapping collinear routes, and crossings too close to terminals need defined diagnostics or fallback presentation. Do not use a hop to hide ambiguous geometry.

## Images and reference drawings

Users can insert images through file selection, paste, or drag-and-drop. Choose supported formats during requirements work. An attached reference image has no electrical connectivity.

Copy the original image asset into the project and keep its placement as a separate text-based object. The initial PNG slice embeds a deduplicated asset table in atomic JSON saves; binary sidecars remain a future packaging change ([decision 0058](decisions/0058-portable-pictures.md)). Reference one asset from several placements without duplicating its bytes. With future sidecar packaging, Git can track the image as a binary asset while reviewing position, crop, and other placement changes as text.

Support aspect-preserving resizing, numeric dimensions and rotation, non-destructive cropping, opacity, stacking order, locking, and visibility. Distinguish crop from resize in the handles and properties. Replacing an image should retain its placement and explicitly define whether its size or aspect ratio is preserved.

Allow page-relative placement and optional anchoring to a symbol or group with an editable offset. An anchored image follows its owner. Define what happens when its owner is deleted rather than leaving an unexplained dangling reference.

Reference backgrounds can be locked and excluded from normal selection. If tracing to a known scale is in the first target workflow, provide calibration against a known distance. An image background still does not supply electrical snap points automatically.

Make print/export inclusion explicit so a tracing background can remain visible in the editor without appearing in final drawings. External image links, if supported, must be explicit and report missing assets; do not create hidden absolute-path dependencies on one user's workstation.

## Proposed delivery slices

1. **Editing foundations:** Whole-object and sub-element selection, grid and terminal snapping, numerical placement, structured paths, local route editing, basic shapes, grouping, alignment/distribution, connection previews, and undo/redo.
2. **Practical drawing tools:** Portable image insertion and transforms, repeat placement, retained linear/rectangular arrays, and derived crossing treatments. Keep their data-model needs in view from the start.
3. **Further parametric tools:** Rich persistent geometric constraints, radial patterns, advanced obstacle-aware routing, and reference-drawing calibration if not required earlier.

These are sequencing proposals, not a commitment to include every tool in the first end-to-end electrical milestone.

## Acceptance scenarios

- Draw a five-segment connection, move its middle segment, add a waypoint, and reopen the project. It remains one editable route with the same terminal references and conductor assignment.
- Move a wired device while preserving connections. Unaffected bends remain stable, terminal attachments remain correct, and one undo restores the operation.
- Align six devices to a chosen terminal reference and distribute them at a numeric pitch. Repeat after changing zoom; document coordinates are unchanged.
- Create a rectangular device array, change pitch and count, and verify retained instance identities and explicit handling of any removed connected instances.
- Cross two unrelated routes. Show a hop, move the crossing, and verify the hop follows it without creating a junction. Then intentionally create a junction and verify the hop disappears.
- Insert and crop an image, lock it as a reference, anchor another placement to a group, and move the group. Reopen on another workstation without the original source path and verify the assets and transforms remain available.
- Cancel an in-progress draw or array operation; no partial objects or accidental connections remain. Undo and redo each completed edit as a whole operation.
- Review a route or image-transform change in Git. Geometry edits do not rewrite unrelated electrical definitions or duplicate unchanged image assets.

## Architecture questions to resolve

- How are branch topology and per-view route geometry related without confusing route selection with electrical network selection?
- Which coordinates, tolerances, and serialization rules preserve precision and stable diffs?
- Which lightweight constraint and local routing rules deliver the initial editing experience?
- How do anchoring, array membership, grouping, and styles compose without circular dependencies?
- Which geometry and hit-testing services are shared between interactive rendering and export?
- How does a preview remain responsive while the committed operation passes through the authoritative command and validation model?

The current shared PNG slice supports browsing, exact placement properties, fit/fill/stretch, replacement/removal and translation in schematics and templates, plus cell-bound pictures. The broader image workflow above remains target scope; paste/drop, rotation, explicit crop handles, anchoring/locking and output parity are not yet implemented.
