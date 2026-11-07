//! Project-local subsystem occurrences and explicit terminal exposure.

use crate::{InterfacePortId, ModelError, SubsystemId, TerminalId};

/// Provisional resource guard, measured by the hierarchy probe; not a recursion limit.
/// Root-level subsystems have depth one. Traversal is iterative and identity ordered.
pub const MAX_SUBSYSTEM_DEPTH: usize = 256;

/// A placed functional block. Shared definition/revision adoption is not implied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subsystem {
    id: SubsystemId,
    name: String,
    parent: Option<SubsystemId>,
}

impl Subsystem {
    /// Create an occurrence; insertion checks its parent, cycle, and depth.
    /// # Errors
    /// Rejects blank names.
    pub fn new(
        id: SubsystemId,
        name: impl Into<String>,
        parent: Option<SubsystemId>,
    ) -> Result<Self, ModelError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(ModelError::EmptyLabel);
        }
        Ok(Self { id, name, parent })
    }

    /// Return the stable occurrence identity.
    pub fn id(&self) -> SubsystemId {
        self.id
    }
    /// Return the functional name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Return the containing block, or `None` for project root.
    pub fn parent(&self) -> Option<SubsystemId> {
        self.parent
    }
    pub(crate) fn set_parent(&mut self, parent: Option<SubsystemId>) {
        self.parent = parent;
    }
}

/// A named external interface mapped to a real internal terminal.
///
/// No duplicate terminal or conductor is created. Nested exposure requires that
/// every intervening child boundary already exposes this same terminal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfacePort {
    id: InterfacePortId,
    subsystem: SubsystemId,
    name: String,
    terminal: TerminalId,
}

impl InterfacePort {
    /// Define an exposure; project insertion checks containment and nested boundaries.
    /// # Errors
    /// Rejects blank names. Port names must also be unique within their owner on insertion.
    pub fn new(
        id: InterfacePortId,
        subsystem: SubsystemId,
        name: impl Into<String>,
        terminal: TerminalId,
    ) -> Result<Self, ModelError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(ModelError::EmptyLabel);
        }
        Ok(Self {
            id,
            subsystem,
            name,
            terminal,
        })
    }

    /// Return this interface identity.
    pub fn id(&self) -> InterfacePortId {
        self.id
    }
    /// Return the owning block.
    pub fn subsystem(&self) -> SubsystemId {
        self.subsystem
    }
    /// Return its external label.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Return the actual connection anchor; independent of drawing position.
    pub fn terminal(&self) -> TerminalId {
        self.terminal
    }
}
