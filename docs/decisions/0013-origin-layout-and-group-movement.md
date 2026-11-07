# 0013: Explicit origin layout and simultaneous connected-anchor movement

Status: implemented headless slice; interactive previews and renderer-dependent bounds remain planned.

## Problem

Multi-object movement, alignment, and distribution must preserve complete objects and connections. Moving connected endpoints sequentially can collapse a path temporarily even when the final translated route is valid. Layout also needs an explicit geometric basis before font metrics, symbols, or a renderer exist.

## Decision

`layout_plan` supports annotation positions, first vertices of whole graphical polylines, and terminal-anchor positions as explicit origins. It deduplicates selected identities, requires one view, and rejects missing or unsupported targets rather than silently moving part of a selection. Direct conductor-route selection is unsupported: select its terminal anchors to move the connected representation. Electrical devices have no inferred drawing origin.

Three operations are provided: signed numeric translation, origin alignment to an explicit coordinate on one axis, and origin distribution between current extrema on one axis. The other coordinate remains unchanged. Distribution requires at least three distinct objects, sorts by coordinate then typed identity, preserves extrema, and uses integer offsets `floor(span * index / intervals)`. Gaps differ by at most one micrometre; ties and negative coordinates are deterministic. There is no automatic grid rounding or persistent relationship. Origins are not estimated text extents or visual bounds.

Arithmetic uses checked signed document coordinates and wider intermediate differences/products. Overflow in any origin or interior path vertex rejects the complete proposal. Alignment may legitimately require a displacement larger than i64 while all resulting coordinates remain representable; wide intermediate offsets preserve that case.

Preflight produces normal commands, changed origins, unchanged selections, and the union of affected routes. A temporary full-project candidate validates the complete proposal. Callers submit the command batch at the revision observed during planning; plan data does not bypass revision checks. The live project remains unchanged on planning or commit failure.

`MoveAnchors` snapshots selected placements and each touching route once. When both endpoints move by the same offset, it translates every vertex, retaining all bends, collinear waypoints, bindings, and appearance. This includes a self-loop attached twice to one moved anchor. Other endpoint changes use the existing deterministic local leg/bend adjustment in A/B order. Differential movement that collapses intermediate geometry is rejected safely and requires explicit compatible whole-route replacement; this slice is not a general simultaneous routing solver.

`RouteLayout` restores exact local geometry. Restoration requires the complete touching-route union, no duplicate records, unchanged anchor terminals/views, and unchanged route identity, mode, view, electrical attachments, and appearance. All resulting geometry is validated against the complete proposed anchor-position map before mutation. Derived indexes remain valid because bindings do not change. Single-anchor commands delegate to this common machinery. Undo and failed-batch rollback use exact snapshots, never reverse offsets or recomputed routing.

Move, align, and distribute are separate shared registry actions with typed requests/replies and captured selection/view/revision. M requests movement; alignment and distribution have explicit unbound defaults and remain remappable/searchable. Availability reports unsupported or mixed-view selections and insufficient distribution targets. Actual numeric controls, pointer previews, and native keyboard wiring remain host work.

Project schema v6 and keymap schema v1 are unchanged: only existing positions/vertices are edited. No selection, plan, persistent constraint, or undo snapshot is added to the file format.

## Validation and revisit conditions

Workflow tests cover mixed selections, simultaneous movement through old endpoint positions, bent and shared routes, unchanged other views/electrical data/style intent, exact undo/redo, full-span distribution and identity ties, overflow, malformed snapshots, collapsed geometry, rollback/no-ops/checkpoints, strict save/reopen, and captured action dispatch. An executable example demonstrates origin layout with connected anchors. Release probes measure local one/two-anchor movement through 10,000 unrelated routes and separate origin-alignment preflight/commit/undo at 20/200/2,000 annotations. Layout timings are single samples including candidate cloning; they are not regression gates or UI capacity claims.

Revisit this contract when symbols, text metrics, locked positions, pinned waypoints, or persistent sub-element references are introduced. Add explicit bounds/edge/centre bases, equal edge gaps, fixed pitch, key-object/page references, and interactive preview/commit agreement as separately validated capabilities. Preserve the distinction between a layout constraint and electrical attachment throughout.
