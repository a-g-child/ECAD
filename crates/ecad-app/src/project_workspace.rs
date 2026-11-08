//! Several independent project controllers in one workspace; every operation names its target.
//! Host locations are opaque normalized keys, not persistent project IDs. The host must derive
//! matching store/key pairs (including path alias policy); project files cannot choose these keys.
use crate::*;
use ecad_domain::Project;

/// Process-local handle for an open project, distinct from its persistent lineage and file path.
/// Remains stable across reload; never persist it in project files or restore it after restart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkspaceProjectId(u64);
impl WorkspaceProjectId {
    /// Inspect the runtime handle for transport/diagnostics.
    pub fn get(self) -> u64 {
        self.0
    }
}
/// Explicit policy for closing one project with local changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseProjectPolicy {
    /// Keep dirty projects open until saved.
    RequireClean,
    /// Drop local edits/history after the caller deliberately chose discard.
    DiscardLocalChanges,
}
/// A project entry with its own store, revision, undo/redo and saved checkpoint.
pub struct OpenProject<S: VersionedProjectStore> {
    id: WorkspaceProjectId,
    location: String,
    document: Document<S>,
}
impl<S: VersionedProjectStore> OpenProject<S> {
    /// Stable runtime workspace handle.
    pub fn id(&self) -> WorkspaceProjectId {
        self.id
    }
    /// Host-normalized location key. This is not a project title or lineage ID.
    pub fn location(&self) -> &str {
        &self.location
    }
    /// Read-only controller access; mutation goes through explicit project-targeted operations.
    pub fn document(&self) -> &Document<S> {
        &self.document
    }
}
/// Own independent project sessions in opening order, with one transient active selection.
/// One action dispatcher consumes a workspace-wide input sequence across project switches.
/// Storage IO is delegated to the existing store contract; path normalization and GUI state
/// remain host responsibilities.
pub struct ProjectWorkspace<S: VersionedProjectStore> {
    projects: Vec<OpenProject<S>>,
    active: Option<WorkspaceProjectId>,
    dispatcher: ActionDispatcher,
}
impl<S: VersionedProjectStore> Default for ProjectWorkspace<S> {
    fn default() -> Self {
        Self {
            projects: vec![],
            active: None,
            dispatcher: ActionDispatcher::default(),
        }
    }
}
impl<S: VersionedProjectStore> ProjectWorkspace<S> {
    /// Inspect open projects in stable opening order.
    pub fn projects(&self) -> impl ExactSizeIterator<Item = &OpenProject<S>> {
        self.projects.iter()
    }
    /// Currently selected project; selecting never changes project history.
    pub fn active(&self) -> Option<WorkspaceProjectId> {
        self.active
    }
    /// Read a specific project, even when another is active.
    /// # Errors
    /// Rejects closed/foreign runtime handles.
    pub fn project(
        &self,
        id: WorkspaceProjectId,
    ) -> Result<&OpenProject<S>, WorkspaceError<S::Error>> {
        self.projects
            .iter()
            .find(|p| p.id == id)
            .ok_or(WorkspaceError::UnknownProject(id))
    }
    fn project_mut(
        &mut self,
        id: WorkspaceProjectId,
    ) -> Result<&mut OpenProject<S>, WorkspaceError<S::Error>> {
        self.projects
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or(WorkspaceError::UnknownProject(id))
    }
    fn location(value: impl Into<String>) -> Result<String, WorkspaceError<S::Error>> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(WorkspaceError::EmptyLocation);
        }
        Ok(value)
    }
    fn unused_location(&self, location: &str) -> Result<(), WorkspaceError<S::Error>> {
        if self.projects.iter().any(|p| p.location == location) {
            return Err(WorkspaceError::LocationInUse(location.into()));
        }
        Ok(())
    }
    fn insert(&mut self, document: Document<S>, location: String) -> WorkspaceProjectId {
        // Capture the initial session identity only once. Reload allocates a new revision epoch
        // inside the same entry; closing drops ownership so a stale handle cannot be reinserted.
        let id = WorkspaceProjectId(document.session().revision().epoch().get());
        self.projects.push(OpenProject {
            id,
            location,
            document,
        });
        self.active = Some(id);
        id
    }
    /// Begin a separate unsaved project at an unused host-normalized destination key.
    /// Does not inspect or write the store; first save still requires an absent destination.
    /// # Errors
    /// Empty or already-reserved locations reject creation without changing active state.
    pub fn new_project(
        &mut self,
        project: Project,
        store: S,
        location: impl Into<String>,
    ) -> Result<WorkspaceProjectId, WorkspaceError<S::Error>> {
        let location = Self::location(location)?;
        self.unused_location(&location)?;
        Ok(self.insert(Document::new(project, store), location))
    }
    /// Open and validate another project, or focus the session already open at this location.
    /// Duplicate opens do not reread files or discard that session's edits. File copies may share
    /// persistent ProjectId while retaining different locations, runtime handles and histories.
    /// # Errors
    /// Load/key failures preserve every open project and the active selection.
    pub fn open(
        &mut self,
        store: S,
        location: impl Into<String>,
    ) -> Result<WorkspaceProjectId, WorkspaceError<S::Error>> {
        let location = Self::location(location)?;
        if let Some(id) = self
            .projects
            .iter()
            .find(|p| p.location == location)
            .map(|p| p.id)
        {
            self.active = Some(id);
            return Ok(id);
        }
        let document = Document::open(store).map_err(WorkspaceError::Store)?;
        Ok(self.insert(document, location))
    }
    /// Select an already-open project without editing or resetting its session.
    /// # Errors
    /// Unknown handles leave the active selection unchanged.
    pub fn activate(&mut self, id: WorkspaceProjectId) -> Result<(), WorkspaceError<S::Error>> {
        self.project(id)?;
        self.active = Some(id);
        Ok(())
    }
    /// Commit a command to its captured project, independently of the active selection.
    /// # Errors
    /// Unknown targets, cross-project/stale revisions and invalid edits leave state intact.
    pub fn execute(
        &mut self,
        id: WorkspaceProjectId,
        expected: Revision,
        command: Command,
    ) -> Result<bool, WorkspaceError<S::Error>> {
        self.project_mut(id)?
            .document
            .execute(expected, command)
            .map_err(WorkspaceError::Edit)
    }
    /// Commit one compound transaction to an explicitly targeted project.
    /// # Errors
    /// Invalid/stale groups roll back through the document's normal transaction contract.
    pub fn execute_batch(
        &mut self,
        id: WorkspaceProjectId,
        expected: Revision,
        commands: Vec<Command>,
    ) -> Result<bool, WorkspaceError<S::Error>> {
        self.project_mut(id)?
            .document
            .execute_batch(expected, commands)
            .map_err(WorkspaceError::Edit)
    }
    /// Undo only the addressed project's last transaction.
    /// # Errors
    /// Rejects unknown targets, stale/cross-project revisions or invalid inverse edits.
    pub fn undo(
        &mut self,
        id: WorkspaceProjectId,
        expected: Revision,
    ) -> Result<bool, WorkspaceError<S::Error>> {
        self.project_mut(id)?
            .document
            .undo(expected)
            .map_err(WorkspaceError::Edit)
    }
    /// Redo only the addressed project's last undone transaction.
    /// # Errors
    /// Rejects unknown targets, stale/cross-project revisions or invalid forward edits.
    pub fn redo(
        &mut self,
        id: WorkspaceProjectId,
        expected: Revision,
    ) -> Result<bool, WorkspaceError<S::Error>> {
        self.project_mut(id)?
            .document
            .redo(expected)
            .map_err(WorkspaceError::Edit)
    }
    /// Save the captured project against its own observed store version.
    /// # Errors
    /// Revision, IO and external-change failures preserve its checkpoint and all other projects.
    pub fn save(
        &mut self,
        id: WorkspaceProjectId,
        expected: Revision,
    ) -> Result<(), WorkspaceError<S::Error>> {
        let p = self.project_mut(id)?;
        p.document
            .session()
            .check_revision(expected)
            .map_err(WorkspaceError::Edit)?;
        p.document.save().map_err(WorkspaceError::Store)
    }
    /// Save the captured project to a new unreserved location, rebinding only on success.
    /// Host normalization must be applied to both the store and key before calling.
    /// # Errors
    /// Unknown/stale targets, reserved destinations or failed IO preserve the original binding.
    pub fn save_as(
        &mut self,
        id: WorkspaceProjectId,
        expected: Revision,
        store: S,
        location: impl Into<String>,
    ) -> Result<(), WorkspaceError<S::Error>> {
        let location = Self::location(location)?;
        self.project(id)?
            .document
            .session()
            .check_revision(expected)
            .map_err(WorkspaceError::Edit)?;
        self.unused_location(&location)?;
        let p = self.project_mut(id)?;
        p.document.save_as(store).map_err(WorkspaceError::Store)?;
        p.location = location;
        Ok(())
    }
    /// Reload only one project, retaining its workspace handle while renewing its revision epoch.
    /// # Errors
    /// Dirty/stale/invalid/different-lineage replacements preserve the existing session/binding.
    pub fn reload(
        &mut self,
        id: WorkspaceProjectId,
        expected: Revision,
        policy: ReloadPolicy,
    ) -> Result<(), WorkspaceError<S::Error>> {
        self.project_mut(id)?
            .document
            .reload(expected, policy)
            .map_err(WorkspaceError::Reload)
    }
    /// Dispatch a shared-registry action to its captured project, never a later active project.
    /// # Errors
    /// Rejects unknown projects, replayed inputs and normal context/command/lifecycle failures.
    pub fn invoke(
        &mut self,
        id: WorkspaceProjectId,
        registry: &ActionRegistry,
        invocation: ActionInvocation,
    ) -> Result<ActionOutcome, WorkspaceError<S::Error>> {
        let Some(p) = self.projects.iter_mut().find(|p| p.id == id) else {
            self.dispatcher
                .record_input(invocation.input_id)
                .map_err(|id| WorkspaceError::Action(ActionDispatchError::ReplayedInput(id)))?;
            return Err(WorkspaceError::UnknownProject(id));
        };
        self.dispatcher
            .invoke(registry, invocation, Some(&mut p.document))
            .map_err(WorkspaceError::Action)
    }
    /// List every dirty project, not just the active one, for host close/quit preflight.
    pub fn dirty_projects(&self) -> Vec<WorkspaceProjectId> {
        self.projects
            .iter()
            .filter(|p| p.document.session().is_dirty())
            .map(|p| p.id)
            .collect()
    }
    /// Close one captured project, requiring clean state unless discard was explicitly chosen.
    /// Closing active selects the next entry, or the preceding entry when closing the last.
    /// # Errors
    /// Unknown/stale targets and unapproved local edits leave every entry/selection intact.
    pub fn close(
        &mut self,
        id: WorkspaceProjectId,
        expected: Revision,
        policy: CloseProjectPolicy,
    ) -> Result<(), WorkspaceError<S::Error>> {
        let p = self.project(id)?;
        p.document
            .session()
            .check_revision(expected)
            .map_err(WorkspaceError::Edit)?;
        if p.document.session().is_dirty() && policy == CloseProjectPolicy::RequireClean {
            return Err(WorkspaceError::UnsavedChanges(id));
        }
        let index = self
            .projects
            .iter()
            .position(|p| p.id == id)
            .expect("checked project");
        self.projects.remove(index);
        if self.active == Some(id) {
            self.active = self
                .projects
                .get(index)
                .or_else(|| self.projects.last())
                .map(|p| p.id);
        }
        Ok(())
    }
}
