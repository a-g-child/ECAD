# 0018: Host-retained move proposals

Status: implemented in the desktop bridge and UI; browser interaction tests exercise the real Rust controller. Native interaction automation and measured large-view latency remain pending.

## Decision

Integrate the existing M action with a transient move tool. A selected annotation, graphical path's first point, or terminal anchor supplies the explicit reference origin. The user can change that reference. Routes move through selected anchors, preserving the core's whole-route geometry and electrical bindings.

Rust retains the original action context and a closed `MovePreview`. Each preview has an opaque lifetime/proposal token. Updates invalidate the preceding proposal before parsing or geometry validation. Commit accepts the token and a shared input ID, consumes the held proposal, and invokes the normal M dispatcher; it never accepts client geometry or recalculates snapping. The original revision/view/selection guards and exact document undo remain authoritative. Cancel drops the proposal without changing history. A replaced/opened document cannot accept an old token.

The bridge sends only affected preview geometry, using appearance rows captured at tool start. SVG merges these rows into its current display. It never calculates route movement. The UI coalesces pointer requests to one in flight and one latest pending point, ignores obsolete replies by tool generation, and disables Apply until the current proposal is acknowledged. Failed updates clear the ghost and cannot commit the preceding valid position. Selection, view changes, and other editing controls are held during placement; commit controls are locked while its response is pending.

Numeric coordinates remain signed integer micrometre strings throughout transport. Pointer coordinates are rounded to integer micrometres only within JavaScript's safe integer range; positions outside that range require numeric input. Pointer queries use terminal/endpoint/midpoint/grid snapping, a ten-CSS-pixel tolerance converted through the current viewport, and a 5 mm grid matching the display. Tab cycles ranked candidates, Alt temporarily suppresses snapping, and the chosen destination is marked. Numeric placement bypasses snapping. Coincidence does not create connectivity.

A fixed-height placement strip is reserved from startup so showing tools/errors does not shift the canvas. Enter/Apply or a canvas click commits an acknowledged preview; Escape/Cancel uses the registered cancellation action. Text/IME protection remains in the central keyboard path. Shortcut sequence timing uses the host's monotonic clock, because frontend timing restarts across reload while the router persists.

## Evidence and revisit conditions

Rust bridge tests cover exact preview/commit agreement for shared routes, one-step undo/redo, cancellation, invalid updates, supersession, stale documents/lifetimes, replay, save/reopen, values above JavaScript's safe integer range, and reset frontend clocks. Chromium workflows cover exact group placement, invalid numeric input, fixed bounds, clean cancellation, saved geometry/history, terminal feedback, cycling, and temporary suppression.

Project schema v6 is unchanged. This is one interaction tool, not wire/symbol creation or vertex editing. Scene rendering still rebuilds the visible SVG, snap queries scan candidates linearly, and no hysteresis is provided. Measure visible density and pointer latency before adding spatial indexing, incremental rendering, or asynchronous worker budgets. Native keyboard/accessibility automation and Windows/Linux execution remain necessary.
