# 0023: Retained graphical path placement

Status: implemented through P/menu/toolbar/palette with local Rust/browser checks. Native platform interaction, post-creation vertex tools and electrical route creation remain pending.

## Decision

Integrate the existing P graphical-polyline action as one multi-step placement tool. Click accepts a vertex; numeric X/Y plus Add or Enter in a coordinate field accepts an exact point. Backspace/Back removes the last draft vertex. Enter on the canvas or Finish creates one whole path from the accepted vertices. Escape/Cancel drops the entire draft. At least two distinct consecutive points are required. The saved object retains one UUID, all ordered vertices and normal Properties/move/rotation/mirror behaviour.

Keep accepted geometry and the tentative next leg distinct. Accepted segments use a continuous preview with vertex markers; the tentative leg is dashed and the cursor marker distinguishes snapped from unsnapped input. Finish never accepts an unclicked tentative leg. Draft vertex steps are transient and do not create document undo records; finishing uses the existing normal P dispatcher and creates one undo step.

Rust owns the draft identity, accepted vertex list, validated cursor and retained complete accepted-path proposal. Updates invalidate older tokens before strict coordinate/snap parsing and validation. Accept/back steps consume their current token and issue a new one, so replay cannot add/remove multiple vertices. Commit accepts only the current proposal token and shared host input ID, retaining the original revision/view/action context. A changed document, cancelled draft or replacement tool cannot commit an older proposal.

Validate candidate geometry with ordinary project insertion and the real graphical stroke creation preset. Retain the accepted path separately from candidate display geometry. Normal P commit uses its stored UUID/vertices with that unchanged revision's stroke preset; commit performs no snap query. Tests verify matching scene/saved appearance intent and unchanged electrical records. Graphical paths do not create conductor identities or electrical attachments when snapped onto terminals.

Share view-local cursor resolution between annotation and path creation: terminal/endpoint/midpoint/5 mm grid priorities, ten-pixel tolerance, explicit cycling, temporary Alt suppression and numeric bypass. The existing move service keeps its different selected/touching-route exclusion rules. All document geometry remains integer micrometres; exact vertices cross the bridge as strings and display-only geometry uses millimetre floats.

The frontend coalesces pointer-only previews while retaining every click as an ordered intent with its captured position and snap choice. Under delayed replies, later pointer motion cannot replace an earlier clicked coordinate. Finish waits for prior intents and pending validation. Cancellation invalidates the generation immediately, so late replies cannot append or commit. Native text/button input keeps control ownership; fixed placement keys remain distinct from the remappable P action. Save/Undo/Redo and other editing controls are held during placement.

Use the reserved tool strip for coordinates, point count, Add/Back/Finish/Cancel. At narrow widths the auxiliary count hides while the footer still reports it, keeping coordinate inputs and completion/cancellation controls usable at 960 px. Dashed legs/vertex markers are view feedback and never enter baseline hit-test records or project persistence.

## Evidence and revisit conditions

The full suite has 278 Rust behaviour tests and two doctests. New bridge regressions cover stroke preset/style intent, accepted versus tentative vertices, one-step history, saved/reopened identity, no electrical mutations, back/duplicate/replay handling, invalid/stale/cancelled/replaced proposals and large exact coordinates. Nineteen Chromium workflows include numeric point entry, draft revision, minimum-width controls, whole-path move/rotate, real saves/history, rapid clicked coordinates under delayed responses, snapping/cycling/suppression and in-flight cancellation preserving redo.

Project schema v6 and keymap schema v1 are unchanged. Post-creation vertex/segment editing, orthogonal/45-degree path aids, explicit closure, repeated stamping, wire/symbol placement and custom placement-key profiles remain future work. Candidate cloning and linear snapping need representative large-view latency/memory/queue-budget measurements before production capacity claims. Native macOS build/launch is checked separately; native Linux/Windows and physical input validation remain required.
