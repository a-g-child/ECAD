//! Reusable-definition and instance validation failures.

use crate::{DefinitionRef, SubsystemId};
use std::{error::Error, fmt};

/// A reusable revision or occurrence would violate identity or immutability rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReuseError {
    /// A reusable body must contain exactly one rooted hierarchy and no root-level devices.
    InvalidDefinitionRoot,
    /// Nested reusable references are deferred; local nested blocks remain supported.
    NestedDefinitionsUnsupported,
    /// Adoption must preserve lineage, root identity, and surviving local-to-placed IDs.
    IncompatibleAdoption,
    /// The occurrence no longer has the binding used to prepare this adoption.
    StaleAdoption(SubsystemId),
    /// A published key already names different immutable content.
    RevisionConflict(DefinitionRef),
    /// A persisted catalog repeats one revision key instead of describing it once.
    DuplicateRevision(DefinitionRef),
    /// The required revision is not bundled into this project.
    MissingRevision(DefinitionRef),
    /// A pin or retained local origin still references this revision.
    RevisionInUse(DefinitionRef),
    /// A mapping is incomplete, has extra keys, or reuses an occurrence ID.
    InvalidMapping,
    /// Materialization exceeds its explicit object budget.
    ExpansionLimit,
    /// An occurrence is already bound at this root.
    InstanceExists(SubsystemId),
    /// There is no bound occurrence at this root.
    MissingInstance(SubsystemId),
    /// Forget retained ancestry explicitly before deleting a local root.
    LocalOriginExists(SubsystemId),
    /// No retained local-source ancestry exists at this root.
    MissingLocalOrigin(SubsystemId),
    /// Materialized content disagrees with its declared source and mapping.
    InstanceMismatch(SubsystemId),
    /// Occurrences must not share owned objects or nest pinned instances yet.
    InstanceOverlap(SubsystemId),
    /// Make an instance local before editing its definition-derived structure.
    ReadOnlyInstance(SubsystemId),
    /// External wiring must use a terminal exposed at the instance root.
    UnexposedInstanceTerminal(SubsystemId),
}
impl fmt::Display for ReuseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompatibleAdoption => {
                f.write_str("adoption must preserve lineage and compatible object identities")
            }
            Self::StaleAdoption(id) => {
                write!(f, "instance {id} changed since adoption was prepared")
            }
            Self::InvalidDefinitionRoot => {
                f.write_str("definition must be a self-contained rooted hierarchy")
            }
            Self::NestedDefinitionsUnsupported => {
                f.write_str("nested reusable dependencies are not supported yet")
            }
            Self::RevisionConflict(id) => {
                write!(f, "revision {id} already contains different immutable data")
            }
            Self::DuplicateRevision(id) => write!(f, "catalog repeats revision {id}"),
            Self::MissingRevision(id) => write!(f, "revision {id} is not available in the project"),
            Self::RevisionInUse(id) => {
                write!(f, "revision {id} is referenced by a pin or local origin")
            }
            Self::InvalidMapping => {
                f.write_str("instance mapping must cover every local object exactly once")
            }
            Self::ExpansionLimit => {
                f.write_str("instance exceeds its materialization object budget")
            }
            Self::InstanceExists(id) => write!(f, "instance {id} is already bound"),
            Self::MissingInstance(id) => write!(f, "instance {id} does not exist"),
            Self::LocalOriginExists(id) => write!(
                f,
                "forget local origin {id} explicitly before removing its root"
            ),
            Self::MissingLocalOrigin(id) => write!(f, "local origin {id} does not exist"),
            Self::InstanceMismatch(id) => {
                write!(f, "instance {id} differs from its pinned definition")
            }
            Self::InstanceOverlap(id) => {
                write!(f, "instance {id} overlaps another pinned occurrence")
            }
            Self::ReadOnlyInstance(id) => {
                write!(f, "make instance {id} local before editing its internals")
            }
            Self::UnexposedInstanceTerminal(id) => {
                write!(f, "external wiring must use instance {id}'s root interface")
            }
        }
    }
}
impl Error for ReuseError {}
