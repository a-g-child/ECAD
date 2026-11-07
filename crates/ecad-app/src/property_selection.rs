//! Union-based property inspection without conflating mixed, absent, and restricted values.

use ecad_domain::{
    ModelError, ObjectRef, PROPERTY_IDS, Project, PropertyAccess, PropertyId, PropertyRestriction,
    PropertyValue,
};
use std::collections::{BTreeMap, BTreeSet};

/// Selection-level value state; a free endpoint/absent scope is still a uniform value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionValue {
    /// Every applicable object has this exact typed value.
    Uniform(PropertyValue),
    /// Applicable objects disagree; never represented as blank text or `None`.
    Mixed,
}

/// One property across a heterogeneous selection, including exact applicability/access.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionProperty {
    /// Uniform or mixed over applicable objects, including read-only values.
    pub value: SelectionValue,
    /// All selected objects supporting this property's meaning and type.
    pub applicable: BTreeSet<ObjectRef>,
    /// Applicable targets that permit validated edits.
    pub editable: BTreeSet<ObjectRef>,
    /// Applicable targets with explicit restrictions, not silently treated as unsupported.
    pub read_only: BTreeMap<ObjectRef, PropertyRestriction>,
    /// Selected objects that do not support this property at all.
    pub inapplicable: BTreeSet<ObjectRef>,
}

/// Read-only inspector projection over a deduplicated, stable-ordered selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertySelection {
    /// The complete selection, retained while editing different applicable subsets.
    pub objects: BTreeSet<ObjectRef>,
    /// Union of applicable properties. A row is common when its applicable count
    /// equals the selection size; an empty selection has no rows.
    pub properties: BTreeMap<PropertyId, SelectionProperty>,
}

/// Inspect the union of real fields over existing objects, without changing the model.
/// # Errors
/// Any missing selection target is an error, not an omitted unsupported object.
pub fn inspect_properties(
    project: &Project,
    selection: impl IntoIterator<Item = ObjectRef>,
) -> Result<PropertySelection, ModelError> {
    let objects: BTreeSet<_> = selection.into_iter().collect();
    for target in &objects {
        project.require_object(*target)?;
    }
    let mut properties = BTreeMap::new();
    for property in PROPERTY_IDS {
        let applicable: BTreeSet<_> = objects
            .iter()
            .copied()
            .filter(|target| property.applies_to(*target))
            .collect();
        let mut values = Vec::new();
        let mut editable = BTreeSet::new();
        let mut read_only = BTreeMap::new();
        for target in &applicable {
            let observation = project.property(*target, property)?;
            values.push(observation.value);
            match observation.access {
                PropertyAccess::Editable => {
                    editable.insert(*target);
                }
                PropertyAccess::ReadOnly(reason) => {
                    read_only.insert(*target, reason);
                }
            }
        }
        if let Some(first) = values.first() {
            let value = if values.iter().all(|value| value == first) {
                SelectionValue::Uniform(first.clone())
            } else {
                SelectionValue::Mixed
            };
            let inapplicable = objects.difference(&applicable).copied().collect();
            properties.insert(
                property,
                SelectionProperty {
                    value,
                    applicable,
                    editable,
                    read_only,
                    inapplicable,
                },
            );
        }
    }
    Ok(PropertySelection {
        objects,
        properties,
    })
}
