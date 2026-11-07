# 0025 — Reviewed desktop origin alignment and distribution

Status: implemented in the bounded desktop prototype.

## Decision

Expose the existing origin-layout engine through Drawing ribbon, Edit menu and command search. An annotation uses its position, a graphical path its first vertex, and a terminal anchor its placed position. The complete supported selection must belong to the active view. Distribution requires at least three distinct selected origins. Routes move through selected anchors and never change electrical connectivity.

Alignment chooses X or Y and an exact signed integer micrometre coordinate, preserving the other axis without grid rounding. Distribution chooses X or Y, retains the extreme origins, sorts coordinate ties by typed identity, and rounds fractional offsets down to micrometres. This is origin spacing; visual bounds and equal edge gaps need a separate contract.

A shared retained LayoutTool serves rotation, mirroring, alignment and distribution in both the Rust bridge and frontend. Existing BeginTransform/PreviewTransform/CommitTransform transport names remain in bridge v1; project schema v6 is unchanged. The alignment coordinate uses the existing x string field for either axis; y is unused. Distribution uses neither coordinate. Exact values remain strings across the bridge.

Each preview retains the original action, selection, view and revision. Apply commits that closed proposal with one undo operation; Escape drops it. Invalid updates invalidate the previous proposal. An invalid initial arrangement remains editable so a different axis or coordinate can recover, while invalid selections still fail to begin. Applying an already-matching arrangement closes the tool without adding history or clearing redo. Revision and preview-token checks still apply to no-ops.

Ribbon, menus and palette share selection eligibility, including the three-origin minimum. A view lookup rebuilt per snapshot avoids repeatedly scanning all objects for each selected target. Controls occupy the reserved strip and remain usable at the 960-pixel minimum window width; validation feedback does not shift panes.

## Verification and limits

Four new Rust bridge regressions cover exact preview/commit/save/reopen/undo/redo, connected geometry, editable invalid seeds, wrong/stale proposals, distribution eligibility and no-op checkpoint/redo preservation. Two new Chromium workflows use the real Rust harness and saved files, exercise both axes and shared eligibility, and check minimum-width controls and fixed pane geometry.

Local macOS/aarch64 verification: cargo xtask full (282 Rust behavior tests and two doctests), cargo xtask ui (23 browser workflows), cargo xtask desktop, embedded-asset debug native build, and native webview-to-document-engine launch. Browser screenshots are written under ignored target/ui/. Native interaction automation and Windows/Linux execution remain pending; these checks do not establish large-document rendering capacity.
