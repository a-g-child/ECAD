# 0050 — Template multi-point paths

Status: implemented for bounded template artwork.

## Decision

Register `template.draw.polyline` alongside the existing line tool, exposed as **Path** in the Template ribbon and command palette. It is initially unbound, like other template artwork actions. Each click appends an ordered paper-space vertex. Enter finishes clicked points only; Backspace removes the last point; Escape cancels. Closing a path is explicit: click its starting point again. Nonconsecutive repeated points are retained, but a consecutive duplicate click is ignored with feedback. Closed paths are outlines, not filled regions.

Share the path-key policy with the schematic path tool, including composition protection and suppression of native key repeats for finish/back/cancel. Preserve the schematic tool's ordered snap-aware Add/Back/Finish intents. Template paths use the existing coalesced preview queue differently: each proposal contains the entire current clicked vertex list, so replacing an older unsent preview cannot drop click or Backspace intents. Hover draws only a rubber segment and never enters committed geometry.

The Rust constructor validates signed coordinates, exact paper bounds, consecutive duplicates and a 2–4096 vertex limit. It normalizes bounds while retaining every ordered local vertex. The bridge accepts full vertex lists only for the registered path tool, rejects mixed text/two-point/path payloads, and retains closed revision/target-bound proposals. Fewer than two local clicks keep an armed gesture with no committable object. Invalid full paths remain editable; Backspace can remove the offending final point. A failed preview cannot commit an older token.

A successful finish creates one artwork object and one undo entry. Existing numeric vertex editing and whole-object movement work on that complete path. No electrical connectivity is created or changed. Retained polyline artwork already exists, so no project/template persistence version changes are needed; publication and project adoption remain explicit.

## Boundaries and revisit conditions

Template paths currently use exact clicked paper coordinates without schematic snapping, orthogonal constraints or numeric coordinate entry. Shared snapping/point collection, pointer vertex handles, circle/arc tools and appearance controls remain future slices. Keep the full-list preview strategy bounded; revisit it with measurements if larger vertex counts or expensive preview validation make interaction slow. Do not replace schematic ordered intents with a latest-only queue.

## Verification

Domain regressions cover closing points, vertex order, zero-width/height axes, signed/overflow/out-of-paper geometry, duplicate points and the 4096 limit. Bridge regressions cover registered dispatch, complete transient proposals, malformed payloads, invalidated tokens, independent history and cancellation preserving redo. Browser workflows cover rapid clicks during delayed previews, Backspace/repeat handling, duplicate clicks, hover exclusion, invalid-point correction, whole-object movement, exact undo/redo identity and library reopen. Existing schematic path tests exercise the shared key policy.
