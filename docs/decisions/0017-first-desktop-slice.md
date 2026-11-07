# 0017: First Tauri editor and a headless desktop bridge

Status: implemented bounded prototype; macOS native build/launch and Chromium integration verified. Linux/Windows and comprehensive native interaction remain pending.

## Problem

The working engine needs a real interaction surface to test document ownership, stable workspace geometry, property applicability, keyboard protection, and useful drawing projections. Desktop libraries must not become prerequisites for headless development, and browser tests must exercise real commands rather than a parallel JavaScript electrical model.

## Decision

Use Tauri 2 with a small TypeScript/DOM/SVG frontend. Keep the native host in a separate Cargo project/lockfile, excluded from the main workspace. Add a headless `ecad-bridge` workspace crate depending on the application/domain and concrete file adapter, with no Tauri dependency. The host owns one mutex-protected controller, exposes strict IPC requests, and handles native file choosers and close/quit protection. Core domain/application dependencies remain unchanged.

Bridge-v1 transport DTOs are distinct from internal objects and project JSON. Whole tree/scene snapshots carry session epoch/sequence, stable typed identities, and action metadata. IDs, revision/input counters, and exact inspector values are strings to avoid JavaScript integer truncation. Scene coordinates are explicitly display-only floating-point millimetres; committed edits parse checked integer values back in Rust. Stale/foreign context tokens and malformed targets are rejected. Snapshot resynchronization also advances the host input counter after a frontend reload.

The bridge uses existing property inspection/planning, commands, document checkpoints, guarded stores, registry/keymap/router/dispatcher, and electrical reports. It does not duplicate topology or style ownership in JavaScript. Supported native/web inputs share one monotonic stream; text/IME/control focus remains protected. Open and fit are registered typed host requests. Mac's minimal native application menu avoids a second document-undo accelerator path. Full native menu/custom keymap integration remains future work.

A pristine built-in demonstration is disposable; editing it activates unsaved protection and first save requires an explicit destination. Opening another file, close, and quit protect unsaved edits. Native chooser results are rejected if the document revision changes while the chooser is open. Existing unobserved save-as destinations remain protected. A full discard/overwrite workflow has not been introduced.

The shell owns fixed pane and toolbar/status geometry, overflow, and shared controls. Imported text is assigned as text, never HTML. SVG renders actual retained paths, terminal placements, and annotations. Pan, zoom, fit, view choice, selection, and viewport feedback remain transient. Whole-object mixed selection exposes the complete core property union; bounded editors support text, lengths, colours, and eligible positions. Read-only structural fields stay visible. Complex geometry/library/default editors and interactive wire tools are not simulated.

A loopback-only browser harness calls the same bridge and actual temporary files. It is a development example, not a production server or a second model. Chromium tests operate through the frontend and real document lifecycle; native IPC and chooser wiring are compiled/launched separately. The harness validates known local origins, bounded JSON bodies, and request DTOs, with no arbitrary frontend file-open endpoint. Production desktop embeds built assets and uses Tauri IPC.

Keep Node formatting/type/build/browser checks and native-host formatting/Clippy in separate `ui`/`desktop` profiles. Core `full` remains independent of Node, browser binaries, and webview system libraries. Commit npm/native lockfiles and source-derived icons; ignore generated build/evidence assets. Project schema v6 remains unchanged.

## Evidence and revisit conditions

Bridge tests verify full inspection, actual mixed edits/history/save conflicts, strict context/request validation, exact i64/u128 values, and input replay/resynchronization. Four browser workflows verify edits/saves/undo, shared fields, protected typing, context selection, pan/zoom/fit, reports, long/markup-like values, focus retention, fixed geometry, and external changes. Native macOS builds, embedded-asset binary build, and webview-to-core launch are verified. Native dialog/keyboard/accessibility automation, packaging, other operating systems, and large-document rendering remain explicit gaps.

Revisit the frontend/renderer after measuring actual visible density, fonts, hit testing, and pointer previews. The first renderer uses complete projections, not incremental scene updates or virtualization. Add typed incremental/cancellation contracts, comprehensive generated transport schemas, native interaction tests, and platform validation before treating this prototype as production architecture. Preserve the bridge/core boundary if replacing DOM/SVG or adopting a frontend framework.

The host follows Tauri's documented [command boundary](https://v2.tauri.app/develop/calling-rust/), [Vite integration](https://v2.tauri.app/start/frontend/vite/), and [native dialog APIs](https://v2.tauri.app/plugin/dialog/). The fixed-pane layout, transport choices, test harness, and prototype scope are project decisions.
