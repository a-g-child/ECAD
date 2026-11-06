//! Revision guards, saved checkpoints, and bounded transaction history.

use crate::{Command, EditError, ProjectStore, Revision, transaction::Transaction};
use ecad_domain::Project;
use std::{collections::VecDeque, num::NonZeroUsize};

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;

#[derive(Clone, Debug)]
struct HistoryEntry {
    transaction: Transaction,
    before: u64,
    after: u64,
}

/// One document with read-only queries, atomic edits, and bounded history.
///
/// History is capped by transaction count, not bytes. Every new or reopened session
/// gets a fresh process-local epoch and empty history. Use [`Self::open`] to start
/// from a clean saved checkpoint; [`Self::new`] starts unsaved.
pub struct Session {
    project: Project,
    revision: Revision,
    state: u64,
    saved_state: Option<u64>,
    undo: VecDeque<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    history_limit: NonZeroUsize,
}

impl Session {
    /// Begin an unsaved session with capacity for 256 undoable transactions.
    pub fn new(project: Project) -> Self {
        Self::with_history_limit(
            project,
            NonZeroUsize::new(256).expect("constant is nonzero"),
        )
    }

    /// Begin an unsaved session with a nonzero transaction-history capacity.
    /// No project data is copied.
    pub fn with_history_limit(project: Project, history_limit: NonZeroUsize) -> Self {
        Self {
            project,
            revision: Revision::fresh(),
            state: 0,
            saved_state: None,
            undo: VecDeque::new(),
            redo: Vec::new(),
            history_limit,
        }
    }

    /// Load a structurally valid project as a clean session with a fresh epoch.
    /// # Errors
    /// Preserves the store's IO, schema, and model failures without exposing a session.
    pub fn open<S: ProjectStore>(store: &S) -> Result<Self, S::Error> {
        let mut session = Self::new(store.load()?);
        session.saved_state = Some(session.state);
        Ok(session)
    }

    /// Save the current state, then mark that exact history position as clean.
    ///
    /// Exclusive access prevents concurrent commands from racing the checkpoint.
    /// Success leaves revision and history unchanged; failure preserves dirty state.
    /// # Errors
    /// Returns the store's original save failure. Store durability is adapter-specific.
    pub fn save<S: ProjectStore>(&mut self, store: &S) -> Result<(), S::Error> {
        store.save(&self.project)?;
        self.saved_state = Some(self.state);
        Ok(())
    }

    /// Whether this history state differs from the last successful save/open.
    ///
    /// Undo/redo to the saved position clears dirty state. Equivalent contents
    /// produced through different edits are conservatively treated as dirty;
    /// no full-document equality scan runs on each edit.
    pub fn is_dirty(&self) -> bool {
        self.saved_state != Some(self.state)
    }

    /// Borrow the authoritative model without granting mutation outside commands.
    pub fn project(&self) -> &Project {
        &self.project
    }

    /// Return the epoch and sequence required when submitting an edit.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// Return the number of retained transactions available for undo.
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    /// Return the number of undone transactions available for redo.
    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }

    /// Commit one edit, or return `false` if it has no effect.
    /// # Errors
    /// Rejects another session's token, stale sequences, invalid edits, or overflow.
    /// Failure and no-ops preserve history, revision, and saved state.
    pub fn execute(&mut self, expected: Revision, command: Command) -> Result<bool, EditError> {
        self.execute_batch(expected, vec![command])
    }

    /// Commit ordered edits as one undoable transaction.
    ///
    /// Each edit sees preceding edits in the batch. Failure rolls back the entire
    /// prefix before returning. Intermediate states and notifications never escape
    /// exclusive session access. No project-wide snapshot is made.
    ///
    /// Empty/all-no-op batches leave history unchanged. A batch with actual edits
    /// that later cancel each other is conservatively recorded as one transaction.
    /// # Errors
    /// Rejects stale/cross-session tokens, invalid edits, or overflow without
    /// changing the document, history, saved checkpoint, or revision.
    pub fn execute_batch(
        &mut self,
        expected: Revision,
        commands: Vec<Command>,
    ) -> Result<bool, EditError> {
        self.check_revision(expected)?;
        let next = self.next_revision()?;
        let Some(transaction) = Transaction::apply(&mut self.project, commands)? else {
            return Ok(false);
        };
        self.undo.push_back(HistoryEntry {
            transaction,
            before: self.state,
            after: next.get(),
        });
        if self.undo.len() > self.history_limit.get() {
            self.undo.pop_front();
        }
        self.redo.clear();
        self.state = next.get();
        self.revision = next;
        Ok(true)
    }

    /// Undo one complete transaction, restoring identities and saved-state position.
    /// Returns `false` if no retained transaction exists.
    /// # Errors
    /// Rejects stale/cross-session tokens, overflow, or an invalid inverse edit.
    pub fn undo(&mut self, expected: Revision) -> Result<bool, EditError> {
        self.check_revision(expected)?;
        let Some(entry) = self.undo.back() else {
            return Ok(false);
        };
        let next = self.next_revision()?;
        entry.transaction.undo(&mut self.project)?;
        let entry = self
            .undo
            .pop_back()
            .expect("entry was inspected under exclusive session access");
        self.state = entry.before;
        self.redo.push(entry);
        self.revision = next;
        Ok(true)
    }

    /// Redo one undone transaction with its original identities and values.
    /// Returns `false` if no redo transaction exists.
    /// # Errors
    /// Rejects stale/cross-session tokens, overflow, or an invalid forward edit.
    pub fn redo(&mut self, expected: Revision) -> Result<bool, EditError> {
        self.check_revision(expected)?;
        let Some(entry) = self.redo.last() else {
            return Ok(false);
        };
        let next = self.next_revision()?;
        entry.transaction.redo(&mut self.project)?;
        let entry = self
            .redo
            .pop()
            .expect("entry was inspected under exclusive session access");
        self.state = entry.after;
        self.undo.push_back(entry);
        self.revision = next;
        Ok(true)
    }

    fn check_revision(&self, expected: Revision) -> Result<(), EditError> {
        if expected == self.revision {
            Ok(())
        } else {
            Err(EditError::StaleRevision {
                expected,
                actual: self.revision,
            })
        }
    }

    fn next_revision(&self) -> Result<Revision, EditError> {
        self.revision.next().ok_or(EditError::RevisionExhausted)
    }
}
