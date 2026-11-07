//! Cable-level overview intent over the same conductors used in detailed wiring.

use crate::{CableId, ConductorId, DeviceId, End, ModelError, SubsystemId};
use std::collections::BTreeSet;

/// A cable overview endpoint; subsystem cores must use exposed terminals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CableEndpoint {
    /// A particular device occurrence.
    Device(DeviceId),
    /// A functional subsystem's external interface.
    Subsystem(SubsystemId),
}

/// One assembly with ordered overview ends and individually identified conductors.
///
/// Conductor A/B attachments correspond to cable A/B ends. Empty assemblies and
/// unterminated cores are valid drafts. Shield/pair and type-library data are later work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cable {
    id: CableId,
    designation: String,
    endpoints: [CableEndpoint; 2],
    conductors: BTreeSet<ConductorId>,
}

impl Cable {
    /// Construct a cable value; insertion validates endpoint existence and core wiring.
    /// # Errors
    /// Rejects blank designations or duplicate core identities within the supplied list.
    pub fn new(
        id: CableId,
        designation: impl Into<String>,
        a: CableEndpoint,
        b: CableEndpoint,
        conductors: impl IntoIterator<Item = ConductorId>,
    ) -> Result<Self, ModelError> {
        let designation = designation.into();
        if designation.trim().is_empty() {
            return Err(ModelError::EmptyLabel);
        }
        let mut members = BTreeSet::new();
        for conductor in conductors {
            if !members.insert(conductor) {
                return Err(ModelError::DuplicateCableMember(conductor));
            }
        }
        Ok(Self {
            id,
            designation,
            endpoints: [a, b],
            conductors: members,
        })
    }

    /// Return the assembly identity.
    pub fn id(&self) -> CableId {
        self.id
    }
    /// Return the human-readable cable identifier.
    pub fn designation(&self) -> &str {
        &self.designation
    }
    /// Return an ordered overview endpoint; this does not imply signal direction.
    pub fn endpoint(&self, end: End) -> CableEndpoint {
        self.endpoints[match end {
            End::A => 0,
            End::B => 1,
        }]
    }
    /// Iterate core identities deterministically.
    pub fn conductors(&self) -> impl ExactSizeIterator<Item = ConductorId> + '_ {
        self.conductors.iter().copied()
    }
}
