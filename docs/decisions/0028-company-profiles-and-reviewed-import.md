# 0028: Company/contact profiles and reviewed project imports

Status: implemented for named multiline company/contact profiles; project schema v7 is unchanged.

## Decision

Add application-owned CompanyProfile with stable ProfileId, nonblank display name and three permitted metadata fields: company.name, company.address and engineer.name. Keep project titles, contracts, engineering issues and author/checker/approver responsibilities outside profile scope. Saving or renaming a profile changes personal defaults only; existing projects retain copied values.

Register workspace.profiles, unbound, for File ribbon/menu and command search. The modal separates Save profile, Preview import and Apply import. Nonempty saved fields are initially selected; empty values require explicit selection to clear project entries. Show exact changed old/new values in the review. Close/Escape invalidates the retained proposal. Unsaved profile edits require Save before preview; Reload discards the draft. Profile selection is disabled while its draft is edited.

Use a pure ProfileImportPreview containing the original session revision, complete proposed metadata and exact field changes. Import preparation preserves all unselected fields. Commit dispatches the closed proposal through document.metadata.edit and Command::ProjectMetadata, with both document revision and proposal-token guards. Invalid preview attempts, profile reload/save, project replacement and cancellation invalidate held proposals. No-op imports preserve redo and checkpoints. Saved profile content is an observed snapshot; later external profile changes do not rewrite a reviewed copy.

Use a separate strict profile-v1 file with canonical identity ordering, stable IDs, unique names and three literal string values. Reject malformed, duplicate, unknown and unsupported data. ProfileFile loads a missing file as an absent snapshot without writes. Explicit saves create parent directories, encode/stage/sync, check observed bytes/location, then replace or create without clobbering an unobserved destination. These checks are optimistic; uncooperative writers can race between check and replacement. A failed refresh revokes the old write observation. Errors preserve existing file bytes and project state.

The native host configures company-profiles.json inside its Tauri-provided application configuration directory without startup writes. The browser harness uses a file beside its isolated project or an explicit ECAD_UI_PROFILES path. No browser localStorage or project-relative profile reference is authoritative. Project files remain portable and contain only copied metadata values.

## Verification and remaining scope

Two application contracts cover field scope, exact literal changes and explicit clearing. Two adapter contracts cover absent reads, canonical storage, observed-version/location conflicts and malformed/duplicate/unknown/future files. Three bridge workflows cover independent history, save/reopen/rename/two-project behavior, selective copies, cancellation, invalidation, stale commits, no-op redo, failed reload and conflict recovery. Two Chromium workflows exercise real profile files, review/apply/cancel, multiline text, 960 × 640 controls, restart/reload and external-write rejection.

Local macOS/aarch64 checks: cargo xtask full (297 behavior tests and two doctests), cargo xtask ui (29 workflows), cargo xtask desktop, embedded-asset debug build and native webview-to-core launch. Native profile interaction automation and Windows/Linux execution remain pending. Profile deletion, automatic default selection, logos and portable profile import/export packaging are future slices. Next sheet work is paper settings, numbering, drawing sets and view placement before structured table cells and bindings.
