//! Immutable electrical report records; identities and ordered ends remain distinct from labels.

use ecad_domain::*;

/// A placed subsystem's current label and containment, independent of drawing sheets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionSubsystem {
    /// Stable occurrence identity, not reusable definition identity.
    pub id: SubsystemId,
    /// Current human-readable name.
    pub name: String,
    /// Immediate enclosing occurrence, or project root.
    pub parent: Option<SubsystemId>,
}
/// One real device occurrence, regardless of how many graphical representations exist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionDevice {
    /// Stable device identity.
    pub id: DeviceId,
    /// Current designation; never used as a lookup key.
    pub designation: String,
    /// Immediate functional owner, or project root.
    pub subsystem: Option<SubsystemId>,
    /// Every real terminal in identity order, including unconnected terminals.
    pub terminals: Vec<TerminalId>,
}
/// One actual conductor end attached to a terminal, not an inferred signal direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConductorEnd {
    /// Stable individual-conductor identity.
    pub conductor: ConductorId,
    /// Ordered A/B end. Self-loops contribute both ends to the same terminal.
    pub end: End,
}
/// One real terminal and all its attached conductor ends, never an arbitrary single wire.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionTerminal {
    /// Stable electrical identity.
    pub id: TerminalId,
    /// Its unique actual device owner.
    pub device: DeviceId,
    /// Current terminal number, independent of identity and sorting.
    pub number: String,
    /// Attached ends ordered by conductor identity, then A before B; empty means unconnected.
    pub connections: Vec<ConductorEnd>,
}
/// Deliberate completeness state; free ends are valid drafts, not missing references.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    /// Both ordered ends reference real terminals.
    BothAttached,
    /// A is free and B is attached.
    AFree,
    /// B is free and A is attached.
    BFree,
    /// Neither end has a terminal assignment.
    BothFree,
}
/// One conductor shared by all views, with real terminal and cable membership identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionConductor {
    /// Stable individual-conductor identity; no user wire designation exists in this slice.
    pub id: ConductorId,
    /// Ordered actual A/B terminal references; `None` is an intentionally free end.
    pub endpoints: [Option<TerminalId>; 2],
    /// Existing cable membership, or a standalone conductor.
    pub cable: Option<CableId>,
}
impl ConnectionConductor {
    /// Inspect attachment completeness without guessing from display names or coordinates.
    pub fn state(&self) -> ConnectionState {
        match self.endpoints {
            [Some(_), Some(_)] => ConnectionState::BothAttached,
            [None, Some(_)] => ConnectionState::AFree,
            [Some(_), None] => ConnectionState::BFree,
            [None, None] => ConnectionState::BothFree,
        }
    }
    /// Return the opposite ordered terminal; `None` indicates a genuine free destination.
    pub fn destination(&self, from: End) -> Option<TerminalId> {
        self.endpoints[match from {
            End::A => 1,
            End::B => 0,
        }]
    }
}
/// Cable overview intent and actual individually identified cores remain separate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionCable {
    /// Stable cable assembly identity.
    pub id: CableId,
    /// Current human-readable cable designation.
    pub designation: String,
    /// Ordered overview endpoints, not substituted for actual core terminations.
    pub endpoints: [CableEndpoint; 2],
    /// Every member conductor in identity order; empty assemblies are retained.
    pub conductors: Vec<ConductorId>,
}
