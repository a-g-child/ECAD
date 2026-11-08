# 0038: Publish complete immutable template revisions

Status: accepted for local filesystem template libraries. Extends [decision 0037](0037-library-template-editor-workspace.md); follows the cooperating-writer contract in [decision 0007](0007-shared-subsystem-library.md).

## Decision

A `TemplateRevision` owns a nonempty complete set of saved paper/layout variants. Every variant must match the enclosing definition/revision pin; variant IDs are unique and iterated canonically. A `TemplateVariantRef` resolves through that exact revision. Adding, removing, renaming or changing a variant requires a new revision, even when earlier variant bodies would remain unchanged. Publishing variants individually into an already published revision would make its contents mutable, so publication accepts complete revisions only.

`TemplateLibrary` retains append-only revision bodies plus optional explicit per-definition recommendations. Identical re-publication is a no-op. Different content under an existing revision pin is rejected. Recommendations may move backward or be cleared without removing bodies; IDs imply no chronological order. Exact variant resolution never follows a recommendation. This catalogue has no project references or project undo stack.

`TemplatePublisher<S>` uses the application-owned `VersionedTemplateLibraryStore` boundary. It persists a complete candidate, including optional recommendation changes, before updating its observed catalogue/version. Even no-op publication checks the observed file version. Failed saves/reloads preserve local state. Stale callers explicitly reload and retry. This controller is separate from the future draft editor; publishing cannot mark a draft clean until its own save lifecycle acknowledges success.

## Storage

`TemplateLibraryFile` uses a strict canonical catalogue schema v1 marked `ecad-template-library`, independent of project schema v11 and standalone variant schema v1. Complete revision records embed the existing strict variant DTOs, keeping model/persistence boundaries distinct. Unknown fields, nested versions, foreign pins, duplicate records, invalid layouts and unresolved recommendations reject the entire load. Files contain no timestamps, sample project values, user-library paths or editor history. Revisions/variants/recommendations have stable identity order; cells retain their semantic presentation order.

Both concrete library adapters share canonical-parent path and create-new writer-lock mechanics. The existing `.<filename>.ecad-library-lock` convention remains unchanged. Hold the lock across read/version check, append-only history check, staging and replacement. Destination/version tokens include location and exact bytes. Unobserved destinations permit creation only. The file adapter rejects removal/rewrite of published complete revisions even when a caller bypasses the publisher controller. Exact-byte no-ops avoid replacement.

Retained locks are reported, never automatically removed or stolen. Destination symlinks are rejected; canonical parent aliases share the lock. Staged contents are synced before replacement. Uncooperative writers, hard-link aliases, cross-machine/network filesystem semantics, parent-directory durability and permission preservation remain outside the guarantee. This extraction does not change subsystem library behavior or project save guarantees.

## Delivery boundary and review triggers

This slice publishes owned table/layout bodies, not reusable block/asset graphs. Transitive dependency validation/bundling comes with those definitions. Project bundling/import, the deterministic preservation migration, saved-template assignment/defaults, adoption, archive/usage discovery and native library/editor UI remain separate work. Existing projects still use the three enum frames and their current paper controls; the template library is not consulted while opening/rendering them.

A local executable example publishes/reopens two saved paper variants. Real-file tests cover closed revisions, strict canonical fixtures, recommendations, exact pins, stale/no-op writes, location tokens, append-only storage, simultaneous creators, retained locks, failed IO/reloads, parent aliases and Unix destination-symlink rejection. Existing subsystem publication tests protect the shared coordination extraction. Native Windows/Linux filesystem execution remains pending.

Revisit whole-file catalogue partitioning/indexes only when measurements justify it. Extend immutable revision dependency closure before reusable blocks/assets ship. Complete project bundling/migration before removing inline paper authoring or offering these saved variants in project pickers.
