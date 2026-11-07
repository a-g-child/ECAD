# 0007: Local shared subsystem publication

Status: accepted for local filesystem libraries.

## Decision

Keep published subsystem content outside project editing history in `SubsystemLibrary`. Retain immutable lineage/revision bodies and optional explicit recommendation pointers. Identical publication is a no-op; a different body under an existing key fails. Recommendations resolve only to published revisions in that lineage. They may point backward or be cleared without deleting history. Opaque revision IDs imply no chronological order.

`LibraryPublisher<S>` coordinates persistence through `VersionedLibraryStore`. Publish a body and optional recommendation in one persisted change; update local catalog/version only after success. Validate storage versions even for no-ops. Stale clients explicitly reload and retry; failed publication or reload preserves the previously observed state. This controller has no project undo stack.

Use an independent, strict canonical library JSON schema v1, marked `ecad-subsystem-library`. Reuse the project revision-body DTO and validation rather than serializing domain structs or allowing nested catalogs. Reject duplicate revision/recommendation entries, unknown fields, unresolved pointers, and malformed bodies. Retain a golden fixture. Project schema remains v4 and its existing fixtures stay unchanged.

## Coherent writes and limits

`LibraryFile` canonicalizes the parent path and acquires an exclusive `create_new` sidecar lock for the entire read/version-check/history-check/stage/replace sequence. The reserved name is `.<filename>.ecad-library-lock`. Compare observed path and SHA-256 bytes under the lock; an unobserved destination may only be created, never overwritten. Check every previous body is still present and unchanged even if callers bypass the publisher controller. Sync staged contents before replacement. Readers do not acquire a lock and see a complete old or new catalog on supported filesystems. Successful exact-byte no-ops skip replacement.

This coordinates writers using this adapter, assuming reliable exclusive creation and replacement. Canonical parent aliases share the lock; destination symlinks are rejected. Hard-link aliases, case aliases not resolved by parent canonicalization, uncooperative writers, network/cross-machine filesystem behavior, directory durability, permissions/metadata preservation, and power-loss recovery remain outside the guarantee. Existing project saves retain their separate optimistic check/replace behavior.

Close the lock handle before unlinking for portability. A failed operation normally releases it. Crashes or cleanup failures may retain it and block future writes. Report the lock path; never infer ownership from age or automatically steal it. A future recovery interface must establish that no writer is active before explicit cleanup. No lock is held during user review or ordinary project editing.

## Consuming projects

`library_import` resolves an explicitly selected exact revision and returns an ordinary project command. Import, placement, and adoption remain separate; a recommendation change edits no consuming project. Imported content remains bundled for offline reopening. Undoing project adoption/import does not erase shared publication or change its recommendation. Publishing a subsequent revision or explicitly changing the recommendation is the supported shared-history response to a mistake.

The initial publication input is a validated standalone hierarchy. Arbitrary-subtree capture, promotion of a local variant with ancestry, known-use discovery, publication UI, external distribution, nested dependencies, and local-variant merging remain separate work. JSON keeps the first catalog inspectable and Git-friendly; SQLite indexing is not required for authoritative storage. Revisit partitioning/indexes and resource budgets when measured catalog size, publication/load cost, and asset needs justify them.

## Verification

Local contracts cover immutable keys, recommendation validation/reversion/clearing, strict golden persistence, malformed bodies, no-ops, stale and concurrent writers, held locks, rejected history rewrites, observed path tokens, canonical parent aliases, failed IO/reload, and Unix symlink rejection. Two consuming projects verify unchanged files/pins after publication, explicit import/adoption, offline reopening, and project undo leaving shared history intact. A temporary-file example demonstrates the publication workflow. Native Windows/Linux and filesystem durability testing remain pending.
