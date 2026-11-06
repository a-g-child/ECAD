//! Bind a session to a versioned store without exposing mutable checkpoint state.

use crate::{Command, DocumentError, EditError, Revision, Session, VersionedProjectStore};
use ecad_domain::Project;

/// Explicit permission governing local edits during a reload operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReloadPolicy {
    /// Reject reload while the document has unsaved local edits.
    RequireClean,
    /// Discard local edits only after the replacement is validated successfully.
    DiscardLocalChanges,
}

/// A document session, its bound store, and the last observed storage revision.
///
/// Use this controller for normal editing. It owns the save/reload checkpoint and
/// exposes the session read-only so raw saves cannot silently change its dirty state.
pub struct Document<S: VersionedProjectStore> {
    session: Session,
    store: S,
    version: Option<S::Version>,
}

impl<S: VersionedProjectStore> Document<S> {
    /// Begin a new unsaved document targeting a not-yet-existing destination.
    /// The first save checks that it is still absent; this performs no IO.
    pub fn new(project: Project, store: S) -> Self {
        Self {
            session: Session::new(project),
            store,
            version: None,
        }
    }

    /// Open a validated snapshot and mark exactly those observed bytes as saved.
    /// # Errors
    /// Returns the store's original failure without creating a partial document.
    pub fn open(store: S) -> Result<Self, S::Error> {
        let snapshot = store.load_snapshot()?;
        let mut session = Session::new(snapshot.project);
        session.mark_saved();
        Ok(Self {
            session,
            store,
            version: Some(snapshot.version),
        })
    }

    /// Inspect project, revision, dirty state, and history without mutable access.
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Commit one edit through the session's invariant-preserving command boundary.
    /// # Errors
    /// Reports stale tokens, invalid electrical edits, or revision exhaustion.
    pub fn execute(&mut self, expected: Revision, command: Command) -> Result<bool, EditError> {
        self.session.execute(expected, command)
    }

    /// Commit an ordered group as one undo operation.
    /// # Errors
    /// Rejected groups restore the full original state; see [`Session::execute_batch`].
    pub fn execute_batch(
        &mut self,
        expected: Revision,
        commands: Vec<Command>,
    ) -> Result<bool, EditError> {
        self.session.execute_batch(expected, commands)
    }

    /// Undo a retained transaction without changing its storage checkpoint.
    /// # Errors
    /// Reports stale tokens, invalid inverse edits, or revision exhaustion.
    pub fn undo(&mut self, expected: Revision) -> Result<bool, EditError> {
        self.session.undo(expected)
    }

    /// Redo an undone transaction without changing its storage checkpoint.
    /// # Errors
    /// Reports stale tokens, invalid forward edits, or revision exhaustion.
    pub fn redo(&mut self, expected: Revision) -> Result<bool, EditError> {
        self.session.redo(expected)
    }

    /// Save only against this document's observed store revision.
    /// # Errors
    /// Conflicts and IO/encoding errors leave the checkpoint, model, and history intact.
    pub fn save(&mut self) -> Result<(), S::Error> {
        let version = self
            .store
            .save_if_unchanged(self.session.project(), self.version.as_ref())?;
        self.version = Some(version);
        self.session.mark_saved();
        Ok(())
    }

    /// Copy to a new destination and bind it only after successful creation.
    /// Keeps project lineage and history; this is not a fork into a new design.
    /// # Errors
    /// Existing destinations and IO errors preserve the current binding/checkpoint.
    pub fn save_as(&mut self, store: S) -> Result<(), S::Error> {
        let version = store.save_if_unchanged(self.session.project(), None)?;
        self.store = store;
        self.version = Some(version);
        self.session.mark_saved();
        Ok(())
    }

    /// Whether the destination differs from the last observed bytes or existence state.
    /// Invalid replacement content still counts as changed; it is not loaded here.
    /// # Errors
    /// Returns the store's IO inspection failure without changing the session.
    pub fn has_external_changes(&self) -> Result<bool, S::Error> {
        Ok(self.store.current_version()? != self.version)
    }

    /// Validate and replace the current project, resetting history and session epoch.
    ///
    /// Reloading another project lineage is rejected even with discard permission.
    /// Failure leaves both the session and observed store version unchanged.
    /// # Errors
    /// Rejects stale tokens, unapproved unsaved edits, IO/validation errors, and
    /// different project identities. Explicitly open other projects separately.
    pub fn reload(
        &mut self,
        expected: Revision,
        policy: ReloadPolicy,
    ) -> Result<(), DocumentError<S::Error>> {
        self.session
            .check_revision(expected)
            .map_err(DocumentError::Edit)?;
        if self.session.is_dirty() && policy == ReloadPolicy::RequireClean {
            return Err(DocumentError::UnsavedChanges);
        }
        let snapshot = self.store.load_snapshot().map_err(DocumentError::Store)?;
        let current = self.session.project().id();
        if snapshot.project.id() != current {
            return Err(DocumentError::DifferentProject {
                expected: current,
                actual: snapshot.project.id(),
            });
        }
        self.session.replace_saved(snapshot.project);
        self.version = Some(snapshot.version);
        Ok(())
    }
}
