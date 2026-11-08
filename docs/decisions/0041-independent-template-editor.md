# 0041: Independent template editor and publication history

Status: accepted; implemented for the first bounded editor slice.

## Boundary

A template draft is an application `TemplateDocument`, independent of electrical `Project`, source views and sheets. It owns a fresh revision epoch, validated complete variant replacements, bounded exact undo/redo and a saved checkpoint. Open loads a complete immutable family; Save publishes a complete fresh revision. Publication updates the checkpoint only after durable success and is never undone by either history. Failures retain draft, history and prior published pin. No persistence schema change is needed.

The desktop hosts one optional editor above its mounted project panes. Context carries a separate editor token and no electrical selection/view/sheet. Shared registry/router/dispatcher handles template and existing Save/Undo/Redo/Properties/Fit actions on the same monotonic input stream. Stale project/editor callbacks and project-only requests cannot retarget the other workspace. Text/IME/repeat protection remains shared. Native close/quit guards include draft and pending property fields; project Open/Save As requires closing the editor first.

Typed bridge projections expose exact micrometre values and identities as strings. UI fields are a transient closed proposal; Apply validates the complete layout before recording history. Pending fields block unrelated operations until Apply/Revert/Discard. Applied state survives frontend reload in the Rust host; unapplied text has no crash-recovery guarantee. New/Open/Close protect dirty drafts; Discard is explicit. The headless controller can run without an electrical project, while the current desktop shell still requires its usual project.

## Library and project effects

The adapter stores `drawing-templates.json` under native application configuration. Configure/read does not create directories; explicit publication may create the parent. Existing observed-version saves and cooperating-writer locks remain authoritative. Explicit Reload refreshes the publisher after conflict and retains the editor. The catalogue combines observed library revisions and bundled project revisions, rejecting conflicting immutable bodies. A saved body bundled only in the project can be opened and explicitly published locally.

Add saved revision to project is a separate ordinary project command. Current sheet/default pins do not change; importing does not adopt. Project undo removes an unused imported body without retracting the library revision. The editor cannot silently publish uncommitted field values or automatically update other projects.

## First UI slice and revisit conditions

Libraries entry, contextual fixed-height ribbon, independent placeholder preview/camera, saved-variant browsing, New/Open/Save/Close/Discard, exact custom paper/margins/content bounds, cell caption and existing literal/variable runs are available. The inspector is shared by selection/right-click. Table/cell geometry is read-only. Full table insertion/split/merge/resize, run insertion, style properties, standard-profile presets, anchoring, reusable title blocks, multiple template tabs and reviewed adoption remain planned in the template editor plan.

Revisit the one-editor guard when implementing workspace tabs; retain captured identities rather than routing by active tab alone. Add layout commands over complete validated partitions when table authoring arrives. Verify normative profiles before claiming ISO/ANSI compliance. Extend dependency closure only when reusable blocks/assets exist.

## Verification

Headless real-file tests cover exact history/checkpoints, independent operation without a project, stale contexts, failed publication and recovery. Bridge tests cover action availability, project/editor isolation, explicit import/undo, dirty lifecycle guards and pending/invalid proposals. Chromium workflows exercise real Rust state, ribbon bounds, multiline properties, publication/history, import and discard. Native interaction and Linux/Windows execution remain separate validation work.
