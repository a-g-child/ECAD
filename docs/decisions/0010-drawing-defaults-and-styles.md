# 0010: Drawing defaults and per-property style inheritance

Status: accepted for headless drawing views, annotations, and graphical polylines.

## Decision

Add a project-owned `DrawingDocument` aggregate separate from electrical objects. Views, annotations, polylines, and named styles have distinct typed IDs. Annotations carry literal text/position; polylines retain one ordered path and do not imply electrical connectivity. Use integer micrometres for signed positions and positive text heights/stroke widths. Output RGB and requested font families are document data, separate from theme, zoom, and personal UI preferences.

Use complete project fallback values, sparse named styles with fixed text/stroke role, optional sparse view overrides, and per-object appearance containing a style link plus sparse overrides. Resolve each applicable property independently in this order: object override, view override, named style, project fallback. Colour is a compatible shared graphical property; font/text height and stroke width remain distinct. Validate style links and roles even where explicit overrides currently mask inherited values. Deleting referenced styles/views is rejected; creation presets also retain style references.

Keep creation presets separate from linked style values. A creation command copies the current initial style link and override patch into a new object. Editing a preset does not rewrite existing objects. Imported/undo-restored objects retain their exact appearance. Updating a named style, a view override, or project fallback changes only inherited effective values; dependent records are not rewritten. A full settings command can deliberately change shared fallbacks and future-object presets coherently.

## Property and history semantics

Extend the typed registry with graphical fields, typed lengths/colours/positions/vertices/style links, and effective per-property origin. Inspect the union of electrical and graphical selections; expose uniform/mixed values and each object's origin separately. Equal appearance may still have different inheritance intent. Literal text/geometry/view/style-link fields remain ordinary values; inheritable fields create explicit overrides through normal inspector assignment.

`PropertyState` records literal values or the presence/absence of an explicit override. Assigning a value equal to inheritance is an actual intent change. Repeating an already explicit identical value is a no-op. Reset removes only that property's override; it preserves the style link and other overrides. Preflight/commit/history compare exact state, not only resolved values. Localized inverse commands restore both state and value. A mixed patch may combine assignments and resets with the same applicability/atomic failure rules as electrical properties. Shared-style commands are separate from object-property assignments, making their wider effect explicit.

## Persistence and boundaries

Project schema v5 retains the flat v4 electrical fields and adds a strict `drawing` aggregate containing portable settings, styles, views, annotations, and paths. Canonical identity ordering and explicit null override fields preserve intent. Reject duplicate IDs, missing/wrong-role links, blank required values, nonpositive lengths, malformed geometry, unknown fields, and unsupported versions. V2–v4 load with unchanged electrical IDs/references, empty drawing content, and portable defaults; explicit saves produce v5. Retain old fixtures and add empty/styled v5 golden fixtures.

Library schema v1 and its strict electrical revision-body v3 remain unchanged. Reusable electrical bodies explicitly reject nondefault drawing content/settings before publication, preventing silent loss through the electrical-only materializer/DTO. Drawing-aware subsystem capture/instantiation needs a dedicated future mapping and compatibility design. Project-level drawings remain intact through electrical revision adoption.

This slice provides headless annotations and graphical paths, not device-linked live labels, symbols, electrically anchored routes, snapping, transforms, rendering, printing, or CAD import/export. Font availability, substitution, embedding/licensing, glyph measurement, and cross-renderer parity need an explicit resolver/policy in the rendering slice. Do not claim identical rendered output merely from deterministic stored style requests. User presets for new projects and view-specific creation presets remain future work.

## Cost and verification

Shared-style replacement changes one record; effective values are resolved on demand without rewriting dependents. Bulk preflight still clones a temporary project; history stores localized records/property states. Existing full-project validation now includes drawing references. Measure shared update, resolution, mixed graphical edit/undo at 20/200/2,000 objects with exact restoration and unchanged-dependent checks. Single samples are exploratory, not capacity or timing guarantees; memory/rendering/large-vertex-path costs remain to measure.

Contracts cover precedence at every layer, independent properties, creation snapshots, shared fallback updates, equal overrides/reset/history, mixed origins/applicability, role/reference/in-use constraints, geometry/units, focused persistence, checkpoints, atomic rollback, canonical fixtures, legacy upgrades, and prevention of reusable drawing loss. A temporary-file example demonstrates the distinction between copied defaults, linked inheritance, and explicit intent. Current execution is macOS/aarch64; Windows/Linux and native rendering remain pending.
