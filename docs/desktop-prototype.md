# First desktop editor slice

Status: a working Tauri 2 desktop prototype backed by the real Rust document engine. Native macOS/aarch64 build and launch are verified. Chromium workflows test the same bridge and actual temporary project files. Linux/Windows execution, comprehensive native interaction automation, packaging, and large-document rendering validation remain pending.

## Start the app

Install the repository's Rust toolchain/components, Node 22.12+, and the appropriate [Tauri system prerequisites](https://v2.tauri.app/start/prerequisites/). Node 25.9.0 was used for the current macOS validation. The frontend and native host each have committed lockfiles.

```sh
cd apps/desktop
npm ci
cargo fetch --locked --manifest-path src-tauri/Cargo.toml
npm run desktop
```

These first dependency-cache setup steps can use the network. Once the required packages are cached, `npm ci --offline` and the offline Cargo checks can run locally. `npm run desktop` starts Vite on loopback port 1420 and the Tauri window. Rust source changes restart the native host; frontend changes reload the webview. Do not rely on a development restart to retain unsaved in-memory work.

To build an embedded-asset debug binary without an installer:

```sh
npm run desktop:build -- --debug
```

It is written under `src-tauri/target/debug/` and runs without a Vite server. This is a native binary build, not an installed/notarized release package. The Tauri host is a separate Cargo project so normal core checks do not require webview libraries.

## Working behavior

- The initial conveyor demonstration is real domain/drawing data, not a static frontend mock. Its pristine state can be replaced or closed; after editing, save protection applies. Save first chooses a new project destination. Existing unobserved files are protected by the document controller.
- Native open/save choosers load validated projects and preserve the original document on failures. Opening another project, closing, or quitting is blocked while changes need saving. Save-as currently refuses unobserved existing destinations, even if a platform chooser offers overwrite. A complete save/discard/cancel workflow remains future work.
- The workspace has fixed explorer/inspector widths, a fixed toolbar/status area, and internal pane scrolling. Errors and long values do not insert layout rows or widen the canvas container. The minimum native window is 960 × 640. Docking, splitter resizing, and saved workspace layouts are not implemented.
- SVG renders committed paths, terminal placements, and literal annotations. Drag the background to pan; scroll to zoom; F fits the active view. Select in the drawing or explorer. Shift selects a mixed group. Right-click exposes the same complete inspector, retaining an existing group.
- The inspector uses all applicable core property metadata, including read-only structural fields, mixed values, origins, and applicable/unsupported counts. Text, positive lengths, colours, and eligible coordinate fields have editors. Enter applies one atomic property edit; Escape reverts the local input. Complex structural/reference/vertex editors remain pending. Lengths and coordinates are edited in exact integer micrometres, distinct from physical wire attributes.
- Undo/redo, save, Properties, fit, command search, and connection reports use the shared registry/router/dispatcher. Open is a host request through that registry. Primary+O/S/Z, Primary+Shift+Z/P, E, and F are available where applicable. Native text fields and IME composition own their input. Mac uses Command; Windows/Linux profiles use Control. Tools not shipped by this UI are disabled in command search.
- The connection pane derives actual wire endpoints and cable membership from the electrical report. It does not create devices or infer connections from crossing graphics. If the document changes while the pane is open, stale rows are replaced by a refresh prompt without resizing the pane.

An untouched loaded project is clean. Actual edits change its core history/checkpoint, and undo/redo can return to a saved state. A report or selection change does not dirty the document. Exact IDs, revisions, and editor values cross the bridge as strings; only scene display coordinates use floating-point millimetres.

## Local verification

Install the pinned browser test runtime once:

```sh
cd apps/desktop
npm exec -- playwright install chromium
```

Then from the repository root:

```sh
cargo xtask full
cargo xtask ui
cargo xtask desktop
```

`full` includes the headless bridge contracts and remains independent of Node/Tauri. `ui` checks bridge tests, Prettier, TypeScript/build, and four browser workflows. It starts an isolated Rust harness on port 3838 and Vite on 1421, operating on a temporary approved project. It does not mock the electrical model and does not contact GitHub. Screenshots are written to ignored `target/ui/`. The harness is an example/test program, not a production HTTP service; it binds loopback, accepts bounded JSON requests only from the known local frontend origins, and exposes no arbitrary file-open endpoint. Native desktop uses IPC instead.

`desktop` checks frontend formatting/build and native Rust formatting/Clippy with the separate locked host manifest. It needs the platform's webview development dependencies. A native launch smoke test separately confirmed that the macOS webview invokes the Rust controller; browser tests do not prove native dialog, accessibility, font, or keyboard behavior.

The browser workflows cover actual property edit/save/undo/redo, mixed applicability, protected typing, right-click selection, pan/zoom/fit, long and markup-like values, error/focus retention, window resizing, report values, and external-save conflicts. The Rust bridge tests cover stale/foreign contexts, malformed input, exact large numeric values and IDs, full union inspection, guarded lifecycle failures, and shared input replay/resynchronization.

## Boundaries and next steps

This is the first interaction slice. Wire/symbol placement, geometry drag/rotate previews, snap feedback, sub-element selection, font measurement/export parity, library/default editors, full native menu integration, pane layout preferences, and coherent cable overview/detailed sheet navigation remain planned. SVG terminal captions are derived view feedback; they are not persisted live connection-label templates. The SVG scene is currently a complete projection and has no virtualization or large-view capacity claim. A different renderer/framework can replace it behind the bridge as measured workloads justify.

Project schema v6 is unchanged. User selection, viewport, pane state, bridge input IDs, and transient messages are not written to project files. See [decision 0017](decisions/0017-first-desktop-slice.md).
