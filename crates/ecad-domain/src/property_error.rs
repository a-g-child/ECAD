//! Property-specific failures retain typed targets and stable field identities.

use crate::{ObjectRef, PropertyId, PropertyRestriction, PropertyValueKind};
use std::{error::Error, fmt};

/// Property inspection/editing failed; unsupported, read-only, and invalid are distinct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyError {
    /// Only graphical style properties support reset to inheritance.
    NotInheritable(PropertyId),
    /// The object kind does not support this field.
    NotApplicable {
        /// Addressed object.
        target: ObjectRef,
        /// Inapplicable field.
        property: PropertyId,
    },
    /// The proposed value has the wrong runtime or typed-reference kind.
    WrongType {
        /// Field whose contract was violated.
        property: PropertyId,
        /// Expected outer runtime kind; references also require the correct object kind.
        expected: PropertyValueKind,
    },
    /// The field exists but cannot currently be edited.
    ReadOnly {
        /// Addressed object.
        target: ObjectRef,
        /// Restricted field.
        property: PropertyId,
        /// Identity, structural, or pinned-instance restriction.
        reason: PropertyRestriction,
    },
}
impl fmt::Display for PropertyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInheritable(property) => {
                write!(f, "property {property:?} does not inherit a style")
            }
            Self::NotApplicable { target, property } => {
                write!(f, "property {property:?} does not apply to {target:?}")
            }
            Self::WrongType { property, expected } => write!(
                f,
                "property {property:?} requires {expected:?} with the correct reference kind"
            ),
            Self::ReadOnly {
                target,
                property,
                reason,
            } => write!(
                f,
                "property {property:?} on {target:?} is read-only: {reason:?}"
            ),
        }
    }
}
impl Error for PropertyError {}
