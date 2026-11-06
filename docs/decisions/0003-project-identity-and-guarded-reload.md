# 0003 — Persistent project identity and guarded file lifecycle

Status: accepted for the next headless slice. Extends [0002](0002-transactions-and-session-checkpoints.md).

## Identity and format

Make project lineage explicit with a typed `ProjectId`. Domain construction requires it; there is no fixed default identity for a new design. UUID-v4 generation lives in the adapter using fallible OS entropy and the uuid formatter. Domain/application crates retain their existing dependency boundaries. Explicit nonzero constructors support deterministic tests and imports, while insertion checks still reject duplicate object IDs.

Store the lineage ID in schema v2 using the same canonical hexadecimal encoding as existing object IDs. Copies, save-as, and Git branches retain lineage; a distinct design fork must choose a new project identity. This does not yet provide complete clone/remap or reusable-definition addressing workflows.

Do not invent an identity when opening a v1 file. Return a migration-required error. An explicit `migrate_v1` conversion accepts the chosen project ID, retains existing device/terminal/conductor identities, validates all data, and returns a project without modifying the source. Save the converted result to a new destination. Retain both v1 and v2 fixtures and reject unknown fields/versions.

## Responsibility boundaries

The editing `Session` continues to own model/history. Add an application-level `Document<S>` controller owning a session, a versioned store, and its last observed store token. The controller exposes the session read-only and forwards edits through its command boundary. The controller owns save, save-as, reload, and checkpoint orchestration; the file adapter owns hashing and filesystem operations.

Keep low-level `ProjectStore` as an explicit unconditional primitive for fixture construction/import tooling. Normal application editing uses the new `VersionedProjectStore` contract, which reads a validated project/token snapshot, inspects current versions without requiring valid JSON, and conditionally saves against an observed token.

## File versions and saves

The JSON adapter's version consists of a canonical-parent path and SHA-256 of the exact read bytes. Snapshot parsing and token derivation use one byte buffer. This detects same-length edits and invalid replacements without relying on timestamps, and prevents reusing a token at another path. Tokens are storage metadata, never persisted electrical identity.

New/save-as destinations require absence. Existing destinations require the token last loaded or successfully saved. Stage complete text, synchronize staged contents, then recheck the observed version before replacement. On errors/conflicts, do not advance the saved checkpoint or change the current binding. Save-as keeps lineage/history and changes binding only after successful creation.

This is optimistic protection, not atomic compare-and-swap against arbitrary writers. Another process can write between check and replacement. No filesystem lock is claimed; alias/symlink handling, metadata preservation, and complete crash durability remain separate work. SHA-256 token checks also rely on the practical collision resistance of that hash.

## Reload

Require the current session revision token. Reject dirty reloads unless the caller explicitly supplies discard permission. Load and validate a complete candidate, reject a different project lineage, then replace the session and storage token together. Failed reads, malformed files, changed identities, and stale requests preserve the current model, history, and observed storage checkpoint.

Successful reload clears undo/redo and allocates a fresh session epoch. An old canvas/projection token cannot act on the replacement. A Git checkout integration test exercises external detection, conflict rejection, and successful same-lineage adoption.

## Follow-up

Add explicit design-fork semantics with reusable hierarchy, bounded input/history resources, save recovery and concurrent-writer coordination, and native file watching when the desktop host needs it. Current external checks are synchronous and intended for lifecycle operations, not per-frame polling.
