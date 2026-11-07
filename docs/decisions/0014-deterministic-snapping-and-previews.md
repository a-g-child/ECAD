# 0014: Deterministic snapping and revision-bound geometry previews

Status: implemented headless movement/preview foundation. Native interaction, rendering, and IPC remain planned.

## Problem

Placement must have a predictable target, respect explicit numeric input, and preview the same geometry that commit will produce. A transient preview must not alter history or silently adapt to a changed document. Coincident graphics must not introduce electrical connectivity.

## Decision

The domain accepts explicit `SnapQuery` data: existing drawing view, pointer coordinates, source settings, inclusive circular tolerance in integer micrometres, and excluded representation identities. Hosts must convert pixel tolerance themselves. `SnapSettings` is transient input, not new persisted project styles or workspace-preference storage. Its default disables sources; callers provide their actual policy.

Candidates include placed terminal anchors, first/last route or graphical-path vertices, each segment midpoint, and the nearest representable point of an explicit origin/spacing grid. Source classes rank terminal, endpoint, midpoint, then grid. Within a class, exact squared distance wins, followed by typed target identity. Coincident candidates remain distinct for deliberate cycling. Route/path endpoint order and midpoint segment indices refer to the observed revision; they are not new persisted vertex IDs.

Tolerance is inclusive; zero permits exact coincidence only. Per-axis filtering precedes distance squaring, preventing overflow across the full signed coordinate span. Wider integer arithmetic computes midpoints and grid coordinates; midpoint components round down independently and grid half ties choose the smaller coordinate. At a coordinate boundary the nearest representable grid line is used. Invalid views return a typed error. Exclusions may name absent objects harmlessly; an excluded anchor cannot reappear through an attached route endpoint.

The first query scans geometry linearly and sorts accepted candidates. It has no spatial cache, hysteresis, alignment snaps, obstacle checks, inferred junctions, or global router. These are deliberate limits to measure before introducing indexes or more interaction rules.

The application provides immutable `LayoutPreview` and `MovePreview` records. Layout preflight produces one validated temporary project candidate and normal commands. Preview extracts exact records for the selected annotations, polylines, and anchors, plus the affected route union, then discards the full candidate. The retained proposal includes the original session epoch/sequence, view, selection, changes/no-ops, and closed commands. No mutable command access or persisted preview state is exposed. Dropping a preview is cancellation.

A move names an origin explicitly within the selection. Exact numeric placement bypasses snapping. Pointer placement requests best candidate, an explicit candidate index, or temporary suppression. Best falls back to the raw pointer only when no candidates exist. An unavailable explicit index is an error. An invalid winning layout fails explicitly rather than silently choosing a different target. The host can cycle or suppress deliberately. Numeric translation offsets must fit the existing signed i64 offset contract; extreme otherwise representable origin-to-destination differences can therefore reject.

Move queries exclude the entire selection and all routes touching selected anchors, including their midpoints. This prevents moving geometry from attracting itself. Unselected representations remain eligible only in the active view. Snapping changes destination coordinates only; terminal references and conductor attachments remain authoritative and unchanged.

Document/session commit consumes the closed layout proposal through normal revision-guarded batch execution. It does not query targets again or replace the preview's revision with the current one. A changed/different session rejects even a no-op preview. Existing deterministic command rules produce the projected geometry; one commit has one exact undo operation and the normal saved-checkpoint behavior.

The existing M action accepts preview parameters, returns the transient move projection, and accepts a reviewed move preview for commit. It checks captured selection/view as well as the preview's own revision. Immediate numeric movement remains supported. Native controls, feedback, cycling keys, and actual tool cancellation are still host responsibilities; the core exposes no new global listeners or shortcut path.

Project schema v6 and keymap schema v1 remain unchanged. No grid, snap target, cache, preview, or session command proposal is added to saved projects. Persisted workspace snap preferences require a separate future contract.

## Validation and revisit conditions

Domain tests cover priority/distance/identity ties, coincident identities, circular/zero tolerance, negative grid ties and offset origins, full-range arithmetic, midpoint rounding, view scope, and exclusions. Workflow tests cover self-snap suppression, cycling, exact numeric placement, invalid targets/layouts, projection/commit equality, generic alignment/distribution previews, cancellation, stale/different-session/no-op guards, exact undo/redo/checkpoints, save/reopen, and shared M replay/context checks. An executable placement example demonstrates the full path. Separate scale cases time 100 endpoint/midpoint queries over 100/1,000/10,000 graphical paths and verify exact deterministic results.

Revisit with real pointer interaction and representative dense harness drawings. Measure complete preview latency, not just raw query time. Establish screen-space tolerance, hysteresis/cycling behavior, background budgets, and cache invalidation before adding a spatial index. Terminal placement, route creation, constraints, and symbol geometry must adopt the same explicit attachment and preview/commit rules as those features arrive.
