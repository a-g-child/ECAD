# 0027: Portable project metadata and sheet/template scope

Status: metadata foundation implemented; sheet geometry, profiles and templates are planned.

## Decision

Include structured sheets/title-block tables and reusable standard, draft, cover and generated-contents layouts as product scope. Separate profile, project, sheet and derived values; templates use stable bindings. Profile values are copied into projects explicitly so another user's machine settings cannot alter drawing identity or historical responsibilities. Detailed scope and delivery sequence live in [sheets and templates](../sheets-and-templates.md).

Start with ProjectMetadata and eight typed MetadataField keys: company.name, company.address, engineer.name, project.title, project.product, project.client, project.contract and project.issue. Values are literal multiline text. Empty strings omit entries, and absent values remain observable. Contact engineer is not an author/checker/approver. Project issue is an explicit engineering value, independent of Git, internal undo revisions and future sheet/template revisions.

Use a separate Command::ProjectMetadata with an exact localized inverse, rollback/no-op/checkpoint behavior through the existing session. Register document.metadata.edit centrally, deliberately unbound. The action requests an editor or accepts typed metadata parameters. File ribbon, File menu and command search share it. Bridge updates validate captured context/revision, strict semantic keys and monotonic input IDs before dispatching. The frontend holds a local draft; Apply replaces all fields once, Cancel discards it, and unchanged Apply preserves redo. Field bodies scroll while headings, feedback and Apply/Cancel retain fixed bounds.

Project schema v7 adds a canonical metadata map beside project_id. Separate versioned DTOs retain strict legacy v5/v6 decoding; v2–v6 receive empty metadata, while v1 migration remains explicit. Old fixtures are retained alongside new v7 golden outputs. Unknown keys, duplicate keys and non-string persisted values are errors. Project metadata is explicitly rejected inside reusable electrical subsystem bodies rather than silently lost by their distinct schema.

No template cells or personal profile paths are persisted by this slice. Future sheet metadata must not be grafted onto drawing views or electrical objects; profile and template stores remain separate from project undo, with explicit copied/adopted project changes.

## Evidence and next work

Three adapter workflows verify literal multiline values, canonical focused save/reopen, exact undo/redo/checkpoints, no-op redo, legacy upgrades, strict malformed data, transaction rollback and subsystem-body rejection. Two bridge regressions verify registered requests, portable values, unchanged scene/electrical records, stale/replayed rejection and history. Two Chromium workflows verify File/menu/palette access, complete saved metadata, independent frontend reload, multiline input ownership, cancel/no-op history and 960 × 640 layout.

Local macOS/aarch64 checks: cargo xtask full (290 behavior tests and two doctests), cargo xtask ui (27 workflows), cargo xtask desktop, embedded-asset debug build and native webview-to-core launch. Windows/Linux native execution, personal-profile storage, table editing, binding placement and output parity remain pending. Next slices are explicit profile import, then sheets/paper/view placement and structured cells/bindings.
