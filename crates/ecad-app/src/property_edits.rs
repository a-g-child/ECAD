//! Preflight heterogeneous property changes, then commit through normal transactions.

use crate::Command;
use ecad_domain::{ModelError, ObjectRef, Project, PropertyId, PropertyValue};
use std::collections::{BTreeMap, BTreeSet};

/// One actual proposed field change, suitable for review before committing the plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyChange {
    /// Existing addressed object.
    pub target: ObjectRef,
    /// Field's stable semantic identity.
    pub property: PropertyId,
    /// Current value at that point in the ordered proposal.
    pub before: PropertyValue,
    /// Proposed validated value.
    pub after: PropertyValue,
}

/// A fully validated proposal with exact effects and explicit unsupported pairs.
/// Submit `commands` as one batch against the session revision observed during planning.
/// Commit revalidates each field; this plan does not bypass revision/access checks.
pub struct PropertyEditPlan {
    /// Normal localized-inverse commands; exact no-ops are omitted.
    pub commands: Vec<Command>,
    /// Actual field changes in the deterministic application order.
    pub changes: Vec<PropertyChange>,
    /// Inapplicable object/field pairs, the only automatically skipped cases.
    pub inapplicable: BTreeSet<(ObjectRef, PropertyId)>,
    /// Applicable object/field pairs whose exact value already matches, including read-only no-ops.
    pub unchanged: BTreeSet<(ObjectRef, PropertyId)>,
}

/// Plan compounded property edits across a retained mixed selection.
///
/// Deduplicate targets and apply in property-ID then object-ID order on a temporary
/// candidate. Skip only inapplicable pairs; reject actual read-only/invalid edits as
/// a whole. This is ordered preflight, not a solver for label swaps or simultaneous
/// topology migrations. Source project and session history remain untouched.
/// # Errors
/// Missing targets, wrong types/reference kinds, restrictions, and normal field/topology
/// validation failures reject the proposal. The model error identifies the restriction.
pub fn property_edit_plan(
    project: &Project,
    selection: impl IntoIterator<Item = ObjectRef>,
    proposals: BTreeMap<PropertyId, PropertyValue>,
) -> Result<PropertyEditPlan, ModelError> {
    let targets: BTreeSet<_> = selection.into_iter().collect();
    for target in &targets {
        project.require_object(*target)?;
    }
    let mut candidate = project.clone();
    let mut plan = PropertyEditPlan {
        commands: Vec::new(),
        changes: Vec::new(),
        inapplicable: BTreeSet::new(),
        unchanged: BTreeSet::new(),
    };
    for (property, value) in proposals {
        for target in &targets {
            if !property.applies_to(*target) {
                plan.inapplicable.insert((*target, property));
                continue;
            }
            let before = candidate.set_property(*target, property, value.clone())?;
            if before == value {
                plan.unchanged.insert((*target, property));
                continue;
            }
            plan.changes.push(PropertyChange {
                target: *target,
                property,
                before,
                after: value.clone(),
            });
            plan.commands.push(Command::SetProperty {
                target: *target,
                property,
                value: value.clone(),
            });
        }
    }
    Ok(plan)
}
