# 0012: Whole conductor routes with explicit terminal anchors

Status: implemented headless foundation. Interactive placement, symbols, snapping, constraints, and crossing presentation remain future work.

## Problem

Independent line segments lose connection intent and make later editing unpredictable. A drawing must retain a whole path without duplicating its electrical conductor, and moving a terminal must preserve connections without rerouting unrelated geometry. Undo must restore the exact original layout.

## Decision

`DrawingRoute` has a stable `RouteId`, one existing conductor, one drawing view, ordered micrometre vertices, explicit orthogonal/free geometry mode, appearance, and optional A/B placed anchors. `TerminalAnchor` has its own `AnchorId`, view, position, and reference to a real electrical terminal. Anchors are independent of symbol geometry in this slice.

A route endpoint has an anchor exactly when its conductor endpoint has a terminal. That anchor must reference the same terminal, in the same view, at the route endpoint position. Electrical A/B order is explicit. A conductor may have one route per view; other views retain independent geometry and reuse electrical identities. Free electrical ends remain valid drafts. Multiple anchor placements of a terminal are allowed; choosing a placement is explicit.

The drawing aggregate validates geometry, views, styles, and attachment positions. Project validation additionally checks real terminals, conductors, and electrical continuity. Reverse indexes from anchors and conductors to routes are derived, maintained by mutators, rebuilt on loading, and checked by full validation. Anchor movement visits touching routes rather than scanning the drawing.

Geometry-only edits never rewire a conductor or infer a junction. Explicit route replacement, indexed vertex insertion/removal/movement, and orthogonal segment movement retain whole-path identity. Attached endpoints move through anchor commands. Segment movement inserts local doglegs where needed. An orthogonal anchor move adjusts its first straight leg, including collinear waypoints, and first bend; the rest of the path stays fixed. A straight path retains its opposite endpoint and receives an elbow if needed. Free paths move their endpoint directly. Consecutive duplicate points are removed locally; collapsed or invalid results fail atomically. Repeated up/down movement must not accumulate elbows.

`RouteCommand` stores localized inverse state. Anchor movement snapshots only the anchor and touching whole routes; restoration checks the complete affected ID set and forbids changing their electrical bindings, styles, mode, or views. Running movement in reverse is insufficient for undo. Shared-anchor movement, self-loops, batch rollback, no-ops, redo, and saved checkpoints use the existing session contract.

Electrical retermination/deletion, device deletion, and revision adoption reject changes that invalidate existing anchors or routes. Resolution is explicit: remove affected drawing representations, change the electrical model, then insert compatible representations through a compound command where appropriate. Geometry does not silently detach or disappear.

The property registry now has 29 fields. Routes participate in inherited stroke appearance, style links, view membership, and whole-vertex edits. Bindings and geometry mode are inspectable structural fields. Anchor position is inspectable but read-only in generic property edits: movement requires the dedicated command and exact layout inverse. Future bulk position tools must preserve that contract.

Route creation has its own copied appearance preset, independent of graphical polyline creation. Inherited stroke properties retain the established object/view/named-style/project precedence and explicit override intent. Preset style references participate in validation and deletion guards.

Project schema v6 persists anchors, routes, and the route creation preset with strict DTOs and canonical identity ordering. V2–v5 loading preserves existing content and creates no invented routes; explicit saves write v6. V5 DTOs stay strict and independently decodable. Reusable electrical revision bodies retain their v3 schema and continue rejecting drawing-bearing bodies until drawing instantiation has a defined contract.

The shared action registry adds `drawing.route.create`, default W in the canvas. It requests explicit input or commits a route for an existing conductor with captured view/revision checks. It does not create an electrical conductor. Native input and interactive placement remain unimplemented host responsibilities.

## Validation and limits

Route workflow tests cover edits, independent views, electrical guards, adoption conflicts, shared anchors, self-loops, malformed snapshots/schema, creation presets, action dispatch, exact undo/checkpoints, and save/reopen. An executable routes example demonstrates the complete headless flow. Separate release probes grow 100/1,000/10,000 routes while moving an anchor with two touching routes 1,000 times, verifying bounded geometry, unchanged electrical data, round trips, and complete undo. These measure fixed-degree local movement, not high-degree fan-out or UI capacity.

No global router, obstacle avoidance, snapping, junction creation, crossing/hop calculation, pinned waypoints, label attachment, arbitrary transforms, or connected-network selection is introduced. Segment indices are transient edit targets, not persistent vertex identities. Revisit this representation when persistent sub-element references, live labels, constraints, or branch geometry require them. Revisit local routing rules with actual interactive previews and representative harness fixtures before treating this as finished CAD interaction.
