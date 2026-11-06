//! Recoverable electrical-model validation failures.

use crate::{ConductorId, DeviceId, TerminalId};
use std::{error::Error, fmt};

/// A proposed edit violates structural validity. Rejected edits leave state intact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelError {
    /// A required designation or terminal number contains only whitespace.
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
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLabel => f.write_str("designation or terminal number cannot be blank"),
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
        }
    }
}

impl Error for ModelError {}
