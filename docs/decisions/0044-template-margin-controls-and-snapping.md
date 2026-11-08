# 0044: Template margin controls and border snapping

Status: accepted and implemented for the bounded template editor.

## Pointer snapping

Table move and outer-edge resize gestures now query the retained template's own margin border. The headless `table_border_placement` calculation uses integer micrometres, signed proposed coordinates and an explicit optional document-space tolerance. Movement chooses the nearest corresponding horizontal and vertical edges independently, with left/top tie priority. Resizing considers only the captured dragged edge and retains the opposite edge. Invalid/collapsed/out-of-interior results reject before a partition can be committed.

The UI converts an eight-screen-pixel tolerance into integer paper units using the actual SVG transform at gesture start. Zoom is locked during the drag. Alt passes no snap tolerance; numeric dialogs continue using exact unsnapped geometry. Click-only handle interactions preserve the original placement. This is geometric placement, not a persistent border attachment, standards profile or electrical connection.

The strict pointer proposal DTO retains its move/edge target in the existing gesture signature. Validated preview feedback identifies chosen margin edges and drives green border guides; the UI does not infer a successful snap from rendered geometry. Commit uses the exact retained command body and never re-queries targets. Stale/invalid/cancelled proposals preserve history and existing token invalidation behavior.

## General margin field

General margin (µm) edits all four existing margin values in one local property proposal. Equal sides display the shared value; differing sides display Mixed. Individual side values remain editable. There is no fifth persisted margin authority and no schema migration.

Editing the general field checks the visible Fit layout inside margins option. The user can uncheck it for strict containment validation. Fitting minimally translates out-of-interior tables without resizing cells or changing their IDs, captions or bindings; it intersects the explicit content region with the new interior. Tables too large to fit, invalid paper and overflowing extents fail coherently. Decreasing a margin does not move an already-contained table outward. This option is a one-time explicit edit, not an inferred reusable anchor.

All paper/content/table changes enter one validated template variant replacement with exact undo/redo and publication checkpoints. Failure retains local fields and the previous draft. Project drawing geometry, metadata, sheet pins and published library bodies are unaffected until their existing explicit workflows run.

## Verification and revisit conditions

Domain tests cover all corresponding borders/corners, inclusive tolerance, deterministic ties, signed/extreme coordinates, bypass and fitting without scaling. Bridge tests cover exact numeric alternatives, snap feedback, captured-edge protection, closed commits, failed fitting, no-op redo and saved checkpoints. Chromium workflows cover the general/mixed fields, opt-in fitting, invalid field retention, pointer guides, Alt, numeric alternatives and click-only intent.

Verified standards presets, persistent anchors, application-wide margin defaults and additional snap-target kinds remain separate work. Any future default must copy or explicitly inherit the same exact four-side contract rather than introducing a second authority.
