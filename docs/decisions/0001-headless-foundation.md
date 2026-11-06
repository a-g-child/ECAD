# 0001 — Start with a headless command and persistence slice

Status: accepted for the initial implementation.

## Context

The product needs coherent electrical identity, hierarchy, coordinated views, reliable history, and reviewable project data. Beginning with a large frontend would make those contracts difficult to test independently.

## Decision

Start with three working Rust crates: dependency-free electrical domain, application commands/sessions, and explicit persistence adapters. Add a portable local task runner. Keep desktop and speculative future modules absent until a working feature needs them.

The initial model contains device occurrences, terminals, and conductors only. IDs are opaque, caller-supplied nonzero 128-bit values. Definition/revision identities will be separate when reusable hierarchy is introduced. The production generation and cross-project addressing policy remains open.

Commands retain exact forward/inverse values for their affected objects, not whole-project snapshots. Session revisions increase through edits and undo/redo. History has a configurable operation-count bound; byte budgets and session epochs remain open.

Save one experimental v1 JSON file through explicit DTOs, with stable identity ordering and LF formatting. serde/serde_json own encoding concerns inside the adapter; tempfile stages a replacement beside the destination. The domain and application do not depend on these libraries. Unknown fields and future versions are rejected instead of silently discarded.

Use Cargo.lock and offline local checks, including real temporary Git integration. Do not create a hosted CI workflow or remote repository as part of this slice.

## Tradeoffs and limits

One file makes a coherent initial round trip easy to inspect, but is not the final Git partitioning decision. Individual file replacement is not a multi-file transaction or a complete crash-recovery policy. tempfile replacement behavior and durability limitations are documented in its [API reference](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html#method.persist); synchronizing staged contents does not establish parent-directory power-loss durability.

Device removal currently scans conductors to reject deletion of connected anchors. Terminal lookup and termination edits use ordered indexes. Benchmark before adding a reverse connection index and retain tests for index consistency when doing so.

The initial scale probe measures only unique-device chains, encoding/decoding, local termination edits, and undo correctness. It does not establish native UI performance, realistic vehicle complexity, memory capacity, or hierarchy scalability.

## Revisit conditions

- Introduce session epochs and ID-generation policy before transport/reload or cross-project editing.
- Add compound transactions and memory budgets before bulk operations and long editing workloads.
- Revisit file partitioning/recovery with hierarchy, assets, and real concurrent Git changes.
- Revisit indexes and storage structures when profiles demonstrate a bottleneck.
- Confirm desktop/rendering choices through native prototypes rather than making them implicit dependencies of the core.
