//! One-pass electrical connectivity projection, independent of drawing count and output format.

use crate::*;
use ecad_domain::*;
use std::collections::BTreeMap;

/// Immutable report at one session revision; rebuild after edits rather than mutating cached text.
/// Data is electrical only: graphical representations do not duplicate devices or connections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionReport {
    project: ProjectId,
    revision: Revision,
    subsystems: BTreeMap<SubsystemId, ConnectionSubsystem>,
    devices: BTreeMap<DeviceId, ConnectionDevice>,
    terminals: BTreeMap<TerminalId, ConnectionTerminal>,
    conductors: BTreeMap<ConductorId, ConnectionConductor>,
    cables: BTreeMap<CableId, ConnectionCable>,
}
impl ConnectionReport {
    /// Capture current electrical values, including all spare ends and unconnected terminals.
    /// Iterates objects and attachments once, maintaining deterministic typed-ID maps.
    /// Never changes document history, dirty state, library content, or project serialization.
    pub fn from_session(session: &Session) -> Self {
        let project = session.project();
        let mut report = Self {
            project: project.id(),
            revision: session.revision(),
            subsystems: project
                .subsystems()
                .map(|s| {
                    (
                        s.id(),
                        ConnectionSubsystem {
                            id: s.id(),
                            name: s.name().into(),
                            parent: s.parent(),
                        },
                    )
                })
                .collect(),
            devices: BTreeMap::new(),
            terminals: BTreeMap::new(),
            conductors: BTreeMap::new(),
            cables: project
                .cables()
                .map(|c| {
                    (
                        c.id(),
                        ConnectionCable {
                            id: c.id(),
                            designation: c.designation().into(),
                            endpoints: [c.endpoint(End::A), c.endpoint(End::B)],
                            conductors: c.conductors().collect(),
                        },
                    )
                })
                .collect(),
        };
        for device in project.devices() {
            report.devices.insert(
                device.id(),
                ConnectionDevice {
                    id: device.id(),
                    designation: device.designation().into(),
                    subsystem: device.subsystem(),
                    terminals: device.terminals().map(Terminal::id).collect(),
                },
            );
            for terminal in device.terminals() {
                report.terminals.insert(
                    terminal.id(),
                    ConnectionTerminal {
                        id: terminal.id(),
                        device: device.id(),
                        number: terminal.number().into(),
                        connections: vec![],
                    },
                );
            }
        }
        for conductor in project.conductors() {
            report.conductors.insert(
                conductor.id(),
                ConnectionConductor {
                    id: conductor.id(),
                    endpoints: [conductor.endpoint(End::A), conductor.endpoint(End::B)],
                    cable: project.conductor_cable(conductor.id()),
                },
            );
            for end in [End::A, End::B] {
                if let Some(terminal) = conductor.endpoint(end) {
                    report
                        .terminals
                        .get_mut(&terminal)
                        .expect("validated project terminal")
                        .connections
                        .push(ConductorEnd {
                            conductor: conductor.id(),
                            end,
                        });
                }
            }
        }
        report
    }
    /// Persistent lineage, independent of file locations or session epochs.
    pub fn project_id(&self) -> ProjectId {
        self.project
    }
    /// Original session epoch/sequence; transient freshness metadata, not an export timestamp.
    pub fn revision(&self) -> Revision {
        self.revision
    }
    /// Whether this exact observation belongs to the current session state.
    /// Even drawing-only edits, undo to equal contents, and reopen invalidate it conservatively.
    pub fn is_current(&self, session: &Session) -> bool {
        self.project == session.project().id() && self.revision == session.revision()
    }
    /// Iterate all placed subsystems in typed identity order.
    pub fn subsystems(&self) -> impl ExactSizeIterator<Item = &ConnectionSubsystem> {
        self.subsystems.values()
    }
    /// Inspect an existing subsystem; absence differs from project-root containment.
    pub fn subsystem(&self, id: SubsystemId) -> Option<&ConnectionSubsystem> {
        self.subsystems.get(&id)
    }
    /// Iterate real device occurrences once, in identity order.
    pub fn devices(&self) -> impl ExactSizeIterator<Item = &ConnectionDevice> {
        self.devices.values()
    }
    /// Inspect a device and every terminal identity for a future live pin table.
    pub fn device(&self, id: DeviceId) -> Option<&ConnectionDevice> {
        self.devices.get(&id)
    }
    /// Iterate every terminal, including unconnected ones, in identity order.
    pub fn terminals(&self) -> impl ExactSizeIterator<Item = &ConnectionTerminal> {
        self.terminals.values()
    }
    /// Inspect a terminal's full connection list, never a guessed primary wire.
    pub fn terminal(&self, id: TerminalId) -> Option<&ConnectionTerminal> {
        self.terminals.get(&id)
    }
    /// Iterate individual conductors once, independent of route representation count.
    pub fn conductors(&self) -> impl ExactSizeIterator<Item = &ConnectionConductor> {
        self.conductors.values()
    }
    /// Inspect actual A/B assignments and cable membership.
    pub fn conductor(&self, id: ConductorId) -> Option<&ConnectionConductor> {
        self.conductors.get(&id)
    }
    /// Iterate all assemblies, including empty cable drafts, in identity order.
    pub fn cables(&self) -> impl ExactSizeIterator<Item = &ConnectionCable> {
        self.cables.values()
    }
    /// Inspect cable overview intent and actual core identities separately.
    pub fn cable(&self, id: CableId) -> Option<&ConnectionCable> {
        self.cables.get(&id)
    }
    /// Resolve containment from outermost occurrence to this subsystem, iteratively.
    /// Names are display values; stable IDs retain their meaning after renames.
    /// # Errors
    /// Rejects a caller-supplied subsystem identity absent from this snapshot.
    pub fn subsystem_path(&self, id: SubsystemId) -> Result<Vec<&ConnectionSubsystem>, ModelError> {
        let mut path = Vec::new();
        let mut current = Some(id);
        while let Some(id) = current {
            let scope = self
                .subsystems
                .get(&id)
                .ok_or(ModelError::MissingSubsystem(id))?;
            path.push(scope);
            current = scope.parent;
        }
        path.reverse();
        Ok(path)
    }
}
