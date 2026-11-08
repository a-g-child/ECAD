# 0035: Default project-page navigation with an alternate electrical explorer

Status: implemented for the current single-project desktop. Multiple open-project roots and scoped tabs require the separate host/workspace integration; this slice does not simulate them in the UI.

Pages is the default explorer mode. Show one real open-project root, logical drawing documents and their ordered sheet memberships. The bridge supplies an is_default hint from exact initial document metadata and identity. Elide a package only when it is the sole package and this hint is true. A sole customized package stays visible, as do every package and empty package in a multi-document project. Membership order, rather than physical sheet-record sorting, determines navigation order. Literal long/multiline titles remain intact, with compact labels and full tooltips.

Selecting a sheet uses the same SheetWorkspace.show guard as the sheet selector, including stable sheet identity, camera restoration and selection clearing on a real sheet switch. Highlight the active page independently of electrical object selection. Right-click a page activates that exact sheet and opens its ordinary sheet editor. Package/root summaries expand/collapse; they never target unrelated electrical properties. Document creation/rename/transfer controls remain future work, although loaded groups and memberships are browsable now.

Electrical mode retains the existing functional/component hierarchy and drawing-representation section, using real source object identities and the complete shared inspector. It is scoped to the current project. Switching explorer modes does not switch sheets, alter drawing selection, dirty the project, modify history or cancel a tool. Mode changes are blocked during unfinished tools, matching the existing action-surface guard. Page activation is also blocked during actions and pointer gestures; delayed clicks cannot retarget a draft.

## Action, focus and workspace contracts

Register workspace.explorer.pages and workspace.explorer.electrical as explicit unbound document actions. Mode buttons, View menu and command search use the shared registry/router/dispatcher and monotonic input stream. Requests preserve captured revision/selection/view/sheet and are rejected when stale or replayed. They change transient frontend presentation only. Workspace checked state is reflected by the mode buttons; it is not persisted project data.

Keep independent expansion maps for Pages and Electrical. Project-root state is scoped to the session epoch, with document/sheet/object keys retaining stable identities. Clear expansion state on document replacement; changing mode retains both maps. Rebuilds preserve the focused element kind and key, distinguishing branch summaries from their nested electrical selection buttons. Native details/summary and buttons support Tab, Enter and Space; this is a navigation landmark, not an ARIA tree promising an unimplemented arrow-key contract. Custom tree traversal, search/filter, drag reorganization and persisted preferences remain future work.

Cancelling a transient drawing tool now returns focus to the canvas after the cancellation response. This prevents a hidden text control's focus context from suppressing the next project Undo. Ordinary active text/IME editing retains its existing protected-input behavior.

Explorer content uses the shell's fixed pane bounds and internal scrolling. Large labels and mode notifications do not resize the drawing canvas. Page navigation has separate stable sheet keys; it never invents an electrical ObjectRef for a page or duplicates the physical connectivity model.

## Evidence and remaining work

Core/bridge checks verify unbound action registration, no-document availability, nonmutating mode requests, replay/stale rejection and the exact default-group hint. Browser tests exercise default Pages, Electrical/menu/palette access, expansion and selection retention, keyboard page activation, page context menus, draft guards, cancel/undo, real grouped-file loading, empty groups, sole custom groups, membership order and stable bounds at 1360 × 880 and 960 × 640. Existing object-editing suites explicitly choose Electrical; explorer tests exercise the default mode. Fixtures are opened by restarting the real Rust browser host, without adding a test mutation endpoint.

Local macOS/aarch64 verification: cargo xtask full (332 behavior tests and two doctests), cargo xtask ui (37 browser workflows), cargo xtask desktop and the embedded-asset debug native build/launch with its webview connected to Rust. Browser checks do not establish native interaction/accessibility, Windows/Linux execution or large-project rendering performance.

Next boundaries: native multi-project ownership and captured-project bridge envelopes, multiple roots, document management, new source-view creation, project-scoped tabs/retained selections, hierarchical sheet navigation and contextual project services. Keep copied project lineages distinct by runtime workspace handles when those features are integrated.
