//! Reviewable, explicit revision adoption for exact pinned occurrences.

use crate::Command;
use ecad_domain::{
    DefinitionRef, InstanceIds, InstanceLimits, ModelError, Project, ReuseError, RevisionChanges,
    SubsystemId,
};
use std::collections::BTreeMap;

/// An adoption preview validated against one project state.
/// Execute `command` using the session revision observed when planning. Plans do not
/// auto-follow newer revisions; stale bindings and resulting conflicts are rechecked.
pub struct AdoptionPlan {
    /// Deterministic source-body changes suitable for a future review panel.
    pub changes: RevisionChanges,
    /// One atomic, reversible replacement, retaining only bindings in history.
    pub command: Command,
}

fn preserve<K: Copy + Ord>(ids: &mut BTreeMap<K, K>, previous: &BTreeMap<K, K>) {
    for (local, placed) in ids {
        if let Some(existing) = previous.get(local) {
            *placed = *existing;
        }
    }
}

/// Prepare adoption of a bundled revision for one pinned occurrence.
///
/// Supply complete target mappings (for example from the OS-random adapter). Values
/// for surviving local IDs are replaced with the existing occurrence IDs. Newly added
/// objects retain supplied fresh IDs. The root name and parent stay unchanged.
/// # Errors
/// Rejects local variants, missing revisions, malformed maps, expansion limits,
/// incompatible identities, destination collisions, and broken external references.
/// Conflicts leave the project unchanged; resolve external wiring explicitly first.
pub fn adoption_plan(
    project: &Project,
    root: SubsystemId,
    source: DefinitionRef,
    mut ids: InstanceIds,
    limits: InstanceLimits,
) -> Result<AdoptionPlan, ModelError> {
    let expected = project
        .instance(root)
        .ok_or(ReuseError::MissingInstance(root))?;
    let before = project
        .definition(expected.source())
        .expect("pin has source");
    let after = project
        .definition(source)
        .ok_or(ReuseError::MissingRevision(source))?;
    let old = expected.ids();
    preserve(&mut ids.subsystems, &old.subsystems);
    preserve(&mut ids.devices, &old.devices);
    preserve(&mut ids.terminals, &old.terminals);
    preserve(&mut ids.ports, &old.ports);
    preserve(&mut ids.conductors, &old.conductors);
    preserve(&mut ids.cables, &old.cables);
    let block = project.subsystem(root).expect("pin has root");
    let target = after
        .materialize(ids, block.parent(), block.name(), limits)?
        .binding;
    // Preview uses the same engine path as commit. This full candidate validation is
    // deliberately conservative; no intermediate deletion is exposed to the caller.
    project.validate_adoption(expected, &target)?;
    Ok(AdoptionPlan {
        changes: RevisionChanges::between(before, after),
        command: Command::AdoptInstance {
            expected: expected.clone(),
            target,
        },
    })
}
