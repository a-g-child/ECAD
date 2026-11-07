# 0008: Advisory usage discovery for registered project files

Status: accepted for explicit local registration and in-memory observations.

## Decision

Extract `ProjectUsage` from a validated project without changing authoritative data. Record project lineage and every bundled revision, with distinct sets of exactly pinned roots and editable local roots retaining source ancestry. Empty sets mean bundled-only content, not a placed instance. Reference keys are stable definition/revision IDs, never matching names or terminal numbers.

Keep observations in the application layer's optional `KnownUseIndex<K, V>`, keyed by caller-owned location identity with adapter-owned content/version tokens. Register, unregister, refresh, check freshness, and query lineage references without entering project command history. Query results are revision/location pairs; callers must not mistake their count for distinct projects or placed instances. Expose the complete registration list, including files with no observation or no matching source, so empty search results cannot hide incomplete coverage.

`KnownProjects` binds this index to explicit local file paths and the existing read-only operations of `VersionedProjectStore`. Register files individually; no implicit directory traversal or remote repository lookup occurs. Refresh loads and validates a whole saved project, extracts its summary, and drops the model. Content checks compare exact-byte/path versions without reloading the model. Batch operations continue after individual failures and report attempted locations plus typed per-file failures. No write boundary is called.

## Freshness and failure

Separate the last validated observation from the result of the latest explicit operation: unscanned, observed, matches observed bytes, changed, missing, or unavailable. Successful refresh replaces the observation. Failed refresh/check retains historical usage, records a diagnostic, and returns the original typed error. Missing/changed sources retain references rather than disappearing from impact review. A valid later refresh may replace all old references or change the observed project lineage at that path.

Statuses describe the latest operation, not continuous freshness or wall-clock age. An observed/matching file may change immediately afterward; no global atomic scan or watcher is implied. Restoring identical validated bytes can match the previous observation again. A content check without a prior valid load cannot manufacture validated usage. Unsaved live edits and other Git branches are outside a saved-file observation.

## Location policy and limits

Resolve existing file aliases through canonicalization and missing files through their existing canonical parent. Reject directories and dangling symlinks. Duplicate canonical registration preserves observations. Copies and worktrees remain separate locations even if they share `ProjectId`; do not claim a count of unique consumers or infer which copy is authoritative. Hard-link aliases are not deduplicated. Paths track resolved locations rather than symlink retargets; moved files require explicit registration and forgetting the old location.

This first index and its registration list are intentionally in memory. Missing/stale cache data never blocks opening/editing a project or publishing an immutable library revision. Durable workspace registration, cache persistence/SQLite, directory discovery, open/save hooks, watchers, timestamps, cancellation, scan budgets, native permissions behavior, and cross-machine coverage remain future work. Measure full-load cost on representative registered projects before introducing metadata shortcuts or background scheduling; validation must not silently bless malformed sources.

## Verification

Headless application contracts verify unknown registrations do no IO, duplicate registration preserves observations, checks cannot invent valid usage, failures retain observations and typed error sources, and discovery never calls the write boundary. Adapter contracts verify pin/local/bundled-only classification, partial coverage, historical hits after missing/invalid files, valid replacement, copied/moved files, canonical aliases, same-length content changes, and a real isolated Git branch checkout. Unix symlink coverage is conditional. A temporary-file example shows partial coverage and local ancestry becoming a labelled historical reference. Native Windows/Linux execution remains pending.
