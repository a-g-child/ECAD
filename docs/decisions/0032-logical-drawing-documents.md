# 0032: Logical drawing-document ownership and deterministic upgrades

Status: headless model, command/history, strict schema and bridge projection implemented. Desktop document management, templates and unified sheet editing remain pending.

## Decision

Introduce DrawingDocumentId and ProjectDrawingDocument for logical package metadata: number, multiline title and engineering revision. Keep these distinct from DrawingDocument, the existing geometry aggregate, and Document<S>, the session/save controller. Projects start with one empty Drawings package and retain at least one package. Document numbers are unique within the project. Sheet drawing numbers keep the v8 project-wide uniqueness rule; transfer/reorder never renumbers them.

Each sheet has exactly one owner. Keep the existing sheet carrier and a canonical flattened sequence in document order then owner-local order, plus a derived ownership index. Expose explicit owner-local insertion and transfer; moving a document moves its entire sheet group without changing any records. Existing flattened insertion remains a compatibility method: use the preceding sheet's owner at boundaries, or the first document at index zero. Legacy flattened moves must stay inside their owner; no implicit reparenting. Sheet removal inverses retain the original owner and owner-local index, avoiding incorrect restoration into an adjacent/empty package.

DrawingDocumentCommand supports insert/replace/remove/reorder, explicit sheet transfers and complete metadata/membership replacement. Remove requires an empty package and another remaining package. Inverses store localized metadata/ID lists/order, not geometry snapshots. Validate complete layouts before mutation: every sheet must exist and occur exactly once. Failed batches restore exact prior ownership. No-op operations preserve redo/checkpoints. Register drawing.documents.edit with a deliberate unbound default; its desktop editor is planned, while typed dispatch and read projection are working.

Schema v9 stores ordered drawing_documents with their ordered sheet-ID lists. Physical sheet records are canonical by identity; membership lists alone define semantic order. Decode through strict independent v8/v7 conversions, then validate/install the v9 layout atomically. V2–v8 load into a deterministic initial package whose typed ID uses the project ID's canonical value in a separate namespace. This is a default-package identity rule, not a general identity conversion. New nondefault packages use supplied/generated identities, with UUID-v4 generation in the adapter. Ordinary loads never allocate random IDs.

Legacy sheet IDs, numbers, transforms, source geometry/styles and electrical objects remain intact. Shared source views remain shared, and unplaced views remain retained. Wrapping those views into template-backed sheets and exposing shared-edit implications require the actual template/sheet editor migration; this slice does not claim that work complete. Custom document metadata is rejected in electrical reusable bodies rather than discarded by their distinct schema; canonical empty initial-package metadata is the only implicit default there.

Bridge document rows are independent DTOs carrying IDs/metadata/membership. The current flat sheet UI can still edit paper and undo removal without losing owners. It is not the target document/page explorer. Document default-template fields are deferred until they can resolve to a real frame rather than placeholder references.

## Evidence and remaining work

Three domain, four adapter and one bridge workflows cover metadata/membership failure atomicity, document order/transfer, strict malformed ownership, v2–v8 upgrades, stable default IDs, shared/unplaced view preservation, exact save/reopen/history/rollback/no-ops, action registration, nondefault-owner removal undo and reusable-body rejection. The executable drawing_documents example verifies two packages, unchanged shared geometry and exact undo/v9 round trip.

Local macOS/aarch64 verification: cargo xtask full (318 behavior tests and two doctests), cargo xtask ui (31 existing workflows), cargo xtask desktop, embedded-asset debug native build/launch and the offline example. Native document-manager interaction, direct sheet editing, template resolution, multi-project desktop routing and Windows/Linux execution are not established by these tests. Continue with the actual basic CAD frame and reviewed free-view-to-sheet migration before switching workspace input/rendering.
