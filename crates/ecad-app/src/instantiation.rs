//! Build ordinary reversible commands from one pinned revision and explicit IDs.

use crate::Command;
use ecad_domain::{
    DefinitionRef, InstanceIds, InstanceLimits, ModelError, Project, ReuseError, SubsystemId,
};

/// Plan one independently identified occurrence as a single batch of normal commands.
///
/// The source must already be bundled. Apply the returned commands through
/// `Session::execute_batch` or `Document::execute_batch` with the current revision.
/// Root name/parent are local parameters; all internals become pinned at the final bind.
/// # Errors
/// Rejects unavailable revisions, invalid/incomplete mappings, and expansion budgets.
/// Destination conflicts are rejected atomically when the commands are applied.
pub fn instance_commands(
    project: &Project,
    source: DefinitionRef,
    ids: InstanceIds,
    parent: Option<SubsystemId>,
    name: &str,
    limits: InstanceLimits,
) -> Result<Vec<Command>, ModelError> {
    let revision = project
        .definition(source)
        .ok_or(ReuseError::MissingRevision(source))?;
    let plan = revision.materialize(ids, parent, name, limits)?;
    let mut commands = Vec::new();
    commands.extend(plan.subsystems.into_iter().map(Command::InsertSubsystem));
    commands.extend(plan.devices.into_iter().map(Command::InsertDevice));
    commands.extend(plan.ports.into_iter().map(Command::InsertPort));
    commands.extend(plan.conductors.into_iter().map(Command::InsertConductor));
    commands.extend(plan.cables.into_iter().map(Command::InsertCable));
    commands.push(Command::BindInstance(plan.binding));
    Ok(commands)
}
