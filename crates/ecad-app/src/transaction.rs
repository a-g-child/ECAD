//! Ordered edits with localized rollback; never expose intermediate model state.

use crate::Command;
use ecad_domain::{ModelError, Project};

#[derive(Clone, Debug)]
struct Change {
    forward: Command,
    inverse: Command,
}

/// A committed group of actual changes; no-op commands are omitted.
#[derive(Clone, Debug)]
pub(crate) struct Transaction {
    changes: Vec<Change>,
}

impl Transaction {
    /// Each edit sees its predecessors under one exclusive borrow. On failure,
    /// exact inverse edits restore the completed prefix before returning.
    pub(crate) fn apply(
        project: &mut Project,
        commands: impl IntoIterator<Item = Command>,
    ) -> Result<Option<Self>, ModelError> {
        let mut changes: Vec<Change> = Vec::new();
        for forward in commands {
            match forward.apply(project) {
                Ok(Some(inverse)) => changes.push(Change { forward, inverse }),
                Ok(None) => {}
                Err(error) => {
                    for change in changes.iter().rev() {
                        // No outside mutation can intervene. Failure here is an
                        // inverse implementation bug, not an expected user error.
                        change.inverse.apply(project).expect(
                            "validated transaction inverse must restore its preceding state",
                        );
                    }
                    return Err(error);
                }
            }
        }
        if changes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Self { changes }))
        }
    }

    pub(crate) fn undo(&self, project: &mut Project) -> Result<(), ModelError> {
        Self::apply(
            project,
            self.changes
                .iter()
                .rev()
                .map(|change| change.inverse.clone()),
        )?;
        Ok(())
    }

    pub(crate) fn redo(&self, project: &mut Project) -> Result<(), ModelError> {
        Self::apply(
            project,
            self.changes.iter().map(|change| change.forward.clone()),
        )?;
        Ok(())
    }
}
