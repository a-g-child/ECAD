//! Immutable object values; project-level edits maintain cross-object references.

use crate::{ConductorId, DeviceId, ModelError, SubsystemId, TerminalId};
use std::collections::{BTreeMap, BTreeSet};

/// An electrical connection anchor owned by exactly one device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terminal {
    id: TerminalId,
    number: String,
}

impl Terminal {
    /// Create a terminal without interpreting its number as its identity.
    /// # Errors
    /// Returns [`ModelError::EmptyLabel`] for a blank number.
    pub fn new(id: TerminalId, number: impl Into<String>) -> Result<Self, ModelError> {
        let number = number.into();
        require_label(&number)?;
        Ok(Self { id, number })
    }

    /// Return the stable electrical identity.
    pub fn id(&self) -> TerminalId {
        self.id
    }

    /// Return the displayed terminal number, which may be numeric or lettered.
    pub fn number(&self) -> &str {
        &self.number
    }
}

/// One device occurrence with its own set of connection anchors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    id: DeviceId,
    designation: String,
    terminals: BTreeMap<TerminalId, Terminal>,
    subsystem: Option<SubsystemId>,
}

impl Device {
    /// Create a generic device; manufacturer assignment is not required.
    /// # Errors
    /// Rejects blank designations and duplicate terminal IDs or exact numbers.
    /// Input order has no effect on canonical terminal iteration.
    pub fn new(
        id: DeviceId,
        designation: impl Into<String>,
        terminals: impl IntoIterator<Item = Terminal>,
    ) -> Result<Self, ModelError> {
        let designation = designation.into();
        require_label(&designation)?;
        let mut by_id = BTreeMap::new();
        let mut numbers = BTreeSet::new();
        for terminal in terminals {
            if by_id.contains_key(&terminal.id) {
                return Err(ModelError::DuplicateTerminal(terminal.id));
            }
            if !numbers.insert(terminal.number.clone()) {
                return Err(ModelError::DuplicateTerminalNumber(terminal.number));
            }
            by_id.insert(terminal.id, terminal);
        }
        Ok(Self {
            id,
            designation,
            terminals: by_id,
            subsystem: None,
        })
    }

    /// Return the occurrence identity.
    pub fn id(&self) -> DeviceId {
        self.id
    }

    /// Return the user-visible designation; not an identity or a lookup key.
    pub fn designation(&self) -> &str {
        &self.designation
    }

    /// Iterate terminals in stable identity order, not displayed-number order.
    pub fn terminals(&self) -> impl ExactSizeIterator<Item = &Terminal> {
        self.terminals.values()
    }

    /// Attach this value to a block; insertion validates that the block exists.
    pub fn in_subsystem(mut self, subsystem: SubsystemId) -> Self {
        self.subsystem = Some(subsystem);
        self
    }
    /// Return its immediate functional owner, or project root.
    pub fn subsystem(&self) -> Option<SubsystemId> {
        self.subsystem
    }
    pub(crate) fn set_subsystem(&mut self, subsystem: Option<SubsystemId>) {
        self.subsystem = subsystem;
    }
}

/// One of the two ends of a conductor; not an indication of signal direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    /// First endpoint.
    A,
    /// Second endpoint.
    B,
}

/// One conductor, including valid partly or completely unterminated drafts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conductor {
    id: ConductorId,
    endpoints: [Option<TerminalId>; 2],
}

impl Conductor {
    /// Create a conductor value. Project insertion validates any terminal references.
    pub fn new(id: ConductorId, a: Option<TerminalId>, b: Option<TerminalId>) -> Self {
        Self {
            id,
            endpoints: [a, b],
        }
    }

    /// Return the stable conductor identity.
    pub fn id(&self) -> ConductorId {
        self.id
    }

    /// Return a terminal attachment, or `None` for an intentionally free end.
    pub fn endpoint(&self, end: End) -> Option<TerminalId> {
        self.endpoints[match end {
            End::A => 0,
            End::B => 1,
        }]
    }

    pub(crate) fn set_endpoint(&mut self, end: End, terminal: Option<TerminalId>) {
        self.endpoints[match end {
            End::A => 0,
            End::B => 1,
        }] = terminal;
    }
}

fn require_label(value: &str) -> Result<(), ModelError> {
    if value.trim().is_empty() {
        Err(ModelError::EmptyLabel)
    } else {
        Ok(())
    }
}
