# 0019: CAD navigation, directional selection, and drag movement

Status: implemented with local Chromium workflows against the real Rust bridge. Native trackpad/mouse verification across supported operating systems remains pending.

## Decision

Reserve the primary button on empty canvas for directional marquee selection. Left-to-right encloses complete objects; right-to-left selects objects touched by the rectangle. Horizontal direction determines the mode regardless of vertical direction. Show a solid blue enclosure and a dashed green crossing rectangle while dragging. Shift adds marquee results to the selection; Shift-click toggles one object. A plain empty click clears selection. Escape or loss of pointer capture/focus cancels an unfinished gesture and retains the selection.

Unmodified vertical wheel/two-finger scrolling zooms about the cursor. Shift-scroll pans horizontally; Ctrl-scroll pans vertically on every platform, including macOS. Shift wins when both are held. Normalize pixel/line/page wheel units, retain sub-step trackpad deltas, ignore zero zoom input, and clamp the viewport scale. Accept Shift scroll events delivered on either delta axis. Middle-button drag remains an additional pan gesture. Navigation and selection never change document history.

Select whole objects by default. Enclosure uses painted bounds including stroke width and square endpoint caps. Crossing checks real path segments/caps and terminal circles rather than accepting empty space inside route bounds. Annotation selection uses the renderer's current font bounds; derived terminal captions are feedback rather than independent selectable text. This is view hit testing, distinct from the headless numeric origin-layout contract. Rich symbols and exact exported font metrics remain future work.

A primary drag starting on a movable object moves it; starting on a selected object preserves the selected group. Use a four-CSS-pixel threshold to distinguish clicks from drags. Preserve the grabbed point's offset relative to the explicitly selected reference origin. Obtain that origin from Rust's exact string-valued move response, then apply checked micrometre pointer offsets; never derive edit coordinates from approximate scene origins.

Drag start invokes the same registered M action. The existing host-retained proposal and dispatcher commit remain authoritative. Coalesce pointer previews and wait for the final acknowledged proposal on release, including when bridge responses are delayed. Escape, pointer cancellation, and focus loss cancel a held drag; obsolete asynchronous replies cannot finish it later. Failed geometry proposals do not commit older positions. Routes still move through terminal anchors; dragging a route itself does not infer which electrical placements to move. M remains available for click-to-place and numeric placement.

Put gesture handling and view-selection geometry in dedicated frontend modules. Keep document mutations in Rust and share the move lifecycle with keyboard/menu/toolbar/palette entry points. Do not add an independent JavaScript geometry-edit history. The project and keymap schemas are unchanged; pointer-profile customization is a future workspace preference.

## Evidence and revisit conditions

Nine Chromium workflows include cursor-anchor invariance, modifier axes, line/page/horizontal wheel delivery, zero delta, middle-button pan, enclosure/crossing/additive selection, empty route bounds, renderer text bounds, stroke/cap edges, marquee Escape/focus loss, preserved group/grab offset, delayed final preview release, one-step undo/redo, and asynchronous drag cancellation. Core Rust suites remain local and unchanged.

Browser-delivered wheel and pointer events do not establish behavior for every physical mouse, touchpad, native pinch gesture, accessibility system, or OS display scale. Verify these on native Linux, Windows, and macOS before release. Selection currently scans visible SVG objects and path vertices; measure large-view input latency before adding spatial indexing. Overlap cycling, selection filters, segment/vertex tools, drag auto-pan, touch-specific gestures, and editable pointer profiles remain separate scope.
