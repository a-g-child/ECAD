# 0009: Typed electrical properties and heterogeneous bulk editing

Status: accepted for the existing electrical model; creation defaults and styles follow separately.

## Decision

Address subsystems, devices, terminals, ports, conductors, and cables through `ObjectRef`, retaining typed IDs. Register 15 stable `PropertyId` values covering all current electrical object fields. Metadata provides stable machine keys, initial labels, value kinds, permitted reference kinds, semantic ownership, applicability, and principled editability. Contextual access additionally reports identity, dedicated structural-operation, and pinned-instance restrictions.

Read directly from real model fields. Text, object references, optional references, and canonical membership lists remain distinct values. Device/cable designations and subsystem/interface names share a property only where their meaning and type agree. Terminal numbers never become identities. Numeric units/ranges, physical wire attributes, graphical roles, custom schemas, and library assignments will extend the registry alongside real model fields; do not invent unused properties or put unvalidated string bags on every object.

Expose identities, terminal ownership/membership, interface mappings, and cable endpoint/core relationships even where editing requires dedicated structural operations. Support validated designation/name/terminal-number edits, existing containment changes, and conductor termination edits. Reconstruct label-bearing values through their existing constructors; preserve IDs and indexes. Terminal-number and interface-name uniqueness remain enforced. Reuse existing topology mutation methods for connections/containment. Respect indexed pin ownership for all editable fields; the placed root's name/parent remain occurrence parameters.

## Selection and editing

`inspect_properties` deduplicates targets and returns the union of applicable fields. Each row exposes exact applicable, editable, restricted, and unsupported subsets. A row is common when every selected object supports it. Uniform typed absence is different from mixed values and blank text. Missing targets are errors, never silently omitted objects.

`property_edit_plan` preflights all proposals on one temporary candidate in stable property-ID then object-ID order. Skip only inapplicable pairs. Return exact changes, exact no-ops, and unsupported pairs for review. Reject actual read-only/type/label/topology failures coherently; users must select an editable subgroup or explicitly make a pin local. Matching values are no-ops even on read-only fields and do not manufacture history. This ordered preflight is not a solver for simultaneous label swaps or topology migrations.

Commit the returned ordinary `SetProperty` commands as one batch using the session revision observed during planning. Each command revalidates the current model and stores only its previous typed value as the inverse. Late failure rolls back the completed prefix; stale tokens, no-ops, saved checkpoints, and redo follow existing session contracts. Compound make-local plus property changes undo to the exact original pin. No property snapshot or projection becomes authoritative.

## Defaults/style boundary and cost

This slice changes existing fields only: project v4 and library v1 persistence stay unchanged and existing golden fixtures remain valid. Creation defaults initialize future objects; inherited styles remain linked per property with explicit override intent. Implement those as distinct upcoming capabilities against this inspection/edit contract. Fonts, text size, and visual stroke width belong to drawing representations; physical wire cross-section is a different engineering property. No drawing, default/style persistence, renderer, or desktop inspector is implied by this foundation.

Planning currently clones the full project temporarily; history retains localized commands. Inspection visits selected fields and does not mutate the document. Release probes at 20/200/2,000 selected device/conductor objects measure inspection, preview, commit, and undo with applicable-subset, round-trip, and exact-restoration checks. Treat these single samples as exploratory; revisit allocation and candidate strategy when representative selections show a need.

## Verification

Contracts cover every registry field on supported object kinds, stable keys and value/reference types, union applicability, mixed/uniform/absent values, duplicate selections, restrictions, blank/duplicate labels, topology violations, pin ownership/root parameters, make-local compound undo, stale/deleted targets, late-failure rollback, no-op redo preservation, saved checkpoints, and save/reopen. A temporary-file example demonstrates compounded edits across several subsets of one retained selection. Current native execution is macOS/aarch64; Windows/Linux remain pending.
