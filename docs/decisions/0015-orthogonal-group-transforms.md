# 0015: Exact orthogonal group transforms and fixed-origin geometry changes

Status: implemented headless quarter-turn/mirror slice. Symbols, glyph orientation, arbitrary-angle transforms, scaling, and native controls remain planned.

## Problem

Group rotation and mirroring must retain complete path geometry and electrical identity. Moving each terminal independently loses internal route shape. Comparing origins alone misses edits to paths rotating about their first vertex and self-loops rotating about a stationary anchor.

## Decision

`DrawingTransform` defines exact quarter-turns and horizontal/vertical-axis reflections around an explicit document pivot. Positive 90 maps relative `(x, y)` to `(-y, x)`, half turns negate both coordinates, and negative 90 maps to `(y, -x)`. These are document-coordinate conventions; native clockwise naming depends on the eventual viewport's y-axis. Vertical-axis mirrors reflect x about `pivot.x`; horizontal-axis mirrors reflect y about `pivot.y`. Integer micrometres, wider intermediate arithmetic, and checked final coordinates avoid rounding and overflow. No bounding-box centre or implicit pivot is inferred.

The application extends the existing layout operation with `Transform`. Existing deduplicated, single-view selection rules apply. Every selected polyline vertex transforms, even when its origin stays fixed. Annotations transform their position only, retaining upright glyphs, literal text, and appearance intent. Terminal anchors transform their positions without changing electrical references.

A dedicated localized route command transforms the selected anchor set and each touching route once. If both route anchors are selected, transform every vertex and retain their ordered A/B references, mode, view, conductor identity, collinear waypoints, and style. This also handles both ends of a self-loop attached to one anchor at the pivot. Singly selected ends use the existing deterministic local leg/bend adjustment, preserving unaffected geometry. Free electrical ends are not selected anchors and are not implicitly carried along. Explicit whole-route replacement remains available when the conservative boundary adjustment cannot produce valid geometry.

The complete affected layout is validated before mutation against the proposed anchor positions. Reverse indexes remain valid because bindings do not change. Undo and batch rollback restore exact local snapshots. An inverse quarter-turn or reflection is not a substitute for undo: local boundary adjustment need not be mathematically invertible. Repeated full internal transforms preserve whole-path vertex order without simplification.

Plans now report changed selected record geometry separately from changed origin coordinates. `unchanged` identifies selected records whose own geometry remains unchanged, even if a connected route changes. A stationary anchor can therefore accompany a real transformed self-loop. Plan preflight removes command-level no-ops and reports only actually changed touching routes. A path at its pivot still produces an edit when any vertex changes.

Generic closed layout preview/commit parameters now work across movement, alignment, distribution, rotation, and mirroring. They use the same validated candidate and original session revision. Each handler validates that the proposed operation matches its action and retains the original selection/view; the host cannot relabel a mirror preview as rotation or supply a fresh context to bypass its revision guard. Existing M-specific snapped preview behavior remains supported.

R requests a quarter-turn and explicit pivot. Mirroring is registered, searchable, and remappable with a deliberate unbound default. Invocation DTOs/errors are separated from dispatch execution to keep source responsibilities manageable. No new shortcut listeners or alternate editing boundary is introduced.

Project schema v6 and keymap schema v1 remain unchanged. Only existing positions and ordered vertices are persisted; no transform, preview, selection, or session history is added to saved data.

## Validation and revisit conditions

Tests cover exact coordinate conventions, full-span arithmetic and overflow, four-turn/double-mirror cycles, whole internal paths, collinear vertices, unchanged electrical bindings/style intent, fixed-origin paths, pivot-anchor self-loops, partially selected routes, independent views, no-op/rollback/revision behavior, exact previews, typed action requests and closed-proposal guards, and save/reopen/checkpoints. The executable transforms example verifies the full document flow. Release probes add 1,000 two-anchor half turns at 100/1,000/10,000 unrelated routes and separate quarter-turn annotation-position preview/commit/undo over 20/200/2,000 objects. These are headless measurements, not native interaction or capacity guarantees.

Revisit when symbols, pinned geometry, locked positions, live label attachment, or text orientation are introduced. Those features need explicit ownership and orientation policies. Arbitrary angles and scaling require their own precision, rounding, stroke/text scaling, and orthogonal-route compatibility decisions. Native previews, pivot picking, and transform controls still require real-host validation.
