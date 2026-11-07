//! Recoverable electrical-model validation failures.

use crate::{CableId, ConductorId, DeviceId, End, InterfacePortId, SubsystemId, TerminalId};
use std::{error::Error, fmt};

/// A proposed edit violates structural validity. Rejected edits leave state intact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelError {
    /// Reusable content, mapping, or instance immutability validation failed.
    Reuse(crate::ReuseError),
    /// A required name, designation, or terminal number contains only whitespace.
    EmptyLabel,
    /// A device identity is already present.
    DuplicateDevice(DeviceId),
    /// A terminal identity is already owned by this or another device.
    DuplicateTerminal(TerminalId),
    /// Two terminals on one device have the same displayed number.
    DuplicateTerminalNumber(String),
    /// A conductor identity is already present.
    DuplicateConductor(ConductorId),
    /// A referenced device does not exist.
    MissingDevice(DeviceId),
    /// A referenced terminal does not exist.
    MissingTerminal(TerminalId),
    /// A referenced conductor does not exist.
    MissingConductor(ConductorId),
    /// A device cannot be removed while one of its terminals is connected.
    DeviceConnected(DeviceId),
    /// A device is still referenced by an interface or cable overview.
    DeviceReferenced(DeviceId),
    /// A subsystem identity already exists.
    DuplicateSubsystem(SubsystemId),
    /// A referenced functional block does not exist.
    MissingSubsystem(SubsystemId),
    /// A parent chain repeats an occurrence.
    HierarchyCycle(SubsystemId),
    /// The provisional hierarchy resource guard would be exceeded.
    HierarchyTooDeep,
    /// Remove children, devices, ports, and referencing cable overviews first.
    SubsystemInUse(SubsystemId),
    /// An interface identity already exists.
    DuplicatePort(InterfacePortId),
    /// A referenced interface does not exist.
    MissingPort(InterfacePortId),
    /// Port names must be unique within their owning block.
    PortNameInUse(SubsystemId, String),
    /// An interface cannot expose a terminal outside its own subtree.
    TerminalOutsideSubsystem(SubsystemId, TerminalId),
    /// A connection or nested exposure requires an explicit boundary mapping.
    InterfaceRequired(SubsystemId, TerminalId),
    /// A cable identity already exists.
    DuplicateCable(CableId),
    /// A referenced cable does not exist.
    MissingCable(CableId),
    /// A core was listed more than once in one assembly.
    DuplicateCableMember(ConductorId),
    /// A conductor belongs to a cable and cannot be removed or assigned elsewhere.
    ConductorInCable(ConductorId),
    /// A core attachment does not match its ordered overview endpoint.
    CableEndpointMismatch(CableId, ConductorId, End),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reuse(error) => error.fmt(f),
            Self::EmptyLabel => {
                f.write_str("required name, designation, or terminal number cannot be blank")
            }
            Self::DuplicateDevice(id) => write!(f, "device {id} already exists"),
            Self::DuplicateTerminal(id) => write!(f, "terminal {id} already exists"),
            Self::DuplicateTerminalNumber(number) => {
                write!(f, "terminal number {number} is duplicated")
            }
            Self::DuplicateConductor(id) => write!(f, "conductor {id} already exists"),
            Self::MissingDevice(id) => write!(f, "device {id} does not exist"),
            Self::MissingTerminal(id) => write!(f, "terminal {id} does not exist"),
            Self::MissingConductor(id) => write!(f, "conductor {id} does not exist"),
            Self::DeviceConnected(id) => {
                write!(f, "disconnect conductors before removing device {id}")
            }
            Self::DeviceReferenced(id) => {
                write!(f, "device {id} is referenced by an interface or cable")
            }
            Self::DuplicateSubsystem(id) => write!(f, "subsystem {id} already exists"),
            Self::MissingSubsystem(id) => write!(f, "subsystem {id} does not exist"),
            Self::HierarchyCycle(id) => write!(f, "cyclic subsystem parent chain at {id}"),
            Self::HierarchyTooDeep => {
                write!(f, "hierarchy exceeds {} levels", crate::MAX_SUBSYSTEM_DEPTH)
            }
            Self::SubsystemInUse(id) => write!(
                f,
                "subsystem {id} still contains or is referenced by objects"
            ),
            Self::DuplicatePort(id) => write!(f, "interface {id} already exists"),
            Self::MissingPort(id) => write!(f, "interface {id} does not exist"),
            Self::PortNameInUse(id, name) => {
                write!(f, "interface name {name} already exists on subsystem {id}")
            }
            Self::TerminalOutsideSubsystem(id, terminal) => {
                write!(f, "terminal {terminal} is outside subsystem {id}")
            }
            Self::InterfaceRequired(id, terminal) => write!(
                f,
                "subsystem {id} must explicitly expose terminal {terminal}"
            ),
            Self::DuplicateCable(id) => write!(f, "cable {id} already exists"),
            Self::MissingCable(id) => write!(f, "cable {id} does not exist"),
            Self::DuplicateCableMember(id) => write!(f, "conductor {id} is listed more than once"),
            Self::ConductorInCable(id) => write!(f, "conductor {id} belongs to a cable"),
            Self::CableEndpointMismatch(cable, conductor, end) => write!(
                f,
                "conductor {conductor} end {end:?} does not match cable {cable}"
            ),
        }
    }
}

impl Error for ModelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Reuse(error) => Some(error),
            _ => None,
        }
    }
}
impl From<crate::ReuseError> for ModelError {
    fn from(error: crate::ReuseError) -> Self {
        Self::Reuse(error)
    }
}
