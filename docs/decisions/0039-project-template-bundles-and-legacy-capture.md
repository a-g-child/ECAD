# 0039: Project-owned template bodies and explicit legacy preservation capture

Status: accepted for the headless bundle/preservation stage. Saved-template assignment migration and editor UI remain pending. Extends [0037](0037-library-template-editor-workspace.md) and [0038](0038-closed-template-revision-publication.md).

## Decision

Projects can bundle exact complete `TemplateRevision` bodies, separately from electrical subsystem revisions and separately from a shared template library. Imports are explicit ordinary project commands with localized exact inverses. Identical imports are no-ops; different content under an existing pin fails without mutation. Compound import failures restore any inserted prefix. Publication/recommendation changes never import or adopt a project template automatically, and project undo never retracts shared publication.

Project schema v12 adds a mandatory identity-ordered `template_revisions` array alongside the existing v11 fields. Reuse the strict complete-revision/variant DTO conversion used by the template library; domain objects are not serialized directly. Reject unknown fields, duplicate complete revisions, malformed layouts and mismatched nested pins. Complete current bodies contain owned tables, so there are no unresolved external block/asset dependencies. Bundled bodies reopen without a library path. Electrical subsystem capture explicitly rejects template-bearing project content rather than silently dropping it.

V2–v11 projects load with no invented template bodies; v1 still requires explicit lineage migration. Explicit saves write v12, including an empty array when no template has been imported. Preserve earlier strict DTOs and historical fixtures. Keep canonical v12 fixtures for the unchanged electrical/drawing model plus the new empty bundle field.

## Legacy preservation stage

`capture_legacy_templates` is a read-only closed proposal at the original session revision. It materializes every current sheet's resolved enum/paper combination, and default-paper variants for all document defaults and the three project creation defaults, including unused choices. It records sheet identities and explicit-pin versus document-inheritance intent. Actual inherited sheets retain their own paper-specific capture; document/default samples do not replace their paper.

The proposal imports complete saved layout/binding bodies through one ordinary batch. Repetition reuses identical bodies and is a no-op. No electrical identities, document membership, sheet order/numbers/titles, view sharing, placements, source coordinates, metadata or appearance state change. Tiny legacy sheets retain their exact paper/border/content region with an omitted table, rather than being normalized or rejected. Current enum fields and editable paper controls remain authoritative for actual sheet rendering during this stage. Capture does not yet assign the saved bodies to sheets or silently change inheritance.

Identity inputs use the versioned `ecad-legacy-template-capture-v1` recipe. Definition IDs derive from the frozen enum key; variant IDs derive from six exact big-endian u32 paper/margin values. Revision IDs derive from the canonical complete variant body using a fixed placeholder revision, so the final identity is not self-referential. SHA-256 prefixes become nonzero 128-bit typed IDs; immutable-body comparisons reject conflicting existing pins. Inputs exclude project lineage/path, timestamps, random IDs and resolved metadata. Equal layouts in different projects have equal pins. Golden definition/variant IDs and repeat/reopen checks protect the recipe. Review/bump the recipe when its semantics change; do not overwrite a stored body under an old pin.

## Boundaries and next migration

This stage creates portable preserved bodies and proves geometry/history behavior before replacing the assignment contract. It does not claim the saved-template migration is complete. Converting sheet/document/project defaults to saved references must retain inheritance/override intent and styles, validate all assigned bodies, and add in-use removal guards before references can be created. Current body removal is limited to unassigned imported content. Transitive block/assets, richer style/field schemas, saved-only pickers and the independent editor remain follow-up work.

Local tests cover offline imported bodies, library independence, exact saved checkpoints/undo, import conflicts/rollback, stable cross-project capture IDs, stale proposals, custom margins, shared/unplaced views, tiny sheets, equal-valued explicit frame overrides, unchanged source records, strict v12 bodies and electrical-reuse rejection. The offline `project_templates` example exercises capture, project save/reopen and exact history through `Document`. Native Windows/Linux execution remains separately verified work.
