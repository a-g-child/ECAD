# Electrical connection reports and CSV outputs

Status: headless electrical projection, shared report action, and three deterministic CSV encoders implemented. Native report controls/file dialogs, printable pages, placed live labels, physical wire attributes, and legacy pinout import remain planned.

`ConnectionReport::from_session` captures actual electrical values at one session epoch/sequence. Devices and terminals are indexed by stable identity. Each terminal retains every attached conductor end; it never guesses a primary wire. A self-loop contributes A and B separately. Unconnected terminals, completely free conductors, partially free conductors, and empty cables remain meaningful drafts.

The report includes placed occurrences only. Bundled reusable definitions do not add physical devices until instantiated. More drawing representations do not add terminals, wires, cable cores, or billable items. A cable's ordered overview endpoints remain distinct from the real device terminals assigned to its cores. A/B order does not imply signal direction.

Reports are immutable observations. Renames, terminal-number changes, hierarchy changes, assignments, and undo become visible in a newly queried report. `is_current(session)` checks original lineage and session revision. Drawing-only edits, undo to equivalent contents, and reopening conservatively invalidate old reports even when exported electrical bytes remain equal. Saving without an edit does not invalidate a report. These guarantees concern the live session, not an unobserved external file change.

`encode_current_connection_csv` rejects stale/different-session snapshots. `encode_connection_csv` deliberately accepts an identified historical snapshot. Both produce strings without file IO, project mutations, checkpoint changes, or undo entries. The remappable `reports.connections` action returns the current projection, with normal captured-context and protected-input handling. It does not save the document or write an output file.

## Output contracts

| Output | Rows | Important fields |
| --- | --- | --- |
| Wire list | One per actual conductor, in conductor-ID order | Project/conductor IDs, completeness state, cable identity/designation, actual A/B terminal/device IDs and labels, functional paths |
| Terminal table | One per attached conductor end, or one for an unconnected terminal; terminal ID, conductor ID, then A/B order | Device/terminal identity and current labels, connected/unconnected/free-destination state, wire/end/cable references, actual destination and paths |
| Cable schedule | One per core, or one for an empty cable; cable ID then core ID | Cable identity/designation, core state/ID, separate ordered overview kind/ID/name/path and actual A/B terminal assignments |

Wire/core states are `both_attached`, `a_free`, `b_free`, and `both_free`. Terminal states are `connected`, `destination_free`, and `unconnected`. Empty assemblies have `empty_cable` core state. A free endpoint has blank terminal/device fields; root scope for an existing endpoint has blank scope IDs and `[]` scope names. These are deliberately distinct cases. Empty projects produce headers only.

Every row contains the persistent project ID. IDs use canonical lowercase hexadecimal. Labels are current display values and do not control row identity or ordering. There is no separate user wire number in the current model, so the output uses conductor IDs rather than inventing one. Wire colours, sizes, materials, manufacturers, shields/pairs, and installation lengths are not inferred from graphical width or geometry.

CSV is UTF-8 with fixed headers and LF record endings. Every cell is quoted; embedded quotes double, and field newlines remain literal data. The adapter preserves values rather than applying spreadsheet-specific interpretation. Functional scope IDs are slash-separated root-to-leaf IDs. Scope names are a compact JSON string array in a quoted CSV cell, preserving Unicode, slashes, commas, quotes, and newlines without ambiguous path splitting. Scope strings are cached per scope during each encoding call.

Session revision/epoch, timestamps, file paths, drawing geometry, and caches are absent from the CSV. Equal electrical content therefore produces identical output across saves, reopened sessions, and geometry-only edits. CSV is a derived output, not an import format or authoritative project representation; project schema v6 and keymap schema v1 remain unchanged.

## Verification and current limits

Run `cargo run --offline --locked -p ecad-adapters --example reports` for a wire-list demonstration with freshness, rename/refresh, undo, and reopen checks. Independently authored conveyor fixtures cover [wires](../crates/ecad-adapters/tests/fixtures/conveyor-wires.csv), [terminals](../crates/ecad-adapters/tests/fixtures/conveyor-terminals.csv), and [cables](../crates/ecad-adapters/tests/fixtures/conveyor-cables.csv).

Local workflow tests cover full connection multiplicity, intentional drafts, hierarchy/labels, independent repeated instances, no mutation, stale/historical behavior, escaping, golden output, and persistence. Scale cases measure full report construction and all three in-memory CSV strings over 100/1,000/10,000 devices paired by individual one-core cables, with fixed two-level hierarchy. Timings are single samples; disk IO, memory, native UI, high-degree terminal fan-out, and large hierarchy-depth combinations are not measured by this recipe.

Future placed pin tables should consume this identity-based value source and make stale observations visible, rather than generating disconnected text or duplicating electrical connectivity. Template/layout authoring, automatic refresh/cancellation, export destination/durability behavior, user wire identifiers and physical fields, and validation by building a system from its documentation require later slices.
