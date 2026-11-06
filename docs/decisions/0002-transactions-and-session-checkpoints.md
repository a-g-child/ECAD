# 0002 — Transactions, saved checkpoints, and session epochs

Status: accepted for the next headless lifecycle slice. Extends [0001](0001-headless-foundation.md) without changing the project-file schema.

## Context

Multi-object placement and future bulk editing need one history operation. Comparing only numeric revision sequences can accept a stale command after reopening a document. Dirty state cannot be tied directly to revision numbers because undo advances the revision while potentially restoring the last saved document.

## Decision

Add ordered `Session::execute_batch` operations. Commands remain individual electrical edits; the application records their forward/inverse changes in one transaction. A command can depend on preceding commands in that transaction, for example creating two devices followed by a conductor attached to their terminals.

Apply and validate under exclusive mutable session access. If any operation fails, replay the completed prefix's inverses in reverse order before returning the error. No observer or notification sees intermediate states. This avoids copying the entire project merely to stage a small edit. Undo and redo use the same reversible transaction path and retain the original object identities.

This is in-memory document atomicity, not a filesystem transaction. No IO or external side effects are permitted in these commands. An inverse failing during rollback is an internal implementation invariant failure, not an expected user error. Regression cases exercise failure at every position in the representative batch, including restoration of deleted devices and their connections.

Give each session a unique process-local epoch from an atomic counter. Revision tokens contain both that epoch and a monotonic sequence. Save/reopen creates a new epoch; tokens are not persisted or intended to survive application restart. A future bridge must carry the complete token, not just its sequence number.

Track history-state identity separately from revision identity. Each successful edit receives a new state ID; history records its before/after state IDs. Undo/redo restore those state IDs while advancing the revision sequence. The saved checkpoint points to the state last loaded or successfully saved.

`Session::new` is unsaved. `Session::open` starts from a clean checkpoint with empty history. `Session::save` updates its checkpoint only after the store reports success, leaving both history and revision unchanged.

## Tradeoffs

Empty and all-no-op batches create no history entry. A group of actual changes that cancel each other remains a transaction, and independently recreated equal contents remain conservatively dirty. Exact history-position tracking avoids false clean states without a full model equality scan or content hashing on every interaction.

The session history count limits transactions rather than constituent commands. Byte budgets and maximum batch/input sizes remain future work. Epochs are process-local; they do not replace production object generation or persistent project/definition identities.

The initial API holds exclusive session access throughout synchronous saving. Background saving will need a captured state/snapshot token and completion checks rather than marking whichever document happens to be current as saved.

## Revisit conditions

- Before IPC: specify DTO validation for both epoch and sequence, and reset projections after a core restart.
- Before background saves/reload: define completion tokens, external-change checks, and document replacement.
- Before large bulk edits: add measured transaction/history resource budgets.
- If exact content-equivalence dirty detection is required: evaluate incremental fingerprints against the cost and collision policy.
