# 0016: Electrical connection projections and deterministic derived schedules

Status: implemented headless report/value-source and CSV slice. Native report/export controls and placed live connection labels remain planned.

## Problem

Pin tables, wire lists, cable schedules, and future mechanical handoff must agree with the electrical model. Using formatted labels, generated artwork, or one arbitrarily chosen connection per terminal loses identity and wiring information. Drawing copies and bundled reusable definitions must not inflate physical occurrence counts. Generated output should remain useful in Git without embedding volatile session metadata.

## Decision

The application owns an immutable `ConnectionReport` captured from one validated session. It stores typed-ID maps for actual subsystems, devices, terminals, conductors, and cables, with separate current display labels. It traverses objects/attachments once and builds terminal adjacency from actual conductor A/B ends. Self-loops retain both ends; all unconnected terminals, free conductors, spare ends, and empty assemblies remain represented. Bundled source bodies and graphical records are excluded. Hierarchy is resolved iteratively from report-owned occurrence references.

The report retains persistent lineage plus its originating session epoch/sequence. Reports are observations, not a second mutable electrical model. Requery after edits to refresh values. Freshness is conservative even after geometry-only edits, undo to equal content, or reopening. Native automatic refresh and persisted live-label bindings are separate future work.

Three adapter-owned CSV encoders consume the same snapshot: one row per wire, one per attached terminal end (or unconnected terminal), and one per cable core (or empty cable). Cable overview endpoints and actual core terminal assignments remain distinct. Completeness/draft states are explicit. No missing physical attributes or user wire numbers are invented. See the [output contract](../connection-reports.md) for row and column semantics.

The default current-value encoding helper checks the report against the supplied session and preserves a typed stale-revision failure. A separate historical encoder intentionally accepts old immutable snapshots. Both return in-memory strings and perform no output file IO. Generated schedules are not authoritative project storage and do not enter undo history.

CSV uses fixed headers, canonical ID order, quoted UTF-8 cells, doubled embedded quotes, and LF records. Fields retain embedded newlines. Functional paths include stable ID sequences plus unambiguous compact JSON name arrays. Revisions, epochs, timestamps, file locations, drawing data, and caches are excluded, preserving byte identity across equal electrical contents and reopened sessions. CSV fixtures have explicit LF checkout attributes. No project or keymap schema change is introduced.

The shared `reports.connections` action is read-only, document-scoped, and deliberately unbound but remappable/searchable. It returns a current projection with normal availability, captured revision, text/IME protection, and replay checks. It does not imply a report viewer, file chooser, or background watcher.

## Validation and revisit conditions

Tests cover exact terminal/link coverage, self-loops, drafts, empty and spare-core cables, overview/core distinction, current labels/hierarchy/assignments, immutable historical observations, freshness guards, no document/history/file mutation, drawing independence, independent placed instances versus bundled-only bodies, escaping, independently authored golden CSV, and unchanged output after undo/save/reopen. An executable example demonstrates refreshing and exporting values. Separate release probes time report construction and each CSV at 100/1,000/10,000 paired devices with individual cables and fixed two-level hierarchy, verifying coverage and round-trip stability.

Revisit when physical wire data, user numbering, connector profiles/templates, placed live labels, report filters, native refresh/cancellation, or persistent export destinations are introduced. Preserve full connection multiplicity and explicit snapshot freshness. Stream output and index targeted queries when measured workloads justify them; current recipes do not establish memory budgets, deep-hierarchy/high-degree capacity, or physical-build documentation completeness.
